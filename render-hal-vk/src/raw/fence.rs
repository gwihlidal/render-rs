#![allow(dead_code)]

use crate::raw::device::Device;
use crate::raw::escape::Escape;
use crate::raw::memory;
use ash;
use ash::version::DeviceV1_0;
use std::collections::VecDeque;
use std::{
    cmp, fmt,
    ops::Range,
    ptr,
    sync::{Arc, RwLock},
};

const WAIT_THRESHOLD: usize = 20;
const WAIT_COUNT: usize = 10;
const WAIT_STAGES: [ash::vk::PipelineStageFlags; WAIT_COUNT] = [
    ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
    ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
    ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
    ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
    ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
    ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
    ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
    ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
    ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
    ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
];

trait PoolType {
    fn create_type(device: Arc<Device>) -> Self;
    fn reset(&self);
}

struct FenceObject {
    device: Arc<Device>,
    fence: ash::vk::Fence,
}

impl FenceObject {
    pub fn is_complete(&self) -> bool {
        let status = unsafe { self.device.device().get_fence_status(self.fence) };
        status.is_ok()
    }
}

impl PoolType for FenceObject {
    fn create_type(device: Arc<Device>) -> FenceObject {
        let create_info = ash::vk::FenceCreateInfo {
            s_type: ash::vk::StructureType::FENCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: ash::vk::FenceCreateFlags::empty(),
        };

        let fence = unsafe {
            device
                .device()
                .create_fence(&create_info, None)
                .expect("Create fence failed.")
        };

        FenceObject { device, fence }
    }

    fn reset(&self) {
        unsafe {
            self.device.device().reset_fences(&[self.fence]).unwrap();
        }
    }
}

impl Drop for FenceObject {
    fn drop(&mut self) {
        unsafe {
            self.device.device().destroy_fence(self.fence, None);
        }
    }
}

struct SemaphoreObject {
    device: Arc<Device>,
    semaphore: ash::vk::Semaphore,
}

impl PoolType for SemaphoreObject {
    fn create_type(device: Arc<Device>) -> SemaphoreObject {
        let create_info = ash::vk::SemaphoreCreateInfo {
            s_type: ash::vk::StructureType::SEMAPHORE_CREATE_INFO,
            p_next: ptr::null(),
            flags: ash::vk::SemaphoreCreateFlags::empty(),
        };

        let semaphore = unsafe {
            device
                .device()
                .create_semaphore(&create_info, None)
                .expect("Create semaphore failed.")
        };

        SemaphoreObject { device, semaphore }
    }

    fn reset(&self) {}
}

impl Drop for SemaphoreObject {
    fn drop(&mut self) {
        unsafe {
            self.device.device().destroy_semaphore(self.semaphore, None);
        }
    }
}

struct ObjectPool<T: PoolType> {
    used_objects: VecDeque<Arc<T>>,
    free_objects: VecDeque<Arc<T>>,
    device: Arc<Device>,
}

impl<T: PoolType> ObjectPool<T> {
    fn new(device: Arc<Device>) -> Self {
        ObjectPool {
            used_objects: VecDeque::with_capacity(16),
            free_objects: VecDeque::with_capacity(16),
            device,
        }
    }

    fn get(&mut self) -> Arc<T> {
        if let Some(object) = self.free_objects.pop_back() {
            object.reset();
            self.used_objects.push_back(Arc::clone(&object));
            object
        } else {
            let object = Arc::new(T::create_type(Arc::clone(&self.device)));
            self.used_objects.push_back(Arc::clone(&object));
            object
        }
    }

    fn pop_all(&mut self) {
        self.free_objects.append(&mut self.used_objects);
        assert!(self.used_objects.len() == 0); // TEMP, just to check...
    }

    fn pop_front(&mut self) {
        let object = self.used_objects.front();
        if let Some(object) = object {
            self.free_objects.push_back(object.clone());
        }
    }

    fn front(&self) -> Option<Arc<T>> {
        self.used_objects.front().map(|object| object.clone())
    }

    fn has_used(&self) -> bool {
        self.used_objects.len() > 0
    }

    fn get_used(&self) -> &VecDeque<Arc<T>> {
        &self.used_objects
    }

    fn get_free(&self) -> &VecDeque<Arc<T>> {
        &self.free_objects
    }
}

type FencePool = ObjectPool<FenceObject>;
type SemaphorePool = ObjectPool<SemaphoreObject>;

