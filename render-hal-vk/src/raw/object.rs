#![allow(dead_code)]
#![allow(unused_variables)]

use ash;
use relevant::Relevant;
use std::iter::FromIterator;

/// Vulkan objects that can be automatically destroyed
/// after checked for not being used anymore.
pub enum VulkanObject {
    Buffer(ash::vk::Buffer),
    BufferView(ash::vk::BufferView),
    CommandBuffer(ash::vk::CommandBuffer),
    CommandPool(ash::vk::CommandPool),
    DescriptorPool(ash::vk::DescriptorPool),
    DescriptorSet(ash::vk::DescriptorSet),
    DeviceMemory(ash::vk::DeviceMemory),
    Event(ash::vk::Event),
    Fence(ash::vk::Fence),
    Framebuffer(ash::vk::Framebuffer),
    Image(ash::vk::Image),
    ImageView(ash::vk::ImageView),
    // IndirectCommandsLayout(ash::vk::IndirectCommandsLayout),
    // ObjectTableNVX(ash::vk::ObjectTableNVX),
    Pipeline(ash::vk::Pipeline),
    RenderPass(ash::vk::RenderPass),
    Sampler(ash::vk::Sampler),
    // SamplerYcbcrConversion(ash::vk::SamplerYcbcrConversion),
    Semaphore(ash::vk::Semaphore),
    QueryPool(ash::vk::QueryPool),
}

impl From<ash::vk::Buffer> for VulkanObject {
    fn from(object: ash::vk::Buffer) -> Self {
        VulkanObject::Buffer(object)
    }
}

impl From<ash::vk::BufferView> for VulkanObject {
    fn from(object: ash::vk::BufferView) -> Self {
        VulkanObject::BufferView(object)
    }
}

impl From<ash::vk::CommandBuffer> for VulkanObject {
    fn from(object: ash::vk::CommandBuffer) -> Self {
        VulkanObject::CommandBuffer(object)
    }
}

impl From<ash::vk::CommandPool> for VulkanObject {
    fn from(object: ash::vk::CommandPool) -> Self {
        VulkanObject::CommandPool(object)
    }
}

impl From<ash::vk::DescriptorPool> for VulkanObject {
    fn from(object: ash::vk::DescriptorPool) -> Self {
        VulkanObject::DescriptorPool(object)
    }
}

impl From<ash::vk::DescriptorSet> for VulkanObject {
    fn from(object: ash::vk::DescriptorSet) -> Self {
        VulkanObject::DescriptorSet(object)
    }
}

impl From<ash::vk::DeviceMemory> for VulkanObject {
    fn from(object: ash::vk::DeviceMemory) -> Self {
        VulkanObject::DeviceMemory(object)
    }
}

impl From<ash::vk::Event> for VulkanObject {
    fn from(object: ash::vk::Event) -> Self {
        VulkanObject::Event(object)
    }
}

impl From<ash::vk::Fence> for VulkanObject {
    fn from(object: ash::vk::Fence) -> Self {
        VulkanObject::Fence(object)
    }
}

impl From<ash::vk::Framebuffer> for VulkanObject {
    fn from(object: ash::vk::Framebuffer) -> Self {
        VulkanObject::Framebuffer(object)
    }
}

impl From<ash::vk::Image> for VulkanObject {
    fn from(object: ash::vk::Image) -> Self {
        VulkanObject::Image(object)
    }
}

impl From<ash::vk::ImageView> for VulkanObject {
    fn from(object: ash::vk::ImageView) -> Self {
        VulkanObject::ImageView(object)
    }
}

// impl From<IndirectCommandsLayout(ash::vk::IndirectCommandsLayout> for VulkanObject {
//     fn from(object: IndirectCommandsLayout(ash::vk::IndirectCommandsLayout) -> Self {
//         VulkanObject::IndirectCommandsLayout(object)
//     }
// }

// impl From<ObjectTableNVX(ash::vk::ObjectTableNVX> for VulkanObject {
//     fn from(object: ObjectTableNVX(ash::vk::ObjectTableNVX) -> Self {
//         VulkanObject::ObjectTableNVX(object)
//     }
// }

impl From<ash::vk::Pipeline> for VulkanObject {
    fn from(object: ash::vk::Pipeline) -> Self {
        VulkanObject::Pipeline(object)
    }
}

impl From<ash::vk::RenderPass> for VulkanObject {
    fn from(object: ash::vk::RenderPass) -> Self {
        VulkanObject::RenderPass(object)
    }
}

impl From<ash::vk::Sampler> for VulkanObject {
    fn from(object: ash::vk::Sampler) -> Self {
        VulkanObject::Sampler(object)
    }
}

// impl From<SamplerYcbcrConversion(ash::vk::SamplerYcbcrConversion> for VulkanObject {
//     fn from(object: SamplerYcbcrConversion(ash::vk::SamplerYcbcrConversion) -> Self {
//         VulkanObject::SamplerYcbcrConversion(object)
//     }
// }

impl From<ash::vk::Semaphore> for VulkanObject {
    fn from(object: ash::vk::Semaphore) -> Self {
        VulkanObject::Semaphore(object)
    }
}

impl From<ash::vk::QueryPool> for VulkanObject {
    fn from(object: ash::vk::QueryPool) -> Self {
        VulkanObject::QueryPool(object)
    }
}

impl VulkanObject {
    pub(crate) unsafe fn destroy(self, fp: &ash::vk::DeviceFnV1_0, device: ash::vk::Device) {
        unimplemented!()
    }
}

pub(crate) struct VulkanObjects {
    objects: Vec<VulkanObject>,
    relevant: Relevant,
}

impl FromIterator<VulkanObject> for VulkanObjects {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = VulkanObject>,
    {
        VulkanObjects {
            objects: iter.into_iter().collect(),
            relevant: Relevant,
        }
    }
}

impl VulkanObjects {
    pub(crate) unsafe fn destroy(self, fp: &ash::vk::DeviceFnV1_0, device: ash::vk::Device) {
        for object in self.objects {
            object.destroy(fp, device);
        }
        self.relevant.dispose();
    }
}
