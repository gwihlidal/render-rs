use ash::{self, version::DeviceV1_0};
use relevant::Relevant;
use std::{marker::PhantomData, ptr::null, sync::Arc};

use crate::raw::command::{buffer::*, capability::*, FamilyId};
use crate::raw::errors::OomError;

/// Command pool usage type.
pub trait Usage {
    fn flags() -> ash::vk::CommandPoolCreateFlags;
}

/// Buffers from command pool can be reset individually.
pub enum ResetBuffer {}

impl Usage for ResetBuffer {
    fn flags() -> ash::vk::CommandPoolCreateFlags {
        ash::vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER
    }
}

/// Buffers from command pool cannot be reset individually.
pub enum NoResetBuffer {}

impl Usage for NoResetBuffer {
    fn flags() -> ash::vk::CommandPoolCreateFlags {
        ash::vk::CommandPoolCreateFlags::empty()
    }
}

/// Command pool marked with capability and resettability of individual buffers.
pub struct Pool<C = Capability, R = NoResetBuffer> {
    fp: Arc<ash::vk::DeviceFnV1_0>,
    device: ash::vk::Device,
    raw: ash::vk::CommandPool,
    relevant: Relevant,
    family: FamilyId<C>,
    markers: PhantomData<R>,
}

impl<C, R> Pool<C, R>
where
    C: Copy,
{
    /// Convert raw command pool handle to the wrapper.
    ///
    /// # Safety
    ///
    /// Command pool must be created with flags according to generic parameters.
    pub(crate) unsafe fn from_raw(
        fp: Arc<ash::vk::DeviceFnV1_0>,
        raw: ash::vk::CommandPool,
        device: ash::vk::Device,
        family: FamilyId<C>,
    ) -> Self {
        Pool {
            fp,
            device,
            raw,
            family,
            relevant: Relevant,
            markers: PhantomData,
        }
    }

    /// Allocate command buffers from pool.
    pub fn allocate<L: Level>(
        &mut self,
        count: u32,
    ) -> Result<impl Iterator<Item = Buffer<C, Initial, R, L>>, OomError> {
        unsafe {
            let mut buffers = Vec::with_capacity(count as usize);
            let result = self.fp.allocate_command_buffers(
                self.device,
                &ash::vk::CommandBufferAllocateInfo {
                    s_type: ash::vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
                    p_next: null(),
                    command_pool: self.raw,
                    level: L::level(),
                    command_buffer_count: count,
                },
                buffers.as_mut_ptr(),
            );

            match result {
                ash::vk::Result::SUCCESS => {
                    buffers.set_len(count as usize);
                    let fp = self.fp.clone();
                    let family = self.family;
                    Ok(buffers.into_iter().map(move |raw| {
                        // Buffer parameters should be valid.
                        Buffer::from_raw(fp.clone(), raw, family, Initial)
                    }))
                }
                ash::vk::Result::ERROR_OUT_OF_HOST_MEMORY => Err(OomError::OutOfHostMemory),
                ash::vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Err(OomError::OutOfDeviceMemory),
                _ => unreachable!(),
            }
        }
    }
}

impl<R> Pool<Capability, R> {
    /// Convert pool with runtime capability into pool with static `Transfer` capability
    /// Panics if not supported.
    pub fn transfer(self) -> Pool<Transfer, R> {
        Pool {
            fp: self.fp,
            device: self.device,
            raw: self.raw,
            relevant: self.relevant,
            markers: self.markers,
            family: self.family.transfer(),
        }
    }

    /// Convert pool with runtime capability into pool with static `Graphics` capability
    /// Panics if not supported.
    pub fn graphics(self) -> Pool<Graphics, R> {
        Pool {
            fp: self.fp,
            device: self.device,
            raw: self.raw,
            relevant: self.relevant,
            markers: self.markers,
            family: self.family.graphics(),
        }
    }

    /// Convert pool with runtime capability into pool with static `Compute` capability
    /// Panics if not supported.
    pub fn compute(self) -> Pool<Compute, R> {
        Pool {
            fp: self.fp,
            device: self.device,
            raw: self.raw,
            relevant: self.relevant,
            markers: self.markers,
            family: self.family.compute(),
        }
    }

    /// Convert pool with runtime capability into pool with static `General` capability
    /// Panics if not supported.
    pub fn general(self) -> Pool<General, R> {
        Pool {
            fp: self.fp,
            device: self.device,
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