pub struct Fence {
    semaphores: SemaphorePool,
    fences: FencePool,
    wait_list: Vec<ash::vk::Semaphore>,
    device: Arc<Device>,
    gpu_value: u64,
    cpu_value: u64,
}

impl fmt::Debug for Fence {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TODO: Fence")
    }
}

impl Fence {
    pub fn new(device: Arc<Device>) -> Self {
        Fence {
            semaphores: SemaphorePool::new(device.clone()),
            fences: FencePool::new(device.clone()),
            wait_list: Vec::with_capacity(16),
            device,
            gpu_value: 0,
            cpu_value: 1,
        }
    }

    pub fn gpu_value(&mut self) -> u64 {
        while self.fences.has_used() {
            let active_fence = self.fences.front().unwrap();
            if active_fence.is_complete() {
                self.fences.pop_front();
                self.gpu_value += 1;
            } else {
                break;
            }
        }

        self.release_semaphores();
        self.gpu_value
    }

    pub fn cpu_value(&self) -> u64 {
        self.cpu_value
    }

    pub fn sync_gpu(&mut self, queue: Arc<RwLock<ash::vk::Queue>>) {
        if self.wait_list.len() > 0 {
            let wait_stages: Vec<ash::vk::PipelineStageFlags> =
                vec![ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT; self.wait_list.len()];
            let submit_info = ash::vk::SubmitInfo {
                s_type: ash::vk::StructureType::SUBMIT_INFO,
                p_next: ptr::null(),
                wait_semaphore_count: self.wait_list.len() as u32,
                p_wait_semaphores: self.wait_list.as_ptr(),
                p_wait_dst_stage_mask: wait_stages.as_ptr(),
                command_buffer_count: 0,
                p_command_buffers: ptr::null(),
                signal_semaphore_count: 0,
                p_signal_semaphores: ptr::null(),
            };

            let queue_write = queue.write().unwrap();
            unsafe {
                self.device
                    .device()
                    .queue_submit(*queue_write, &[submit_info], ash::vk::Fence::null())
                    .expect("sync gpu failed.");
            }
        }
    }

    pub fn sync_cpu(&mut self) {
        if self.fences.has_used() {
            let active_fences = self.fences.get_used();
            let fence_objects = active_fences
                .iter()
                .map(|object| object.fence)
                .collect::<Vec<ash::vk::Fence>>();
            unsafe {
                self.device
                    .device()
                    .wait_for_fences(&fence_objects, true /* wait all */, u64::max_value())
                    .expect("wait for fences failed.");
            }
        }
    }

    pub fn signal_gpu(&mut self, queue: Arc<RwLock<ash::vk::Queue>>) -> u64 {
        self.cpu_value += 1;
        let fence = self.fences.get();
        let semaphore = self.semaphores.get();
        self.wait_list.push(semaphore.semaphore);

        let mut submit_info = ash::vk::SubmitInfo {
            s_type: ash::vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: 0,
            p_wait_semaphores: ptr::null(),
            p_wait_dst_stage_mask: ptr::null(),
            command_buffer_count: 0,
            p_command_buffers: ptr::null(),
            signal_semaphore_count: 1,
            p_signal_semaphores: &semaphore.semaphore,
        };

        if self.wait_list.len() > WAIT_THRESHOLD {
            submit_info.wait_semaphore_count = WAIT_COUNT as u32;
            submit_info.p_wait_semaphores = self.wait_list.as_ptr();
            submit_info.p_wait_dst_stage_mask = WAIT_STAGES.as_ptr();
        }

        let queue_write = queue.write().unwrap();
        unsafe {
            self.device
                .device()
                .queue_submit(*queue_write, &[submit_info], fence.fence)
                .expect("queue submit failed.");
        }

        if self.wait_list.len() > WAIT_THRESHOLD {
            self.wait_list.drain(0..WAIT_THRESHOLD);
        }

        self.cpu_value() - 1
    }

    fn release_semaphores(&mut self) {
        let semaphore_count = self.semaphores.get_used().len();
        let fence_count = self.fences.get_used().len();
        assert!(fence_count <= semaphore_count);

        let fence_delta = semaphore_count - fence_count;
        let wait_count = self.wait_list.len();
        assert!(wait_count <= semaphore_count);

        let wait_delta = semaphore_count - wait_count;
        let pop_count = cmp::min(wait_delta, fence_delta);
        for _ in 0..pop_count {
            self.semaphores.pop_front();
        }
    }
}
