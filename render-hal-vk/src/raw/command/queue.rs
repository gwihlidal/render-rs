#![allow(unused_variables)]

use ash;
use relevant::Relevant;
use smallvec::SmallVec;
use std::{
    collections::LinkedList, fmt::Debug, marker::PhantomData, ops::Add, ptr::null, sync::Arc,
};

use crate::raw::command::{buffer::*, capability::*, fence::*, QueueId};
use crate::raw::device::DeviceTracker;
use crate::raw::errors::{DeviceError, DeviceLost, OomError};
use crate::raw::object::VulkanObjects;

/// Vulkan's command queue marked with capability.
pub struct Queue<C = Capability> {
    fp: Arc<ash::vk::DeviceFnV1_0>,
    pub raw: ash::vk::Queue,
    pub id: QueueId<C>,

    /// Next epoch index for this queue
    next: u64,

    /// Free bucket nodes.
    free: LinkedList<Bucket>,

    /// Pending bucket nodes.
    pending: LinkedList<Bucket>,
}

impl<C> Drop for Queue<C> {
    fn drop(&mut self) {
        unsafe {
            self.fp.queue_wait_idle(self.raw);
        }
    }
}

impl<C> Queue<C>
where
    C: Copy + Debug,
{
    /// Submit commands for execution.
    /// This function accepts command buffer submissions with reference to `DeviceTracker` as tracker.
    pub unsafe fn submit_scoped<'a, I, W, B, S>(
        &mut self,
        submissions: I,
        fence: Option<UnarmedFence>,
    ) -> Result<Option<ArmedFence<C>>, DeviceError>
    where
        I: IntoIterator<Item = Submission<W, B, S>>,
        W: IntoIterator<Item = (ash::vk::Semaphore, ash::vk::PipelineStageFlags)>,
        B: IntoIterator<Item = Submit<C, &'a DeviceTracker>>,
        S: IntoIterator<Item = ash::vk::Semaphore>,
    {
        self.submit_impl(submissions, fence, |_: &DeviceTracker| ())
    }

    /// Submit commands for execution.
    /// This function accepts command buffer submissions without tracker. Tracker is still bound to the command buffer.
    pub unsafe fn submit<I, W, B, S>(
        &mut self,
        submissions: I,
        fence: Option<UnarmedFence>,
    ) -> Result<Option<ArmedFence<C>>, DeviceError>
    where
        I: IntoIterator<Item = Submission<W, B, S>>,
        W: IntoIterator<Item = (ash::vk::Semaphore, ash::vk::PipelineStageFlags)>,
        B: IntoIterator<Item = Submit<C>>,
        S: IntoIterator<Item = ash::vk::Semaphore>,
    {
        self.submit_impl(submissions, fence, |()| ())
    }

    pub(crate) unsafe fn submit_impl<I, W, B, S, T, F>(
        &mut self,
        submissions: I,
        fence: Option<UnarmedFence>,
        mut accept: F,
    ) -> Result<Option<ArmedFence<C>>, DeviceError>
    where
        I: IntoIterator<Item = Submission<W, B, S>>,
        W: IntoIterator<Item = (ash::vk::Semaphore, ash::vk::PipelineStageFlags)>,
        B: IntoIterator<Item = Submit<C, T>>,
        S: IntoIterator<Item = ash::vk::Semaphore>,
        F: FnMut(T),
    {
        let mut waits = SmallVec::<[_; 64]>::new();
        let mut stages = SmallVec::<[_; 64]>::new();
        let mut buffers = SmallVec::<[_; 256]>::new();
        let mut signals = SmallVec::<[_; 64]>::new();
        let submissions = submissions
            .into_iter()
            .map(|submission| {
                let wait_count = submission
                    .wait
                    .into_iter()
                    .fold(0, |acc, (semaphore, stage)| {
                        waits.push(semaphore);
                        stages.push(stage);
                        acc + 1
                    });
                let buffers_count = submission.submits.into_iter().fold(0, |acc, submit| {
                    assert_eq!(submit.family, self.id.family);
                    buffers.push(submit.raw);
                    accept(submit.tracker);
                    acc + 1
                });
                let signal_count = submission.signal.into_iter().fold(0, |acc, semaphore| {
                    signals.push(semaphore);
                    acc + 1
                });
                ash::vk::SubmitInfo {
                    s_type: ash::vk::StructureType::SUBMIT_INFO,
                    p_next: null(),
                    wait_semaphore_count: wait_count as u32,
                    p_wait_semaphores: waits[waits.len() - wait_count..].as_ptr(),
                    p_wait_dst_stage_mask: stages[stages.len() - wait_count..].as_ptr(),
                    command_buffer_count: buffers_count as u32,
                    p_command_buffers: buffers[buffers.len() - buffers_count..].as_ptr(),
                    signal_semaphore_count: signal_count as u32,
                    p_signal_semaphores: signals[signals.len() - signal_count..].as_ptr(),
                }
            })
            .collect::<SmallVec<[_; 64]>>();

        let mut armed_fence = None;

        let result = self.fp.queue_submit(
            self.raw,
            submissions.len() as u32,
            submissions.as_ptr(),
            match fence {
                Some(UnarmedFence { raw }) => {
                    armed_fence = Some(ArmedFence {
                        raw,
                        queue: self.id,
                        epoch: self.next,
                        relevant: Relevant,
                    });
                    raw
                }
                None => ash::vk::Fence::null(),
            },
        );

        match result {
            ash::vk::Result::SUCCESS => {
                if armed_fence.is_some() {
                    self.next += 1;
                }
                Ok(armed_fence)
            }
            error => Err(DeviceError::from_vk_result(error)),
        }
    }

    /// Convert raw queue handle to the wrapper.
    ///
    /// # Safety
    ///
    /// `family` - index of the family specified when queue handle fetched from device. That family must support specified capabilities.
    pub unsafe fn from_raw(
        fp: Arc<ash::vk::DeviceFnV1_0>,
        raw: ash::vk::Queue,
        id: QueueId<C>,
    ) -> Self {
        Queue {
            fp,
            raw,
            id,
            next: 0,
            free: LinkedList::new(),
            pending: LinkedList::new(),
        }
    }

    /// Push tracked object.
    /// Associate with next bucket.
    pub(crate) fn push_track(&mut self, objects: Arc<VulkanObjects>) {
        let bucket = Bucket {
            epoch: self.next,
            objects: objects.into(),
        };

        if self.free.is_empty() {
            self.pending.push_back(bucket)
        } else {
            *self.free.back_mut().unwrap() = bucket;
            let split = self.free.len() - 1;
            self.pending.append(&mut self.free.split_off(split));
        }
    }

    /// Called with ready fence to release resources that was possibly referenced by this queue.
    /// This fence must be armed by this queue.
    /// Otherwise this function will panic.
    pub(crate) unsafe fn complete(
        &mut self,
        device: ash::vk::Device,
        fence: ReadyFence<C>,
    ) -> UnarmedFence {
        assert_eq!(fence.queue, self.id);

        while !self.pending.is_empty() {
            if self.pending.front().unwrap().epoch <= fence.epoch {
                let tail = self.pending.split_off(1);
                self.pending.back_mut().unwrap().objects.take();
                self.free.append(&mut self.pending);
                self.pending = tail;
            } else {
                break;
            }
        }

        fence.relevant.dispose();
        UnarmedFence { raw: fence.raw }
    }
}

/// Command queue submission.
pub struct Submission<W, B, S> {
    pub wait: W,
    pub submits: B,
    pub signal: S,
}

struct Bucket {
    /// Index of the bucket.
    epoch: u64,
    /// Objects collected from `Terminal` in batch.
    objects: Option<Arc<VulkanObjects>>,
}
