#![allow(dead_code)]

use crate::descriptors::CachedDescriptorSet;
use crate::device::{RenderDeviceFrames, RenderDeviceVk};
use crate::raw::device::Device as RawDevice;
use crate::raw::fence::Fence;
use crate::raw::format::{bool_to_vk, get_compare_op};
use crate::raw::swap_chain::SwapChain;
use crate::shader_views::{ShaderResourceViewBinding, UnorderedAccessViewBinding};
use ash;
use ash::version::DeviceV1_0;
use num_traits::clamp;
use render_core::constants::*;
use render_core::error::{Error, Result};
use render_core::resources::RenderResourceBase;
use render_core::state::*;
use render_core::types::*;

use std::{
    borrow::Cow,
    fmt, ptr,
    sync::{Arc, RwLock},
};

#[derive(Clone, Debug)]
pub struct RenderShaderVk {
    pub name: Cow<'static, str>,
    pub module: ash::vk::ShaderModule,
    pub set_layouts: Vec<(
        u32, /* set index */
        Vec<ash::vk::DescriptorSetLayoutBinding>,
    )>,
    //pub byte_code: Vec<u8>,
    pub entry_point: std::ffi::CString,
}

impl RenderResourceBase for RenderShaderVk {
    #[inline]
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::Shader
    }

    #[inline]
    fn get_name(&self) -> &str {
        &self.name
    }
}

pub struct RenderSwapChainVk {
    pub name: Cow<'static, str>,
    pub swap_chain: ash::vk::SwapchainKHR,
    pub swap_chain_info: ash::vk::SwapchainCreateInfoKHR,
    pub surface: ash::vk::SurfaceKHR,
    pub textures: Vec<RenderTextureVk>,
    pub back_buffer_index: u32,
    pub acquire_image_semaphore: ash::vk::Semaphore,
    pub render_done_semaphore: ash::vk::Semaphore,
}

impl RenderResourceBase for RenderSwapChainVk {
    #[inline]
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::SwapChain
    }

    #[inline]
    fn get_name(&self) -> &str {
        &self.name
    }
}

impl fmt::Debug for RenderSwapChainVk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TODO: RenderSwapChainVk - {}", self.get_name())
    }
}

#[derive(Clone, Debug)]
pub struct RenderBufferVk {
    pub name: Cow<'static, str>,
    pub desc: RenderBufferDesc,
    pub buffer: ash::vk::Buffer,
    pub allocation: vk_mem::Allocation,
    pub supported_states: RenderResourceStates,
    pub default_state: RenderResourceStates,
}

impl RenderResourceBase for RenderBufferVk {
    #[inline]
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::Buffer
    }

    #[inline]
    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct RenderTextureVk {
    pub name: Cow<'static, str>,
    pub desc: RenderTextureDesc,
    pub image: ash::vk::Image,
    pub allocation: Option<vk_mem::Allocation>,

    pub supported_states: RenderResourceStates,
    pub default_state: RenderResourceStates,
}

impl RenderResourceBase for RenderTextureVk {
    #[inline]
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::Texture
    }

    #[inline]
    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderFenceVk {
    pub name: Cow<'static, str>,
    pub fence: ash::vk::Fence,
}

impl RenderResourceBase for RenderFenceVk {
    #[inline]
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::Fence
    }

    #[inline]
    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct RenderSamplerStateVk {
    pub name: Cow<'static, str>,
    pub sampler: ash::vk::Sampler,
}

impl RenderResourceBase for RenderSamplerStateVk {
    #[inline]
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::SamplerState
    }

    #[inline]
    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Default, Debug)]
pub struct DescriptorSetLayout {
    pub set_index: u32,
    pub bindings: Vec<ash::vk::DescriptorSetLayoutBinding>,
    pub layout: ash::vk::DescriptorSetLayout,
}

#[derive(Clone, Default, Debug)]
pub struct RenderPipelineLayoutVk {
    pub static_samplers: Vec<ash::vk::Sampler>,
    pub pipeline_layout: ash::vk::PipelineLayout,
    pub combined_layouts: Vec<DescriptorSetLayout>,
    pub sampler_layouts: Vec<ash::vk::DescriptorSetLayoutBinding>,
    pub pool_sizes: Vec<ash::vk::DescriptorPoolSize>,
}

#[derive(Clone, Default, Debug)]
pub struct RenderGraphicsPipelineStateVk {
    pub name: Cow<'static, str>,
    pub data: RenderPipelineLayoutVk,
    pub pipeline: ash::vk::Pipeline,
}

