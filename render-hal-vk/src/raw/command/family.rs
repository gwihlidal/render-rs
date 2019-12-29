use ash::{self, version::DeviceV1_0};
use relevant::Relevant;
use std::{fmt::Debug, iter::FromIterator, marker::PhantomData, ptr::null, sync::Arc};

use crate::raw::command::{
    capability::Capability,
    pool::{self, Pool, Usage},
    queue::Queue,
    FamilyId, QueueId,
};
use crate::raw::errors::OomError;

/// Collection of all queues from one family
/// Family marked with capability level.
/// Runtime value `Capability` by default.
pub struct Family<C = Capability> {
    fp: Arc<ash::vk::DeviceFnV1_0>,
    device: ash::vk::Device,
    id: FamilyId<C>,
    queues: Vec<Queue<C>>,
}

impl<C> Family<C>
where
    C: Copy + Debug,
{
    /// Fetch queue family from device.
    ///
    /// # Safety
    ///
    /// `id`   - must be one of the queue family ids specified when device was created.
    /// `count` - must be less than or equal to number of queues created for the specified queue family index when device was created.
    pub(crate) unsafe fn from_device(
        fp: Arc<ash::vk::DeviceFnV1_0>,
        device: ash::vk::Device,
        id: FamilyId<C>,
        count: u32,
    ) -> Self {
        Family {
            id,
            queues: (0..count)
                .map(|queue_index| {
                    let mut queue = ash::vk::Queue::null();
                    fp.get_device_queue(device, id.index, queue_index, &mut queue);
                    Queue::from_raw(
                        fp.clone(),
                        queue,
                        QueueId {
                            family: id,
                            index: queue_index,
                        },
                    )
                    .into()
                })
                .collect(),
            fp,
            device,
        }
    }

    /// Create command pool associated with the queue family.
    pub(crate) unsafe fn create_pool<R: Usage>(&self) -> Result<Pool<C, R>, OomError> {
        // Must be valid according to https://www.khronos.org/registry/vulkan/specs/1.1-extensions/html/vkspec.html#vkCreateCommandPool
        let mut raw = ash::vk::CommandPool::null();
        let result = self.fp.create_command_pool(
            self.device,
            &ash::vk::CommandPoolCreateInfo {
                s_type: ash::vk::StructureType::COMMAND_POOL_CREATE_INFO,
                p_next: null(),
                flags: R::flags(),
                queue_family_index: self.id.index,
            },
            null(),
            &mut raw,
        );

        match result {
            ash::vk::Result::SUCCESS => {
                Ok(Pool::from_raw(self.fp.clone(), raw, self.device, self.id))
            }
            ash::vk::Result::ERROR_OUT_OF_HOST_MEMORY => Err(OomError::OutOfHostMemory),
            ash::vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Err(OomError::OutOfDeviceMemory),
            _ => unreachable!(),
        }
    }

    /// Fetch queue family from device.
    ///
    /// # Safety
    ///
    /// `index` - must be one of the queue family indices specified when device was created. That family must support specified capabilities.
    /// `queues` - actual family queues collection.
    pub(crate) unsafe fn from_raw(
        fp: Arc<ash::vk::DeviceFnV1_0>,
        device: ash::vk::Device,
        id: FamilyId<C>,
        queues: Vec<Queue<C>>,
    ) -> Self {
        Family {
            id,
            queues,
            fp,
            device,
        }
    }

    /// Get queues.
    pub fn queues(&mut self) -> &mut [Queue<C>] {
        &mut self.queues
    }
}
