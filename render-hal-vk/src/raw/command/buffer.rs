#![allow(dead_code)]
#![allow(unused_variables)]

use ash;
use relevant::Relevant;
use std::{marker::PhantomData, sync::Arc};

use crate::raw::command::{
    capability::*,
    pool::{NoResetBuffer, Pool, ResetBuffer},
    FamilyId,
};
use crate::raw::errors::OomError;

pub trait Level {
    #[doc(hidden)]
    fn level() -> ash::vk::CommandBufferLevel;
}

pub trait Usage {
    #[doc(hidden)]
    fn flags() -> ash::vk::CommandBufferUsageFlags;
}

/// States that allow resetting buffer to initial state.
pub trait Resettable {}

/// Primary buffers can be submitted to queues.
pub struct Primary;

impl Level for Primary {
    fn level() -> ash::vk::CommandBufferLevel {
        ash::vk::CommandBufferLevel::PRIMARY
    }
}

/// Secondary buffers can be executed in primary buffers.
pub struct Secondary;

impl Level for Secondary {
    fn level() -> ash::vk::CommandBufferLevel {
        ash::vk::CommandBufferLevel::SECONDARY
    }
}

/// Buffer will be invalidated after completion.
pub struct OneTimeUse;

impl Usage for OneTimeUse {
    fn flags() -> ash::vk::CommandBufferUsageFlags {
        ash::vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT
    }
}

/// Buffer can be reused after completion.
pub struct MultiUse<S = ()>(S);

impl Usage for MultiUse {
    fn flags() -> ash::vk::CommandBufferUsageFlags {
        ash::vk::CommandBufferUsageFlags::empty()
    }
}

/// Buffer can be reused during execution.
pub struct Simultaneous;

impl Usage for MultiUse<Simultaneous> {
    fn flags() -> ash::vk::CommandBufferUsageFlags {
        ash::vk::CommandBufferUsageFlags::SIMULTANEOUS_USE
    }
}

/// Buffer is in initial state after creating or resetting.
pub struct Initial;

/// After begin buffer moves to recording state.
/// Buffer in recording state can accept commands to record.
pub struct Recording<U, T>(U, T);

/// After finish buffer moves to executable state.
/// Executable buffer can be submitted.
pub struct Executable<U, T>(U, T);

/// After submission buffer is in pending state.
/// Device can read it and execute commands recorded.
/// Unless buffer has `SimultaneousUse` flag it can't be resubmitted.
/// After completion buffer moves to executable state (or invalid state if it is `OneTimeUse` buffer).
pub struct Pending<N>(N);

/// State it which buffer is moved when any internal data of it is invalidated.
pub struct Invalid;

impl Resettable for Initial {}
impl<U, T> Resettable for Recording<U, T> {}
impl<U, T> Resettable for Executable<U, T> {}
impl Resettable for Invalid {}

/// Vulkan's command buffer marked with capability, state, resettablility and level.
pub struct Buffer<C, S, R = NoResetBuffer, L = Primary> {
    fp: Arc<ash::vk::DeviceFnV1_0>,
    raw: ash::vk::CommandBuffer,
    family: FamilyId<C>,
    state: S,
    relevant: Relevant,
    markers: PhantomData<(R, L)>,
}

impl<C, R, L> Buffer<C, Initial, R, L>
where
    L: Level,
{
    /// Start buffer recording.
    /// This function accepts tracker that will track referenced resource to prevent their destruction.
    pub(crate) unsafe fn begin<U: Usage, T>(
        self,
        tracker: T,
    ) -> Result<Buffer<C, Recording<U, T>, R, L>, OomError> {
        unimplemented!()
    }
}

impl<C, U, T, R, L> Buffer<C, Recording<U, T>, R, L> {
    /// Finish buffer recording.
    pub(crate) unsafe fn finish(self) -> Result<Buffer<C, Executable<U, T>, R, L>, OomError> {
        unimplemented!()
    }
}

/// Command buffer usable to submit into command queue.
pub struct Submit<C, T = ()> {
    pub(crate) raw: ash::vk::CommandBuffer,
    pub(crate) family: FamilyId<C>,
    pub(crate) tracker: T,
}