impl RenderResourceBase for RenderGraphicsPipelineStateVk {
    #[inline]
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::GraphicsPipelineState
    }

    #[inline]
    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct RenderComputePipelineStateVk {
    pub name: Cow<'static, str>,
    pub data: RenderPipelineLayoutVk,
    pub pipeline: ash::vk::Pipeline,
}

impl RenderResourceBase for RenderComputePipelineStateVk {
    #[inline]
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::ComputePipelineState
    }

    #[inline]
    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct RenderShaderViewsVk {
    pub name: Cow<'static, str>,
    pub srvs: Vec<ShaderResourceViewBinding>,
    pub uavs: Vec<UnorderedAccessViewBinding>,

    // Lazy-initialized on first bind
    pub cached_descriptor_sets: Vec<CachedDescriptorSet>,
}

impl RenderResourceBase for RenderShaderViewsVk {
    #[inline]
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::ShaderViews
    }

    #[inline]
    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct RenderDrawBindingSetVk {
    pub name: Cow<'static, str>,
    pub desc: RenderDrawBindingSetDesc,
    // TODO: Optimize lookups
    //pub vertex_buffers: [Option<Arc<RenderBufferVk>>; MAX_VERTEX_STREAMS],
    pub vertex_buffers: [Option<RenderResourceHandle>; MAX_VERTEX_STREAMS],
    pub vertex_buffer_offsets: [ash::vk::DeviceSize; MAX_VERTEX_STREAMS],
    // TODO: Optimize lookups
    //pub index_buffer: Option<Arc<RenderBufferVk>>,
    pub index_buffer: Option<RenderResourceHandle>,
    pub index_buffer_offset: ash::vk::DeviceSize,
    pub index_buffer_format: ash::vk::IndexType,
}

impl RenderResourceBase for RenderDrawBindingSetVk {
    #[inline]
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::DrawBindingSet
    }

    #[inline]
    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct RenderFrameBindingSetVk {
    pub name: Cow<'static, str>,
    pub desc: RenderFrameBindingSetDesc,
    pub render_target_handles: [Option<RenderResourceHandle>; MAX_RENDER_TARGET_COUNT],
    pub render_target_resources:
        [Option<Arc<RwLock<Box<dyn RenderResourceBase>>>>; MAX_RENDER_TARGET_COUNT], // TODO: Use explicit RenderTextureVk?
    pub depth_stencil_handle: Option<RenderResourceHandle>,
    pub depth_stencil_resource: Option<Arc<RwLock<Box<dyn RenderResourceBase>>>>, // TODO: Use explicit RenderTextureVk?
    pub image_views: Vec<ash::vk::ImageView>, // color views + depth views at the end
    pub swap_chain: Option<Arc<RwLock<Box<dyn RenderResourceBase>>>>, // TODO: Use explicit RenderSwapChainVk?
    pub frame_buffer_info: ash::vk::FramebufferCreateInfo,
    pub render_target_count: u32,
}

impl RenderResourceBase for RenderFrameBindingSetVk {
    #[inline]
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::FrameBindingSet
    }

    #[inline]
    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone)]
pub struct RenderPassVk {
    pub name: Cow<'static, str>,
    pub desc: RenderPassDesc,
    pub render_pass: ash::vk::RenderPass,
    pub frame_buffer: ash::vk::Framebuffer,
    pub frame_binding: Arc<RwLock<Box<dyn RenderResourceBase>>>, // TODO: Use explicit RenderFrameBindingSetVk?
    pub depth_stencil_layout: ash::vk::ImageLayout,
    pub clear_values: Vec<ash::vk::ClearValue>,
}

impl fmt::Debug for RenderPassVk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TODO: RenderPassVk")
    }
}

impl RenderResourceBase for RenderPassVk {
    #[inline]
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::RenderPass
    }

    #[inline]
    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct RenderRayTracingPipelineStateVk {
    pub name: Cow<'static, str>,
}

impl RenderResourceBase for RenderRayTracingPipelineStateVk {
    #[inline]
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::RayTracingPipelineState
    }

    #[inline]
    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct RenderRayTracingProgramVk {
    pub name: Cow<'static, str>,
}

impl RenderResourceBase for RenderRayTracingProgramVk {
    #[inline]
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::RayTracingProgram
    }

    #[inline]
    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct RenderRayTracingGeometryVk {
    pub name: Cow<'static, str>,
}

impl RenderResourceBase for RenderRayTracingGeometryVk {
    #[inline]
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::RayTracingGeometry
    }

    #[inline]
    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct RenderRayTracingAccelerationVk {
    pub name: Cow<'static, str>,
}

impl RenderResourceBase for RenderRayTracingAccelerationVk {
    #[inline]
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::RayTracingAcceleration
    }

    #[inline]
    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct RenderRayTracingShaderTableVk {
    pub name: Cow<'static, str>,
}

impl RenderResourceBase for RenderRayTracingShaderTableVk {
    #[inline]
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::RayTracingShaderTable
    }

    #[inline]
    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct RenderTimingHeapVk {
    pub name: Cow<'static, str>,
    pub desc: RenderTimingHeapDesc,

    pub current_buffer: u32,
    pub previous_buffer: u32,
}

impl RenderResourceBase for RenderTimingHeapVk {
    #[inline]
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::TimingHeap
    }

    #[inline]
    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug)]
pub struct CommandBuffer {
    pub device: Arc<RawDevice>,
    pub command_pool: Arc<ash::vk::CommandPool>,
    pub command_buffer: Arc<ash::vk::CommandBuffer>,
    pub submit_fence: Fence,
    //pub submit_fence: Arc<Fence>,
    pub submit_value: u64,
    pub opened: bool,
}

impl CommandBuffer {
    pub fn new(device: Arc<RawDevice>, command_pool: Arc<ash::vk::CommandPool>) -> Self {
        let allocate_info = ash::vk::CommandBufferAllocateInfo {
            s_type: ash::vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_buffer_count: 1,
            command_pool: *command_pool,
            level: ash::vk::CommandBufferLevel::PRIMARY,
        };

        let command_buffers = unsafe {
            device
                .device()
                .allocate_command_buffers(&allocate_info)
                .unwrap()
        };

        let command_buffer = Arc::new(command_buffers[0]);

        CommandBuffer {
            device: device.clone(),
            command_pool,
            command_buffer,
            submit_fence: Fence::new(device),
            //submit_fence: Arc::new(Fence::new(device)),
            submit_value: 0,
            opened: false,
        }
    }

    pub fn open(&mut self) {
        assert!(!self.is_open());
        assert!(!self.in_flight());
        self.opened = true;

        let begin_info = ash::vk::CommandBufferBeginInfo {
            s_type: ash::vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            p_inheritance_info: ptr::null(),
            flags: ash::vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
        };

        unsafe {
            self.device
                .device()
                .begin_command_buffer(*self.command_buffer, &begin_info)
                .unwrap();
        }
    }

    pub fn close(&mut self) {
        assert!(self.is_open());
        self.opened = false;
        unsafe {
            self.device
                .device()
                .end_command_buffer(*self.command_buffer)
                .unwrap();
        }
    }

    pub fn submit(&mut self, queue: Arc<RwLock<ash::vk::Queue>>) {
        assert!(!self.is_open());
        assert!(!self.in_flight());

        let wait_stage = ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
        let submit_info = ash::vk::SubmitInfo {
            s_type: ash::vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: 0,
            p_wait_semaphores: ptr::null(),
            p_wait_dst_stage_mask: &wait_stage,
            command_buffer_count: 1,
            p_command_buffers: &*self.command_buffer,
            signal_semaphore_count: 0,
            p_signal_semaphores: ptr::null(),
        };

        unsafe {
            let queue_write = queue.write().unwrap();
            self.device
                .device()
                .queue_submit(*queue_write, &[submit_info], ash::vk::Fence::null())
                .expect("sync gpu failed.");
        }

        self.submit_fence.signal_gpu(queue);
        self.submit_value = self.submit_fence.cpu_value();
        self.submit_fence.sync_cpu();
    }

    #[inline]
    pub fn get(&self) -> Arc<ash::vk::CommandBuffer> {
        Arc::clone(&self.command_buffer)
    }

    //#[inline]
    //pub fn fence(&self) -> Arc<Fence> {
    //	Arc::clone(&self.fence)
    //}

    #[inline]
    pub fn is_open(&self) -> bool {
        self.opened
    }

    pub fn in_flight(&mut self) -> bool {
        let gpu_value = self.submit_fence.gpu_value();
        self.submit_value > gpu_value
    }
}

impl Drop for CommandBuffer {
    fn drop(&mut self) {
        assert!(!self.is_open());
        assert!(!self.in_flight());
        unsafe {
            self.device
                .device()
                .free_command_buffers(*self.command_pool, &[*self.command_buffer]);
        }
    }
}