impl<C, U, T, R> Buffer<C, Executable<U, T>, R, Primary> {
    /// Prepare command buffer for submission.
    /// Buffer will be moved to invalid state after completion.
    /// This transfers resource tracker from buffer's state to `Submit`.
    pub(crate) unsafe fn submit_once(
        self,
    ) -> (Submit<C, T>, Buffer<C, Pending<Invalid>, R, Primary>) {
        unimplemented!()
    }
}

impl<C, S, T, R> Buffer<C, Executable<MultiUse<S>, T>, R, Primary> {
    /// Prepare command buffer for submission.
    /// Buffer will be moved back to executable state after completion.
    /// `Submit` doesn't receive resource tracker.
    pub(crate) unsafe fn submit(
        self,
    ) -> (
        Submit<C>,
        Buffer<C, Pending<Executable<MultiUse<S>, T>>, R, Primary>,
    ) {
        unimplemented!()
    }
}

impl<C, N, R, L> Buffer<C, Pending<N>, R, L> {
    /// Move buffer from pending state.
    /// This function is marked unsafe since user is responsible to check that
    /// buffer is not used by device.
    pub(crate) unsafe fn complete(self) -> Buffer<C, N, R, L> {
        unimplemented!()
    }
}

impl<C, S, L> Buffer<C, S, ResetBuffer, L>
where
    S: Resettable,
{
    /// Reset command buffer.
    pub(crate) unsafe fn reset(self) -> Buffer<C, Initial, ResetBuffer, L> {
        unimplemented!()
    }
}

impl<C, S, R, L> Buffer<C, S, R, L>
where
    S: Resettable,
{
    /// Free command buffer.
    pub(crate) unsafe fn free(self, pool: &mut Pool<C, R>) {
        unimplemented!()
    }
}

impl<C, S, R, L> Buffer<C, S, R, L> {
    /// Mark command buffer as resetted.
    /// Should called only after resetting command pool.
    pub(crate) unsafe fn resetted(self) -> Buffer<C, Initial, R, L> {
        unimplemented!()
    }

    /// Convert raw buffer handle to the wrapper.
    ///
    /// # Safety
    ///
    /// `pool`   - command pool from which this Buffer was allocated.
    /// `raw`    - raw command buffer allocated from `pool`. It must be in initial state.
    /// Command buffer must be created with flags according to type parameters.
    pub(crate) unsafe fn from_raw(
        fp: Arc<ash::vk::DeviceFnV1_0>,
        raw: ash::vk::CommandBuffer,
        family: FamilyId<C>,
        state: S,
    ) -> Self {
        Buffer {
            fp,
            raw,
            family,
            state,
            relevant: Relevant,
            markers: PhantomData,
        }
    }
}

impl<S, R, L> Buffer<Capability, S, R, L> {
    /// Convert buffer with runtime capability into buffer with static `Transfer` capability
    /// Panics if not supported.
    pub fn transfer(self) -> Buffer<Transfer, S, R, L> {
        Buffer {
            fp: self.fp,
            raw: self.raw,
            state: self.state,
            relevant: self.relevant,
            markers: self.markers,
            family: self.family.transfer(),
        }
    }

    /// Convert buffer with runtime capability into buffer with static `Transfer` capability
    /// Panics if not supported.
    pub fn graphics(self) -> Buffer<Graphics, S, R, L> {
        Buffer {
            fp: self.fp,
            state: self.state,
            raw: self.raw,
            relevant: self.relevant,
            markers: self.markers,
            family: self.family.graphics(),
        }
    }

    /// Convert buffer with runtime capability into buffer with static `Transfer` capability
    /// Panics if not supported.
    pub fn compute(self) -> Buffer<Compute, S, R, L> {
        Buffer {
            fp: self.fp,
            state: self.state,
            raw: self.raw,
            relevant: self.relevant,
            markers: self.markers,
            family: self.family.compute(),
        }
    }

    /// Convert buffer with runtime capability into buffer with static `Transfer` capability
    /// Panics if not supported.
    pub fn general(self) -> Buffer<General, S, R, L> {
        Buffer {
            fp: self.fp,
            state: self.state,
            raw: self.raw,
            relevant: self.relevant,
            markers: self.markers,
            family: self.family.general(),
        }
    }

    /// Get capability level of the family
    pub fn capability(&self) -> Capability {
        self.family.capability()
    }
}