pub struct CommandBufferPool {
    pub device: Arc<RawDevice>,
    pub command_pool: Arc<ash::vk::CommandPool>,
    pub command_buffers: RwLock<Vec<Arc<RwLock<CommandBuffer>>>>,
    pub usage_count: usize,
}

impl CommandBufferPool {
    pub fn new(device: Arc<RawDevice>, queue_family_index: u32) -> Self {
        // NOTE: We reset the command pool entirely, so no need for individual command buffer resets
        let create_info = ash::vk::CommandPoolCreateInfo {
            s_type: ash::vk::StructureType::COMMAND_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags: ash::vk::CommandPoolCreateFlags::TRANSIENT,
            queue_family_index,
        };

        let pool = unsafe {
            device
                .device()
                .create_command_pool(&create_info, None)
                .unwrap()
        };
        CommandBufferPool {
            device,
            command_pool: Arc::new(pool),
            command_buffers: RwLock::new(Vec::new()),
            usage_count: 0,
        }
    }

    pub fn allocate(&mut self) -> Arc<RwLock<CommandBuffer>> {
        let mut command_buffers = self.command_buffers.write().unwrap();
        if self.usage_count >= command_buffers.len() {
            command_buffers.push(Arc::new(RwLock::new(CommandBuffer::new(
                self.device.clone(),
                self.command_pool.clone(),
            ))));
        }

        assert!(self.usage_count < command_buffers.len());
        let slot = self.usage_count;
        self.usage_count += 1;
        command_buffers[slot].clone()
    }

    pub fn reset(&mut self) {
        let _ = self.command_buffers.write().unwrap();

        // Command buffers are all reset implicitly by resetting the pool
        unsafe {
            self.device
                .device()
                .reset_command_pool(*self.command_pool, ash::vk::CommandPoolResetFlags::empty())
                .unwrap();
        }

        self.usage_count = 0;
    }
}

impl Drop for CommandBufferPool {
    fn drop(&mut self) {
        // Command buffers are implicitly freed by destroying the pool
        self.command_buffers.write().unwrap().clear();
        unsafe {
            self.device
                .device()
                .destroy_command_pool(*self.command_pool, None);
        }
    }
}

#[derive(Clone)]
pub struct RenderCommandListVk {
    pub name: Cow<'static, str>,
    pub raw_device: Arc<RawDevice>,
    pub frames: Arc<RwLock<RenderDeviceFrames>>,
    pub list_type: RenderCommandListType,
    pub command_buffer: Option<Arc<RwLock<CommandBuffer>>>,
}

impl fmt::Debug for RenderCommandListVk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TODO: RenderCommandListVk")
    }
}

impl Drop for RenderCommandListVk {
    fn drop(&mut self) {
        trace!("Command list drop");
        // Command list cannot be open during destruction
        assert!(!self.is_open());
    }
}

impl RenderCommandListVk {
    pub fn new(
        raw_device: Arc<RawDevice>,
        frames: Arc<RwLock<RenderDeviceFrames>>,
        list_type: RenderCommandListType,
        debug_name: Cow<'static, str>,
    ) -> Self {
        assert_ne!(list_type, RenderCommandListType::Invalid);
        RenderCommandListVk {
            raw_device,
            frames,
            list_type,
            name: debug_name,
            command_buffer: None,
        }
    }

    pub fn get(&mut self) -> Result<Arc<ash::vk::CommandBuffer>> {
        if self.command_buffer.is_none() {
            assert!(!self.is_open());
            match self.list_type {
                RenderCommandListType::Universal => {
                    let frames_lock = self.frames.clone();
                    let frames = frames_lock.write().unwrap();
                    let pool_lock = &*frames.frames[frames.frame_index].universal_pool.clone();
                    let mut pool = pool_lock.write().unwrap();
                    self.command_buffer = Some(pool.allocate());
                }
                _ => unimplemented!(),
            }
        }

        if let Some(ref command_buffer) = self.command_buffer {
            Ok(Arc::clone(&command_buffer.read().unwrap().get()))
        } else {
            Err(Error::backend("no command buffer bound"))
        }
    }

    pub fn open(&mut self) -> Result<Arc<ash::vk::CommandBuffer>> {
        assert!(!self.is_open());
        assert!(self.command_buffer.is_none()); // Command lists can only be recorded once before a submit
        let vk_command_buffer = self.get()?;
        assert!(self.command_buffer.is_some());
        if let Some(ref command_buffer) = self.command_buffer {
            let mut command_buffer = command_buffer.write().unwrap();
            command_buffer.open();
            Ok(vk_command_buffer)
        } else {
            Err(Error::backend("no command buffer bound"))
        }
    }

    pub fn close(&mut self) -> Result<()> {
        assert!(self.is_open());
        assert!(self.command_buffer.is_some());
        if let Some(ref command_buffer) = self.command_buffer {
            let mut command_buffer = command_buffer.write().unwrap();
            command_buffer.close();
            Ok(())
        } else {
            Err(Error::backend("no command buffer bound"))
        }
    }

    pub fn submit(
        &mut self,
        queue: Arc<RwLock<ash::vk::Queue>>,
        wait_for: &[ash::vk::Semaphore],
        signal_after: &[ash::vk::Semaphore],
        fence: Option<ash::vk::Fence>,
        wait_dst_stage_mask: ash::vk::PipelineStageFlags,
    ) -> Result<()> {
        assert!(!self.is_open());
        assert!(self.command_buffer.is_some());
        match self.command_buffer {
            Some(ref wrapper) => {
                let command_buffer = wrapper.read().unwrap().get();
                let command_buffer_ref = *command_buffer;

                let wait_masks = vec![wait_dst_stage_mask; wait_for.len()];
                let submit_info = ash::vk::SubmitInfo {
                    s_type: ash::vk::StructureType::SUBMIT_INFO,
                    p_next: ptr::null(),
                    wait_semaphore_count: wait_for.len() as u32,
                    p_wait_semaphores: wait_for.as_ptr(),
                    p_wait_dst_stage_mask: wait_masks.as_ptr(),
                    command_buffer_count: 1,
                    p_command_buffers: &command_buffer_ref,
                    signal_semaphore_count: signal_after.len() as u32,
                    p_signal_semaphores: signal_after.as_ptr(),
                };

                let queue_arc = Arc::clone(&queue);
                let queue_write = queue_arc.write().unwrap();

                let raw_device = Arc::clone(&self.raw_device);

                unsafe {
                    raw_device
                        .device()
                        .queue_submit(
                            *queue_write,
                            &[submit_info],
                            fence.unwrap_or(ash::vk::Fence::null()),
                        )
                        .expect("queue submit failed.");
                }
            }
            None => {}
        }

        self.command_buffer = None;

        Ok(())
    }

    #[inline]
    pub fn is_open(&self) -> bool {
        match self.command_buffer {
            Some(ref command_buffer) => command_buffer.read().unwrap().is_open(),
            None => false,
        }
    }

    #[inline]
    pub fn list_type(&self) -> RenderCommandListType {
        self.list_type
    }
}

impl RenderResourceBase for RenderCommandListVk {
    #[inline]
    fn get_type(&self) -> RenderResourceType {
        RenderResourceType::CommandList
    }

    #[inline]
    fn get_name(&self) -> &str {
        &self.name
    }
}

pub fn make_vulkan_sampler(desc: &RenderSamplerState) -> ash::vk::SamplerCreateInfo {
    #[inline(always)]
    fn get_addressing_mode(address_mode: RenderSamplerAddressMode) -> ash::vk::SamplerAddressMode {
        match address_mode {
            RenderSamplerAddressMode::Wrap => ash::vk::SamplerAddressMode::REPEAT,
            RenderSamplerAddressMode::Mirror => ash::vk::SamplerAddressMode::MIRRORED_REPEAT,
            RenderSamplerAddressMode::MirrorOnce => ash::vk::SamplerAddressMode::MIRRORED_REPEAT, // TODO: Where is VK_SAMPLER_ADDRESS_MODE_MIRROR_CLAMP_TO_EDGE?
            RenderSamplerAddressMode::Clamp => ash::vk::SamplerAddressMode::CLAMP_TO_EDGE,
            RenderSamplerAddressMode::Border => ash::vk::SamplerAddressMode::CLAMP_TO_BORDER,
        }
    }

    let mut sampler_info = ash::vk::SamplerCreateInfo::default();
    sampler_info.address_mode_u = get_addressing_mode(desc.address_u);
    sampler_info.address_mode_v = get_addressing_mode(desc.address_v);
    sampler_info.address_mode_w = get_addressing_mode(desc.address_w);
    sampler_info.min_lod = clamp(desc.min_mip, 0, 16) as f32; // clamp to valid range
    sampler_info.max_lod = clamp(desc.max_mip, 0, 16) as f32; // clamp to valid range
    sampler_info.mip_lod_bias = 0f32;

    // NOTE: Unfiltered loads(Load[], etc.) handled elsewhere
    sampler_info.unnormalized_coordinates = ash::vk::FALSE;

    match desc.filter {
        RenderSamplerFilter::MinMagMipPoint
        | RenderSamplerFilter::MinimumMinMagMipPoint
        | RenderSamplerFilter::MaximumMinMagMipPoint
        | RenderSamplerFilter::ComparisonMinMagMipPoint => {
            sampler_info.min_filter = ash::vk::Filter::NEAREST;
            sampler_info.mag_filter = ash::vk::Filter::NEAREST;
            sampler_info.mipmap_mode = ash::vk::SamplerMipmapMode::NEAREST;
            sampler_info.anisotropy_enable = ash::vk::FALSE;
            sampler_info.max_anisotropy = 1f32;
        }

        RenderSamplerFilter::MinMagPointMipLinear
        | RenderSamplerFilter::MinimumMinMagPointMipLinear
        | RenderSamplerFilter::MaximumMinMagPointMipLinear
        | RenderSamplerFilter::ComparisonMinMagPointMipLinear => {
            sampler_info.min_filter = ash::vk::Filter::NEAREST;
            sampler_info.mag_filter = ash::vk::Filter::NEAREST;
            sampler_info.mipmap_mode = ash::vk::SamplerMipmapMode::LINEAR;
            sampler_info.anisotropy_enable = ash::vk::FALSE;
            sampler_info.max_anisotropy = 1f32;
        }

        RenderSamplerFilter::MinPointMagLinearMipPoint
        | RenderSamplerFilter::MinimumMinPointMagLinearMipPoint
        | RenderSamplerFilter::MaximumMinPointMagLinearMipPoint
        | RenderSamplerFilter::ComparisonMinPointMagLinearMipPoint => {
            sampler_info.min_filter = ash::vk::Filter::NEAREST;
            sampler_info.mag_filter = ash::vk::Filter::LINEAR;
            sampler_info.mipmap_mode = ash::vk::SamplerMipmapMode::NEAREST;
            sampler_info.anisotropy_enable = ash::vk::FALSE;
            sampler_info.max_anisotropy = 1f32;
        }

        RenderSamplerFilter::MinPointMagMipLinear
        | RenderSamplerFilter::MinimumMinPointMagMipLinear
        | RenderSamplerFilter::MaximumMinPointMagMipLinear
        | RenderSamplerFilter::ComparisonMinPointMagMipLinear => {
            sampler_info.min_filter = ash::vk::Filter::NEAREST;
            sampler_info.mag_filter = ash::vk::Filter::LINEAR;
            sampler_info.mipmap_mode = ash::vk::SamplerMipmapMode::LINEAR;
            sampler_info.anisotropy_enable = ash::vk::FALSE;
            sampler_info.max_anisotropy = 1f32;
        }

        RenderSamplerFilter::MinLinearMagMipPoint
        | RenderSamplerFilter::MinimumMinLinearMagMipPoint
        | RenderSamplerFilter::MaximumMinLinearMagMipPoint
        | RenderSamplerFilter::ComparisonMinLinearMagMipPoint => {
            sampler_info.min_filter = ash::vk::Filter::LINEAR;
            sampler_info.mag_filter = ash::vk::Filter::NEAREST;
            sampler_info.mipmap_mode = ash::vk::SamplerMipmapMode::NEAREST;
            sampler_info.anisotropy_enable = ash::vk::FALSE;
            sampler_info.max_anisotropy = 1f32;
        }

        RenderSamplerFilter::MinLinearMagPointMipLinear
        | RenderSamplerFilter::MinimumMinLinearMagPointMipLinear
        | RenderSamplerFilter::MaximumMinLinearMagPointMipLinear
        | RenderSamplerFilter::ComparisonMinLinearMagPointMipLinear => {
            sampler_info.min_filter = ash::vk::Filter::LINEAR;
            sampler_info.mag_filter = ash::vk::Filter::NEAREST;
            sampler_info.mipmap_mode = ash::vk::SamplerMipmapMode::LINEAR;
            sampler_info.anisotropy_enable = ash::vk::FALSE;
            sampler_info.max_anisotropy = 1f32;
        }

        RenderSamplerFilter::MinMagLinearMipPoint
        | RenderSamplerFilter::MinimumMinMagLinearMipPoint
        | RenderSamplerFilter::MaximumMinMagLinearMipPoint
        | RenderSamplerFilter::ComparisonMinMagLinearMipPoint => {
            sampler_info.min_filter = ash::vk::Filter::LINEAR;
            sampler_info.mag_filter = ash::vk::Filter::LINEAR;
            sampler_info.mipmap_mode = ash::vk::SamplerMipmapMode::NEAREST;
            sampler_info.anisotropy_enable = ash::vk::FALSE;
            sampler_info.max_anisotropy = 1f32;
        }

        RenderSamplerFilter::MinMagMipLinear
        | RenderSamplerFilter::MinimumMinMagMipLinear
        | RenderSamplerFilter::MaximumMinMagMipLinear
        | RenderSamplerFilter::ComparisonMinMagMipLinear => {
            sampler_info.min_filter = ash::vk::Filter::LINEAR;
            sampler_info.mag_filter = ash::vk::Filter::LINEAR;
            sampler_info.mipmap_mode = ash::vk::SamplerMipmapMode::LINEAR;
            sampler_info.anisotropy_enable = ash::vk::FALSE;
            sampler_info.max_anisotropy = 1f32;
        }

        RenderSamplerFilter::Anisotropic
        | RenderSamplerFilter::MinimumAnisotropic
        | RenderSamplerFilter::MaximumAnisotropic
        | RenderSamplerFilter::ComparisonAnisotropic => {
            sampler_info.min_filter = ash::vk::Filter::LINEAR;
            sampler_info.mag_filter = ash::vk::Filter::LINEAR;
            sampler_info.mipmap_mode = ash::vk::SamplerMipmapMode::LINEAR;
            sampler_info.anisotropy_enable = ash::vk::TRUE;
            sampler_info.max_anisotropy = clamp(desc.max_aniso, 1, 16) as f32; // clamp to valid range
        }
    }

    sampler_info.border_color = match desc.border_color {
        RenderBorderColor::BlackA0 => ash::vk::BorderColor::FLOAT_TRANSPARENT_BLACK,
        RenderBorderColor::BlackA1 => ash::vk::BorderColor::FLOAT_OPAQUE_BLACK,
        RenderBorderColor::WhiteA1 => ash::vk::BorderColor::FLOAT_OPAQUE_WHITE,
    };

    sampler_info.compare_enable = bool_to_vk(is_comparison_filter(desc.filter));

    sampler_info.compare_op = get_compare_op(desc.comparison);

    let mut reduction_mode = Default::default();
    if is_min_filter(desc.filter) {
        reduction_mode = ash::vk::SamplerReductionModeEXT::MIN;
    } else if is_max_filter(desc.filter) {
        reduction_mode = ash::vk::SamplerReductionModeEXT::MAX;
    }

    // TODO: Figure out pointer chaining
    let _reduction_mode_info = ash::vk::SamplerReductionModeCreateInfoEXT::builder()
        .reduction_mode(reduction_mode)
        .build();

    let reduction_mode_ptr = std::ptr::null(); //&reduction_mode as *const std::ffi::c_void;
    sampler_info.p_next = reduction_mode_ptr;

    sampler_info
}

#[inline]
pub fn get_image_layout(states: RenderResourceStates) -> ash::vk::ImageLayout {
    // Images shouldn't have these set
    assert!(
        !states.contains(RenderResourceStates::VERTEX_AND_CONSTANT_BUFFER)
            && !states.contains(RenderResourceStates::INDEX_BUFFER)
            && !states.contains(RenderResourceStates::STREAM_OUT)
            && !states.contains(RenderResourceStates::INDIRECT_ARGUMENT)
    );

    if states == RenderResourceStates::COMMON {
        return ash::vk::ImageLayout::GENERAL;
    }

    if states.contains(RenderResourceStates::RENDER_TARGET) {
        return ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL;
    }

    // TODO: New depth/stencil read/write combinations

    if states.contains(RenderResourceStates::DEPTH_WRITE) {
        return ash::vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL;
    }

    if states.contains(RenderResourceStates::DEPTH_READ) {
        return ash::vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL;
    }

    // VK_IMAGE_LAYOUT_DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL_KHR
    // VK_IMAGE_LAYOUT_DEPTH_ATTACHMENT_STENCIL_READ_ONLY_OPTIMAL_KHR

    if states.contains(RenderResourceStates::PIXEL_SHADER_RESOURCE)
        || states.contains(RenderResourceStates::NON_PIXEL_SHADER_RESOURCE)
    {
        return ash::vk::ImageLayout::GENERAL; // return VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL;
    }

    if states.contains(RenderResourceStates::COPY_DEST) {
        return ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL;
    }

    if states.contains(RenderResourceStates::COPY_SOURCE) {
        return ash::vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
    }

    ash::vk::ImageLayout::GENERAL
}

#[inline]
pub fn get_image_access_type(states: RenderResourceStates) -> vk_sync::AccessType {
    // Images shouldn't have these set
    assert!(
        !states.contains(RenderResourceStates::VERTEX_AND_CONSTANT_BUFFER)
            && !states.contains(RenderResourceStates::INDEX_BUFFER)
            && !states.contains(RenderResourceStates::STREAM_OUT)
            && !states.contains(RenderResourceStates::INDIRECT_ARGUMENT)
    );

    if states == RenderResourceStates::COMMON || states == RenderResourceStates::UNORDERED_ACCESS {
        return vk_sync::AccessType::General;
    }

    if states.contains(RenderResourceStates::RENDER_TARGET) {
        return vk_sync::AccessType::ColorAttachmentWrite;
    }

    // TODO: New depth/stencil read/write combinations

    if states.contains(RenderResourceStates::DEPTH_WRITE) {
        return vk_sync::AccessType::DepthStencilAttachmentWrite;
    }

    if states.contains(RenderResourceStates::DEPTH_READ) {
        return vk_sync::AccessType::DepthStencilAttachmentRead;
    }

    if states.contains(RenderResourceStates::PIXEL_SHADER_RESOURCE)
        || states.contains(RenderResourceStates::NON_PIXEL_SHADER_RESOURCE)
    {
        //return vk_sync::AccessType::General;
        return vk_sync::AccessType::AnyShaderReadOther;
    }

    if states.contains(RenderResourceStates::COPY_DEST) {
        return vk_sync::AccessType::TransferWrite;
    }

    if states.contains(RenderResourceStates::COPY_SOURCE) {
        return vk_sync::AccessType::TransferRead;
    }

    vk_sync::AccessType::General
}

#[inline]
pub fn get_buffer_access_type(states: RenderResourceStates) -> vk_sync::AccessType {
    // Stream out is not supported on Vulkan
    assert!(!states.contains(RenderResourceStates::STREAM_OUT));

    // Buffer shouldn't have these set
    assert!(
        !states.contains(RenderResourceStates::RENDER_TARGET)
            && !states.contains(RenderResourceStates::DEPTH_WRITE)
            && !states.contains(RenderResourceStates::DEPTH_READ)
    );

    if states == RenderResourceStates::COMMON {
        return vk_sync::AccessType::General;
    }

    if states.contains(RenderResourceStates::UNORDERED_ACCESS) {
        return vk_sync::AccessType::General;
    }

    if states.contains(RenderResourceStates::VERTEX_AND_CONSTANT_BUFFER) {
        return vk_sync::AccessType::General; // return vk_sync::AccessType::VertexBuffer
    }

    if states.contains(RenderResourceStates::INDEX_BUFFER) {
        return vk_sync::AccessType::IndexBuffer;
    }

    if states.contains(RenderResourceStates::INDIRECT_ARGUMENT) {
        return vk_sync::AccessType::IndirectBuffer;
    }

    if states.contains(RenderResourceStates::PIXEL_SHADER_RESOURCE)
        || states.contains(RenderResourceStates::NON_PIXEL_SHADER_RESOURCE)
    {
        return vk_sync::AccessType::General;
        //return vk_sync::AccessType::AnyShaderReadOther;
    }

    if states.contains(RenderResourceStates::COPY_DEST) {
        return vk_sync::AccessType::TransferWrite;
    }

    if states.contains(RenderResourceStates::COPY_SOURCE) {
        return vk_sync::AccessType::TransferRead;
    }

    vk_sync::AccessType::General
}

#[inline(always)]
pub fn get_shader_stage(shader_type: RenderShaderType) -> ash::vk::ShaderStageFlags {
    match shader_type {
        RenderShaderType::Vertex => ash::vk::ShaderStageFlags::VERTEX,
        RenderShaderType::Geometry => ash::vk::ShaderStageFlags::GEOMETRY,
        RenderShaderType::Hull => ash::vk::ShaderStageFlags::TESSELLATION_CONTROL,
        RenderShaderType::Domain => ash::vk::ShaderStageFlags::TESSELLATION_EVALUATION,
        RenderShaderType::Pixel => ash::vk::ShaderStageFlags::FRAGMENT,
        RenderShaderType::Compute => ash::vk::ShaderStageFlags::COMPUTE,
    }
}
