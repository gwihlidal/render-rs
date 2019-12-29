#![allow(unused_variables)]
#![allow(dead_code)]

use crate::allocator::HostLinearAllocator;
use crate::backend::RenderBackendVk;
use crate::compile::RenderCompileContext;
use crate::descriptors::{
    merge_descriptor_set_layouts, tally_descriptor_pool_sizes, DescriptorSetCache,
};
use crate::queue::RenderCommandQueueVk;
use crate::raw::command;
use crate::raw::command::family;
use crate::raw::device::CreateQueueFamily;
use crate::raw::device::Device as RawDevice;
use crate::raw::device::PhysicalDevice;
use crate::raw::format::convert_format;
use crate::raw::format::get_image_aspect_flags;
use crate::raw::format::{
    bool_to_vk, get_blend_factor, get_blend_op, get_compare_op, get_cull_mode, get_polygon_mode,
    get_primitive_topology, get_stencil_op_state,
};
use crate::raw::image;
use crate::raw::image::convert_view_dimension_to_view_type;
use crate::raw::instance::Instance;
use crate::raw::surface::Surface as RawSurface;
use crate::raw::swap_chain::{SwapChain, SwapChainConfig};
use crate::shader_views::{ShaderResourceViewBinding, UnorderedAccessViewBinding};
use crate::types::*;
use ash::version::DeviceV1_0;
use ash::version::DeviceV1_1;
use ash::vk::Handle;
use ash::{self, Device};
use digest::Digest;
use meowhash;
use num_traits::FromPrimitive;
use render_core::constants::*;
use render_core::device::*;
use render_core::encoder::*;
use render_core::error::{Error, Result};
use render_core::format::*;
use render_core::handles::RenderResourceHandle;
use render_core::resources::{RenderResourceBase, RenderResourceStorage};
use render_core::state::*;
use render_core::types::*;
use render_core::utilities::any_as_u8_slice;
use spirv_reflect;
use std::hash::Hasher;
use std::{
    borrow::Cow,
    cell::RefCell,
    collections::HashMap,
    fmt,
    iter::once,
    mem,
    mem::align_of,
    ptr,
    sync::{Arc, RwLock},
    u32,
};
use twox_hash;
use vk_sync;
//use winit;

#[cfg(target_os = "macos")]
use ash::extensions::mvk::MacOSSurface;
#[cfg(target_os = "macos")]
use cocoa::appkit::{NSView, NSWindow};
#[cfg(target_os = "macos")]
use cocoa::base::id as cocoa_id;
#[cfg(target_os = "macos")]
use metal::CoreAnimationLayer;
#[cfg(target_os = "macos")]
use objc::runtime::YES;

#[cfg(target_os = "windows")]
use ash::extensions::khr::Win32Surface;

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
use ash::extensions::khr::XlibSurface;

pub const MAX_GPU_FRAMES: usize = 2;

//#[derive(Clone, Copy)]
pub struct RenderDeviceFrame {
    pub universal_pool: Arc<RwLock<CommandBufferPool>>,
    pub linear_allocator: Arc<RwLock<HostLinearAllocator>>,
}

pub struct RenderDeviceFrames {
    pub frames: [RenderDeviceFrame; MAX_GPU_FRAMES],
    pub frame_index: usize,
}

struct BindingSetRemap {
    binding_index: u32,
    old_binding: u32,
    new_binding: u32,
    old_set: u32,
    new_set: u32,
}

pub const CBV_OFFSET: u32 = 0;
pub const SRV_OFFSET: u32 = 30;
pub const SMP_OFFSET: u32 = 4;
pub const UAV_OFFSET: u32 = 60;
pub const SET_OFFSET: u32 = 4;
pub const ARG_OFFSET: u32 = 0;

//#[derive(Debug)]
pub struct RenderDeviceVk {
    device_info: RenderDeviceInfo,
    instance: Arc<Instance>,
    physical_device: Arc<PhysicalDevice>,
    logical_device: Arc<RawDevice>,
    frames: Arc<RwLock<RenderDeviceFrames>>,
    swap_chain_loader: Arc<ash::extensions::khr::Swapchain>,
    storage: Arc<RenderResourceStorage<Box<RenderResourceBase>>>,
    queue_info: [CreateQueueFamily; MAX_RENDER_QUEUES],
    //command_queues: [Option<RenderCommandQueueVk>; MAX_RENDER_QUEUES],
    command_queues: [Option<Arc<RwLock<ash::vk::Queue>>>; MAX_RENDER_QUEUES],
    present_command_list: RefCell<RenderCommandListVk>,
    compat_render_passes: RefCell<HashMap<u64, ash::vk::RenderPass>>,
    empty_descriptor_set_layout: ash::vk::DescriptorSetLayout,
    cbuffer_descriptor_set_layouts: [ash::vk::DescriptorSetLayout; MAX_SHADER_PARAMETERS],
    pipeline_cache: ash::vk::PipelineCache,
    descriptor_cache: Arc<DescriptorSetCache>,
    global_allocator: Arc<RwLock<vk_mem::Allocator>>,
}

impl fmt::Debug for RenderDeviceVk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TODO: RenderDeviceVk")
        //write!(f, "Point {{ x: {}, y: {} }}", self.x, self.y)
    }
}

impl RenderDeviceVk {
    pub fn new(
        device_info: RenderDeviceInfo,
        instance: Arc<Instance>,
        physical_device: Arc<PhysicalDevice>,
    ) -> Result<Self> {
        let surface_extensions = RawSurface::extensions();
        let swapchain_extensions = SwapChain::extensions();

        debug!("Picking families");

        for family in physical_device.families().into_iter() {
            debug!("iter family: {:?}", family);
        }
        /*physical_device
        .families()
        .into_iter()
        .map(|family| {
            debug!("iter family: {:?}", family);
        })
        .collect();*/

        let graphics_family = physical_device
            .families()
            .into_iter()
            .find(|family| {
                // TODO: Use platform specific query that doesn't use a surface instance
                //surface.supports_queue_family(&physical_device, family.index).unwrap_or(false) &&
                //family.capability.supports(command::Capability::Graphics)
                match family.capability {
                    command::Capability::Graphics => true,
                    command::Capability::General => true,
                    _ => false,
                }
            })
            .map(|family| CreateQueueFamily {
                family: family.index,
                count: 1,
            })
            .ok_or(format_err!("Can't find any graphics queues"))
            .unwrap();
        debug!("graphics family: {:?}", graphics_family);

        let compute_family = physical_device
            .families()
            .into_iter()
            .find(|family| {
                match family.capability {
                    command::Capability::Compute => true,
                    _ => false,
                }
                //family.capability.supports(command::Capability::Compute)
            })
            .map(|family| CreateQueueFamily {
                family: family.index,
                count: 1,
            });
        //.ok()//_or(format_err!("Can't find any compute queues"))
        //.unwrap();
        debug!("compute family: {:?}", compute_family);

        let transfer_family = physical_device
            .families()
            .into_iter()
            .find(|family| {
                match family.capability {
                    command::Capability::Transfer => true,
                    _ => false,
                }
                //family.capability.supports(command::Capability::Transfer)
            })
            .map(|family| CreateQueueFamily {
                family: family.index,
                count: 1,
            });
        //.ok_or(format_err!("Can't find any transfer queues"))
        //.unwrap();
        debug!("transfer family: {:?}", transfer_family);

        let mut families: Vec<CreateQueueFamily> = Vec::new();
        //if graphics_family.is_some() {
        families.push(graphics_family);
        //}
        if compute_family.is_some() {
            families.push(compute_family.unwrap());
        }
        if transfer_family.is_some() {
            families.push(transfer_family.unwrap());
        }

        //let families = &[graphics_family, compute_family, transfer_family];

        let mut queue_info: [CreateQueueFamily; MAX_RENDER_QUEUES] = Default::default();
        queue_info[RenderQueueType::Universal as usize] = graphics_family;
        queue_info[RenderQueueType::Compute as usize] = compute_family.unwrap_or_default();
        queue_info[RenderQueueType::Transfer as usize] = transfer_family.unwrap_or_default();

        //let formats = surface.supported_formats(&physical_device).unwrap().into_iter().collect::<Vec<_>>();
        //trace!("Picking format from: {:#?}", formats);
        //let format = formats[0];

        debug!("Creating device");
        let device_extensions = physical_device
            .extensions()
            .unwrap()
            .into_iter()
            .collect::<Vec<_>>();

        assert!(swapchain_extensions
            .iter()
            .all(|&swapchain_extension| device_extensions
                .iter()
                .find(|&extension| extension == swapchain_extension)
                .is_some()));

        let features = ash::vk::PhysicalDeviceFeatures::builder()
            .shader_clip_distance(true)
            .shader_cull_distance(true)
            .fill_mode_non_solid(true)
            .independent_blend(true)
            .build();
        // Features
        // https://www.khronos.org/registry/vulkan/specs/1.0/man/html/VkPhysicalDeviceFeatures.html
        // https://www.khronos.org/registry/spir-v/specs/1.0/SPIRV.html for shader features
        /*m_deviceFeatures.shaderClipDistance = VK_TRUE;
        m_deviceFeatures.shaderCullDistance = VK_TRUE;
        m_deviceFeatures.fillModeNonSolid = VK_TRUE;
        m_deviceFeatures.independentBlend = VK_TRUE;
        m_deviceFeatures.imageCubeArray = VK_TRUE;
        m_deviceFeatures.tessellationShader = VK_TRUE;
        m_deviceFeatures.textureCompressionBC = VK_TRUE;
        m_deviceFeatures.occlusionQueryPrecise = VK_TRUE;
        m_deviceFeatures.samplerAnisotropy = VK_TRUE;
        m_deviceFeatures.shaderImageGatherExtended = VK_TRUE;
        m_deviceFeatures.shaderStorageImageMultisample = VK_TRUE;
        m_deviceFeatures.shaderStorageImageExtendedFormats = VK_TRUE;	// Required for RWBuffer<uint> and friends
        m_deviceFeatures.depthBounds = VK_TRUE;
        m_deviceFeatures.depthClamp = VK_TRUE;
        m_deviceFeatures.depthBiasClamp = VK_TRUE;
        m_deviceFeatures.multiDrawIndirect = VK_TRUE;
        m_deviceFeatures.drawIndirectFirstInstance = VK_TRUE;
        m_deviceFeatures.dualSrcBlend = VK_TRUE;
        //m_deviceFeatures.logicOp = VK_TRUE;
        m_deviceFeatures.fullDrawIndexUint32 = VK_TRUE;
        m_deviceFeatures.robustBufferAccess = VK_TRUE; // TODO: This is slower, so perhaps handle the safety more explicitly?
        m_deviceFeatures.geometryShader = VK_TRUE;
        // TODO: Bindless
        //m_deviceFeatures.shaderUniformBufferArrayDynamicIndexing = VK_TRUE;
        //m_deviceFeatures.shaderSampledImageArrayDynamicIndexing = VK_TRUE;
        //m_deviceFeatures.shaderStorageBufferArrayDynamicIndexing = VK_TRUE;
        //m_deviceFeatures.shaderStorageImageArrayDynamicIndexing = VK_TRUE;
        m_deviceFeatures.fragmentStoresAndAtomics = VK_TRUE;
        m_deviceFeatures.vertexPipelineStoresAndAtomics = VK_TRUE;*/

        let mut raw_device = RawDevice::create(
            Arc::clone(&physical_device),
            families, //once(families),
            swapchain_extensions.into_iter().map(String::from),
            features,
        )
        .unwrap();

        let mut command_queues: [Option<Arc<RwLock<ash::vk::Queue>>>; MAX_RENDER_QUEUES] =
            Default::default();

        for family in &mut raw_device.families {
            let queues = family.queues();
            for queue in queues.iter() {
                match queue.id.family.capability {
                    command::Capability::Graphics => {
                        command_queues[RenderQueueType::Universal as usize] =
                            Some(Arc::new(RwLock::new(queue.raw)));
                    }
                    command::Capability::General => {
                        command_queues[RenderQueueType::Universal as usize] =
                            Some(Arc::new(RwLock::new(queue.raw)));
                    }
                    command::Capability::Compute => {
                        command_queues[RenderQueueType::Compute as usize] =
                            Some(Arc::new(RwLock::new(queue.raw)));
                    }
                    command::Capability::Transfer => {
                        command_queues[RenderQueueType::Transfer as usize] =
                            Some(Arc::new(RwLock::new(queue.raw)));
                    }
                }
                debug!("queue id:{:?} raw:{:?}", queue.id, queue.raw);
            }
            debug!("family: {:?}", queues.len());
        }

        let device = Arc::new(raw_device);

        let raw_device = device.device();
        let swap_chain_loader = Arc::new(ash::extensions::khr::Swapchain::new(
            instance.get_instance(),
            raw_device,
        ));

        let frame0 = RenderDeviceFrame {
            universal_pool: Arc::new(RwLock::new(CommandBufferPool::new(device.clone(), 0))),
            linear_allocator: Arc::new(RwLock::new(HostLinearAllocator::new(
                physical_device.raw,
                (*raw_device).clone(),
                (*instance).get_instance().clone(),
                16 * 1024 * 1024,
            ))),
        };

        let frame1 = RenderDeviceFrame {
            universal_pool: Arc::new(RwLock::new(CommandBufferPool::new(device.clone(), 0))),
            linear_allocator: Arc::new(RwLock::new(HostLinearAllocator::new(
                physical_device.raw,
                (*raw_device).clone(),
                (*instance).get_instance().clone(),
                16 * 1024 * 1024,
            ))),
        };

        let frames = Arc::new(RwLock::new(RenderDeviceFrames {
            frame_index: 0,
            frames: [frame0, frame1],
        }));

        let present_command_list = RefCell::new(RenderCommandListVk::new(
            device.clone(),
            frames.clone(),
            RenderCommandListType::Universal,
            "Present Command List".into(),
        ));

        let dummy_layout = ash::vk::DescriptorSetLayoutCreateInfo::default();
        let empty_descriptor_set_layout = unsafe {
            raw_device
                .create_descriptor_set_layout(&dummy_layout, None)
                .unwrap()
        };

        let mut cbuffer_descriptor_set_layouts: [ash::vk::DescriptorSetLayout;
            MAX_SHADER_PARAMETERS] = Default::default();
        for cbuffer_index in 0..MAX_SHADER_PARAMETERS {
            let cbuffer_binding = ash::vk::DescriptorSetLayoutBinding::builder()
                .descriptor_count(1)
                .descriptor_type(ash::vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                .stage_flags(ash::vk::ShaderStageFlags::ALL)
                .binding(cbuffer_index as u32)
                .build();

            let cbuffer_layout = ash::vk::DescriptorSetLayoutCreateInfo::builder()
                .bindings(&[cbuffer_binding])
                .build();

            cbuffer_descriptor_set_layouts[cbuffer_index] = unsafe {
                raw_device
                    .create_descriptor_set_layout(&cbuffer_layout, None)
                    .unwrap()
            };
        }

        let pipeline_cache_info = ash::vk::PipelineCacheCreateInfo::builder().build();
        let pipeline_cache = unsafe {
            raw_device
                .create_pipeline_cache(&pipeline_cache_info, None)
                .unwrap()
        };

        let storage = Arc::new(RenderResourceStorage::new());
        let descriptor_cache = Arc::new(DescriptorSetCache::new(device.clone(), storage.clone()));

        Ok(RenderDeviceVk {
            device_info,
            instance: instance.clone(),
            physical_device: physical_device.clone(),
            logical_device: device.clone(),
            frames,
            swap_chain_loader,
            storage,
            queue_info,
            command_queues,
            present_command_list,
            compat_render_passes: RefCell::new(HashMap::new()),
            empty_descriptor_set_layout,
            cbuffer_descriptor_set_layouts,
            pipeline_cache,
            descriptor_cache,
            global_allocator: Arc::new(RwLock::new(
                vk_mem::Allocator::new(&vk_mem::AllocatorCreateInfo {
                    physical_device: physical_device.raw,
                    device: (*raw_device).clone(),
                    instance: (*instance).get_instance().clone(),
                    ..Default::default()
                })
                .unwrap(),
            )),
        })
    }

    /*fn advance(&self) -> Result<()> {
        Ok(())
    }*/

    fn get_queue(&self, queue_type: RenderQueueType) -> &Option<Arc<RwLock<ash::vk::Queue>>> {
        &self.command_queues[queue_type as usize]
    }

    fn get_list_queue(
        &self,
        list_type: RenderCommandListType,
    ) -> &Option<Arc<RwLock<ash::vk::Queue>>> {
        match list_type {
            RenderCommandListType::Universal => self.get_universal_queue(),
            RenderCommandListType::Present => self.get_universal_queue(),
            RenderCommandListType::Compute => self.get_compute_queue(),
            RenderCommandListType::Transfer => self.get_transfer_queue(),
            _ => unimplemented!(),
        }
    }

    fn get_universal_queue(&self) -> &Option<Arc<RwLock<ash::vk::Queue>>> {
        self.get_queue(RenderQueueType::Universal)
    }

    fn get_compute_queue(&self) -> &Option<Arc<RwLock<ash::vk::Queue>>> {
        self.get_queue(RenderQueueType::Compute)
    }

    fn get_transfer_queue(&self) -> &Option<Arc<RwLock<ash::vk::Queue>>> {
        self.get_queue(RenderQueueType::Transfer)
    }

    fn flush_transfers(&self) {}
}

impl Drop for RenderDeviceVk {
    fn drop(&mut self) {
        trace!("Drop called for RenderDeviceVk!");
        let device = Arc::clone(&self.logical_device);
        let raw_device = device.device();

        for _ in 0..MAX_GPU_FRAMES {
            // Flush any frames in flight
            match self.advance_frame() {
                Ok(_) => {}
                Err(err) => error!("Error flushing frames during render device drop: {:?}", err),
            }
        }

        unsafe {
            raw_device.device_wait_idle().unwrap();
            if self.pipeline_cache != ash::vk::PipelineCache::null() {
                raw_device.destroy_pipeline_cache(self.pipeline_cache, None);
            }
            if self.empty_descriptor_set_layout != ash::vk::DescriptorSetLayout::null() {
                raw_device.destroy_descriptor_set_layout(self.empty_descriptor_set_layout, None);
            }
            for i in 0..MAX_SHADER_PARAMETERS {
                let cbuffer_layout = self.cbuffer_descriptor_set_layouts[i];
                if cbuffer_layout != ash::vk::DescriptorSetLayout::null() {
                    raw_device.destroy_descriptor_set_layout(cbuffer_layout, None);
                }
            }

            for (_, pass) in self.compat_render_passes.borrow().iter() {
                raw_device.destroy_render_pass(*pass, None);
            }
        }
    }
}

impl RenderDevice for RenderDeviceVk {
    fn valid_resource(&self, handle: RenderResourceHandle) -> bool {
        self.storage.valid(handle)
    }

    fn destroy_resource(&self, handle: RenderResourceHandle) -> Result<()> {
        let instance = Arc::clone(&self.instance);
        let device = Arc::clone(&self.logical_device);
        let raw_device = device.device();
        let resource_lock = self.storage.remove(handle)?;
        let mut resource = resource_lock.write().unwrap();
        info!(
            "Destroying resource - name: {}, handle: {:?}",
            resource.get_name(),
            handle
        );
        assert!(resource.get_type() == handle.get_type());
        match handle.get_type() {
            RenderResourceType::SwapChain => {
                let resource = resource.downcast_mut::<RenderSwapChainVk>().unwrap();
                unsafe {
                    raw_device.destroy_semaphore(resource.acquire_image_semaphore, None);
                    raw_device.destroy_semaphore(resource.render_done_semaphore, None);
                    self.swap_chain_loader
                        .destroy_swapchain(resource.swap_chain, None);
                    instance
                        .get_surface_loader()
                        .destroy_surface(resource.surface, None);
                }
            }
            RenderResourceType::Buffer => {
                let resource = resource.downcast_mut::<RenderBufferVk>().unwrap();
                self.global_allocator
                    .write()
                    .unwrap()
                    .destroy_buffer(resource.buffer, &resource.allocation)
                    .unwrap();
            }
            RenderResourceType::Texture => {
                let resource = resource.downcast_mut::<RenderTextureVk>().unwrap();
                unsafe {
                    if let Some(ref allocation) = resource.allocation {
                        self.global_allocator
                            .write()
                            .unwrap()
                            .destroy_image(resource.image, &allocation)
                            .unwrap();
                    } else {
                        // Image not created through vk_mem (i.e. swap chain)
                        raw_device.destroy_image(resource.image, None);
                    }
                }
            }
            RenderResourceType::SamplerState => {
                let resource = resource.downcast_mut::<RenderSamplerStateVk>().unwrap();
                unsafe {
                    raw_device.destroy_sampler(resource.sampler, None);
                }
            }
            RenderResourceType::Shader => {
                let shader = resource.downcast_mut::<RenderShaderVk>().unwrap();
                unsafe { raw_device.destroy_shader_module(shader.module, None) };
            }
            RenderResourceType::ShaderViews => {
                let resource = resource.downcast_mut::<RenderShaderViewsVk>().unwrap();
                for cached_set in &resource.cached_descriptor_sets {
                    // TODO: Don't free the descriptor, just the pool, until the pool is no longer tied to this object
                    unsafe {
                        raw_device.destroy_descriptor_pool(cached_set.descriptor_pool, None);
                    }
                }

                resource.cached_descriptor_sets.clear();

                for srv_binding in &mut resource.srvs {
                    if srv_binding.image_view != ash::vk::ImageView::null() {
                        unsafe {
                            raw_device.destroy_image_view(srv_binding.image_view, None);
                        }
                    }

                    if srv_binding.buffer_view != ash::vk::BufferView::null() {
                        unsafe {
                            raw_device.destroy_buffer_view(srv_binding.buffer_view, None);
                        }
                    }
                }

                resource.srvs.clear();

                for uav_binding in &mut resource.uavs {
                    if uav_binding.image_view != ash::vk::ImageView::null() {
                        unsafe {
                            raw_device.destroy_image_view(uav_binding.image_view, None);
                        }
                    }

                    if uav_binding.buffer_view != ash::vk::BufferView::null() {
                        unsafe {
                            raw_device.destroy_buffer_view(uav_binding.buffer_view, None);
                        }
                    }
                }

                resource.uavs.clear();
            }
            RenderResourceType::GraphicsPipelineState => {
                let resource = resource
                    .downcast_mut::<RenderGraphicsPipelineStateVk>()
                    .unwrap();

                unsafe {
                    raw_device.destroy_pipeline(resource.pipeline, None);

                    for sampler in &resource.data.static_samplers {
                        raw_device.destroy_sampler(*sampler, None);
                    }

                    for layout in &resource.data.combined_layouts {
                        raw_device.destroy_descriptor_set_layout(layout.layout, None);
                    }

                    raw_device.destroy_pipeline_layout(resource.data.pipeline_layout, None);
                }
            }
            RenderResourceType::ComputePipelineState => {
                let resource = resource
                    .downcast_mut::<RenderComputePipelineStateVk>()
                    .unwrap();

                unsafe {
                    raw_device.destroy_pipeline(resource.pipeline, None);

                    for sampler in &resource.data.static_samplers {
                        raw_device.destroy_sampler(*sampler, None);
                    }

                    for layout in &resource.data.combined_layouts {
                        raw_device.destroy_descriptor_set_layout(layout.layout, None);
                    }

                    raw_device.destroy_pipeline_layout(resource.data.pipeline_layout, None);
                }
            }
            RenderResourceType::RayTracingGeometry => {
                let _resource = resource
                    .downcast_mut::<RenderRayTracingGeometryVk>()
                    .unwrap();
                unimplemented!()
            }
            RenderResourceType::RayTracingProgram => {
                let _resource = resource
                    .downcast_mut::<RenderRayTracingProgramVk>()
                    .unwrap();
                unimplemented!()
            }
            RenderResourceType::RayTracingAcceleration => {
                let _resource = resource
                    .downcast_mut::<RenderRayTracingAccelerationVk>()
                    .unwrap();
                unimplemented!()
            }
            RenderResourceType::RayTracingPipelineState => {
                let mut _resource = resource
                    .downcast_ref::<RenderRayTracingPipelineStateVk>()
                    .unwrap();
                unimplemented!()
            }
            RenderResourceType::RayTracingShaderTable => {
                let _resource = resource
                    .downcast_mut::<RenderRayTracingShaderTableVk>()
                    .unwrap();
                unimplemented!()
            }
            RenderResourceType::DrawBindingSet => {
                let _ = resource.downcast_mut::<RenderDrawBindingSetVk>().unwrap();
            }
            RenderResourceType::FrameBindingSet => {
                let resource = resource.downcast_ref::<RenderFrameBindingSetVk>().unwrap();
                for &view in resource.image_views.iter() {
                    unsafe { raw_device.destroy_image_view(view, None) };
                }
            }
            RenderResourceType::RenderPass => {
                let resource = resource.downcast_ref::<RenderPassVk>().unwrap();
                unsafe {
                    raw_device.destroy_framebuffer(resource.frame_buffer, None);
                    raw_device.destroy_render_pass(resource.render_pass, None);
                }
            }
            RenderResourceType::CommandList => {
                let _ = resource.downcast_mut::<RenderCommandListVk>().unwrap();
            }
            RenderResourceType::Fence => {
                let resource = resource.downcast_mut::<RenderFenceVk>().unwrap();
                unsafe {
                    raw_device.destroy_fence(resource.fence, None);
                }
            }
            RenderResourceType::TimingHeap => {
                let _resource = resource.downcast_mut::<RenderTimingHeapVk>().unwrap();
                unimplemented!()

                /*for (auto& buffer : timingHeap.second->buffers)
                {
                    vkDestroyQueryPool(m_logicalDevice, buffer.queryPool, nullptr);
                    delete buffer.writeFence;
                    buffer.readBack->destroy();
                    buffer.readBack.reset();
                }*/
            }
        }
        Ok(())
    }

    // Resource Management
    fn create_swap_chain(
        &self,
        handle: RenderResourceHandle,
        desc: &RenderSwapChainDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        info!(
            "Creating swap chain: name:{}, format:{:?}, width:{}, height:{}, buffers:{}",
            debug_name, desc.format, desc.width, desc.height, desc.buffer_count
        );
        let device = Arc::clone(&self.logical_device);
        let raw_pdevice = device.physical;
        let raw_device = device.device();

        let instance = Arc::clone(&self.instance);
        let raw_instance: &ash::Instance = instance.get_instance();
        let raw_entry: &ash::Entry = instance.get_entry();

        let surface = unsafe { create_surface(raw_entry, raw_instance, &desc.window).unwrap() };

        let surface_formats = unsafe {
            instance
                .get_surface_loader()
                .get_physical_device_surface_formats(raw_pdevice, surface)
                .unwrap()
        };

        let surface_format = surface_formats
            .iter()
            .map(|sfmt| match sfmt.format {
                ash::vk::Format::UNDEFINED => ash::vk::SurfaceFormatKHR {
                    format: ash::vk::Format::B8G8R8_UNORM,
                    color_space: sfmt.color_space,
                },
                _ => sfmt.clone(),
            })
            .nth(0)
            .expect("Unable to find suitable surface format.");

        let surface_capabilities = unsafe {
            instance
                .get_surface_loader()
                .get_physical_device_surface_capabilities(raw_pdevice, surface)
                .unwrap()
        };

        let universal_info = &self.queue_info[RenderQueueType::Universal as usize];
        let supports_surface = instance
            .get_surface()
            .as_ref()
            .unwrap()
            .supports_queue_family(raw_pdevice, surface, universal_info.family)
            .unwrap_or(false);

        if !supports_surface {
            unimplemented!()
        }

        let mut desired_image_count = surface_capabilities.min_image_count + 1;
        if surface_capabilities.max_image_count > 0
            && desired_image_count > surface_capabilities.max_image_count
        {
            desired_image_count = surface_capabilities.max_image_count;
        }

        let surface_resolution = match surface_capabilities.current_extent.width {
            u32::MAX => ash::vk::Extent2D {
                width: desc.width,
                height: desc.height,
            },
            _ => surface_capabilities.current_extent,
        };

        let pre_transform = if surface_capabilities
            .supported_transforms
            .contains(ash::vk::SurfaceTransformFlagsKHR::IDENTITY)
        {
            ash::vk::SurfaceTransformFlagsKHR::IDENTITY
        } else {
            surface_capabilities.current_transform
        };

        let present_modes = unsafe {
            instance
                .get_surface_loader()
                .get_physical_device_surface_present_modes(raw_pdevice, surface)
                .unwrap()
        };

        let present_mode = present_modes
            .iter()
            .cloned()
            .find(|&mode| mode == ash::vk::PresentModeKHR::MAILBOX)
            .unwrap_or(ash::vk::PresentModeKHR::FIFO);

        let swap_chain_info = ash::vk::SwapchainCreateInfoKHR {
            s_type: ash::vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: Default::default(),
            surface: surface,
            min_image_count: desired_image_count,
            image_color_space: surface_format.color_space,
            image_format: surface_format.format,
            image_extent: surface_resolution.clone(),
            image_usage: ash::vk::ImageUsageFlags::TRANSFER_DST
                | ash::vk::ImageUsageFlags::COLOR_ATTACHMENT
                | ash::vk::ImageUsageFlags::SAMPLED,
            image_sharing_mode: ash::vk::SharingMode::EXCLUSIVE,
            pre_transform: pre_transform,
            composite_alpha: ash::vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode: present_mode,
            clipped: 1,
            old_swapchain: ash::vk::SwapchainKHR::null(),
            image_array_layers: 1,
            p_queue_family_indices: ptr::null(),
            queue_family_index_count: 0,
        };

        let swap_chain = unsafe {
            self.swap_chain_loader
                .create_swapchain(&swap_chain_info, None)
                .unwrap()
        };

        let images = unsafe {
            self.swap_chain_loader
                .get_swapchain_images(swap_chain)
                .unwrap()
        };

        let tex_desc = RenderTextureDesc {
            texture_type: RenderTextureType::Tex2d,
            bind_flags: RenderBindFlags::RENDER_TARGET,
            format: desc.format,
            width: desc.width,
            height: desc.height,
            depth: 1,
            elements: 1,
            levels: 1,
        };

        //debug::setImageName(m_logicalDevice, texture->image, "Swap Chain Image");

        let textures: Vec<RenderTextureVk> = images
            .iter()
            .map(|&image| {
                RenderTextureVk {
                    name: "Swap Chain Texture".into(),
                    desc: tex_desc.clone(),
                    image,
                    allocation: None,
                    supported_states: get_resource_states(tex_desc.bind_flags),
                    default_state: RenderResourceStates::COMMON, //PRESENT,
                }
            })
            .collect();

        let semaphore_create_info = ash::vk::SemaphoreCreateInfo {
            s_type: ash::vk::StructureType::SEMAPHORE_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
        };

        let acquire_image_semaphore = unsafe {
            raw_device
                .create_semaphore(&semaphore_create_info, None)
                .unwrap()
        };

        let render_done_semaphore = unsafe {
            raw_device
                .create_semaphore(&semaphore_create_info, None)
                .unwrap()
        };

        let resource: Arc<RwLock<Box<RenderResourceBase>>> =
            Arc::new(RwLock::new(Box::new(RenderSwapChainVk {
                name: debug_name.to_string().into(),
                swap_chain,
                swap_chain_info,
                surface,
                textures,
                back_buffer_index: 0,
                acquire_image_semaphore,
                render_done_semaphore,
            })));

        self.storage.put(handle, resource)?;
        Ok(())
    }

    fn create_buffer(
        &self,
        handle: RenderResourceHandle,
        desc: &RenderBufferDesc,
        initial_data: Option<&[u8]>,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        info!("Creating buffer: {}, {:?}", debug_name, desc);
        let pdevice = Arc::clone(&self.physical_device);
        let device = Arc::clone(&self.logical_device);
        let raw_device = device.device();

        // https://www.khronos.org/registry/vulkan/specs/1.0/html/vkspec.html#VkMemoryPropertyFlagBits
        let mut usage =
            ash::vk::BufferUsageFlags::TRANSFER_DST | ash::vk::BufferUsageFlags::TRANSFER_SRC;

        if desc.bind_flags.contains(RenderBindFlags::SHADER_RESOURCE)
            || desc.bind_flags.contains(RenderBindFlags::UNORDERED_ACCESS)
        {
            usage |= ash::vk::BufferUsageFlags::UNIFORM_TEXEL_BUFFER
                | ash::vk::BufferUsageFlags::UNIFORM_BUFFER
                | ash::vk::BufferUsageFlags::STORAGE_TEXEL_BUFFER
                | ash::vk::BufferUsageFlags::STORAGE_BUFFER;
        }

        if desc.bind_flags.contains(RenderBindFlags::VERTEX_BUFFER)
            || desc.bind_flags.contains(RenderBindFlags::CONSTANT_BUFFER)
        {
            usage |= ash::vk::BufferUsageFlags::VERTEX_BUFFER
                | ash::vk::BufferUsageFlags::UNIFORM_BUFFER;
        }

        if desc.bind_flags.contains(RenderBindFlags::INDEX_BUFFER) {
            usage |= ash::vk::BufferUsageFlags::INDEX_BUFFER;
        }

        if desc.bind_flags.contains(RenderBindFlags::INDIRECT_BUFFER) {
            usage |= ash::vk::BufferUsageFlags::INDIRECT_BUFFER;
        }

        if desc.bind_flags.contains(RenderBindFlags::STREAM_OUTPUT) {
            usage |= ash::vk::BufferUsageFlags::STORAGE_BUFFER;
        }

        let supported_states = get_resource_states(desc.bind_flags)
            | RenderResourceStates::COPY_SOURCE
            | RenderResourceStates::COPY_DEST;

        let default_state = get_default_resource_states(desc.bind_flags);

        let allocation_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::GpuOnly,
            ..Default::default()
        };

        let (buffer, allocation, allocation_info) = self
            .global_allocator
            .write()
            .unwrap()
            .create_buffer(
                &ash::vk::BufferCreateInfo::builder()
                    .size(desc.size as u64)
                    .usage(usage)
                    .build(),
                &allocation_info,
            )
            .unwrap();

        if let Some(data) = initial_data {
            let scratch_allocation = {
                let mut frames = self.frames.write().unwrap();
                let frame_index = frames.frame_index;
                let allocator = &mut frames.frames[frame_index].linear_allocator;
                let allocation = allocator.write().unwrap().allocate(desc.size, None);
                allocation
            };

            let mut data_slice = unsafe {
                ash::util::Align::new(
                    scratch_allocation.address as *mut ::std::ffi::c_void,
                    align_of::<u8>() as u64,
                    desc.size as u64,
                )
            };
            data_slice.copy_from_slice(&data);

            let vk_device = self.logical_device.device();
            let transfer_list = self.present_command_list.borrow_mut().open()?;

            unsafe {
                vk_device.cmd_copy_buffer(
                    *transfer_list,
                    scratch_allocation.buffer,
                    buffer,
                    &[ash::vk::BufferCopy::builder()
                        .dst_offset(0)
                        .src_offset(scratch_allocation.offset as u64)
                        .size(desc.size as u64)
                        .build()],
                );
            }

            self.present_command_list.borrow_mut().close()?;
            if let Some(ref queue) = self.get_universal_queue() {
                self.present_command_list.borrow_mut().submit(
                    queue.clone(),
                    &[],
                    &[],
                    None,
                    ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                )?;
            }
        }

        let resource: Arc<RwLock<Box<RenderResourceBase>>> =
            Arc::new(RwLock::new(Box::new(RenderBufferVk {
                name: debug_name.to_string().into(),
                desc: desc.clone(),
                buffer,
                allocation,
                default_state,
                supported_states,
            })));

        self.storage.put(handle, resource)?;
        Ok(())
    }

    fn create_texture(
        &self,
        handle: RenderResourceHandle,
        desc: &RenderTextureDesc,
        initial_data: Option<RenderTextureSubResourceData>,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        info!("Creating texture: {}, {:?}", debug_name, desc);
        assert_eq!(handle.get_type(), RenderResourceType::Texture);

        let pdevice = Arc::clone(&self.physical_device);
        let device = Arc::clone(&self.logical_device);
        let raw_device = device.device();

        let supported_states = get_resource_states(desc.bind_flags)
            | RenderResourceStates::COPY_SOURCE
            | RenderResourceStates::COPY_DEST;

        // We want render targets to default to read, so that all other
        // read only resources never need to be checked for transitions
        let mut default_state = RenderResourceStates::NON_PIXEL_SHADER_RESOURCE
            | RenderResourceStates::PIXEL_SHADER_RESOURCE;

        if desc.bind_flags.contains(RenderBindFlags::CROSS_DEVICE) {
            assert!(initial_data.is_none());
            default_state = RenderResourceStates::COMMON; // Copy not allowed on Copy queue.. stupid
        } else if desc.bind_flags.contains(RenderBindFlags::SHADER_RESOURCE) {
            default_state = RenderResourceStates::NON_PIXEL_SHADER_RESOURCE
                | RenderResourceStates::PIXEL_SHADER_RESOURCE;
        } else if desc.bind_flags.contains(RenderBindFlags::CONSTANT_BUFFER)
            || desc.bind_flags.contains(RenderBindFlags::VERTEX_BUFFER)
        {
            default_state = RenderResourceStates::VERTEX_AND_CONSTANT_BUFFER;
        } else if desc.bind_flags.contains(RenderBindFlags::INDEX_BUFFER) {
            default_state = RenderResourceStates::INDEX_BUFFER;
        } else if desc.bind_flags.contains(RenderBindFlags::RENDER_TARGET) {
            default_state = RenderResourceStates::RENDER_TARGET;
        } else if desc.bind_flags.contains(RenderBindFlags::DEPTH_STENCIL) {
            default_state = RenderResourceStates::DEPTH_WRITE;
        }

        trace!("Supported States: {:?}", supported_states);
        trace!("Default State: {:?}", default_state);

        let create_info = image::get_image_create_info(&desc, initial_data.is_some());

        let is_cube = match desc.texture_type {
            RenderTextureType::Cube | RenderTextureType::CubeArray => true,
            _ => false,
        };

        let layer_count = match is_cube {
            true => desc.elements * 6,
            false => desc.elements,
        } as u32;

        let image_range = ash::vk::ImageSubresourceRange {
            aspect_mask: get_image_aspect_flags(desc.format, false /* ignore stencil */),
            base_mip_level: 0,
            level_count: desc.levels as u32,
            base_array_layer: 0,
            layer_count,
        };

        let allocation_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::GpuOnly,
            ..Default::default()
        };

        let (image, allocation, allocation_info) = self
            .global_allocator
            .write()
            .unwrap()
            .create_image(&create_info, &allocation_info)
            .unwrap();

        let pre_write_barrier = vk_sync::ImageBarrier {
            previous_accesses: vec![vk_sync::AccessType::Nothing],
            next_accesses: vec![vk_sync::AccessType::TransferWrite],
            previous_layout: vk_sync::ImageLayout::General,
            next_layout: vk_sync::ImageLayout::Optimal,
            discard_contents: true,
            src_queue_family_index: ash::vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: ash::vk::QUEUE_FAMILY_IGNORED,
            image,
            range: image_range,
        };

        let post_write_barrier = vk_sync::ImageBarrier {
            previous_accesses: vec![vk_sync::AccessType::TransferWrite],
            next_accesses: vec![vk_sync::AccessType::General],
            previous_layout: vk_sync::ImageLayout::Optimal,
            next_layout: vk_sync::ImageLayout::Optimal,
            discard_contents: false,
            src_queue_family_index: ash::vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: ash::vk::QUEUE_FAMILY_IGNORED,
            image,
            range: image_range,
        };

        let vk_device = self.logical_device.device();
        let transfer_list = self.present_command_list.borrow_mut().open()?;

        vk_sync::cmd::pipeline_barrier(
            &vk_device.fp_v1_0(),
            *transfer_list,
            None,
            &[],
            &[pre_write_barrier],
        );

        if let Some(ref initial_data) = initial_data {
            let max_mip_levels = std::cmp::min(
                desc.levels as u32,
                get_texture_max_mip_count(desc.width as u32, desc.height as u32, desc.depth as u32),
            );
            assert_eq!(max_mip_levels, desc.levels as u32);
            let sub_resource_count = layer_count * desc.levels as u32;
            let size = get_texture_size(
                desc.format,
                desc.width as u32,
                desc.height as u32,
                desc.depth as u32,
                desc.levels as u32,
                match is_cube {
                    true => desc.elements * 6,
                    false => desc.elements,
                } as u32,
            );
            for sub_resource in 0..sub_resource_count {
                let surface_width = desc.width as u32;
                let surface_height = desc.height as u32;

                let mip_index =
                    get_texture_sub_resource_mip_index(sub_resource, desc.levels.into());
                let slice_index =
                    get_texture_sub_resource_slice_index(sub_resource, desc.levels.into());
                assert_eq!(
                    calc_texture_sub_resource_index(mip_index, slice_index, desc.levels.into()),
                    sub_resource
                );

                let mip_width = std::cmp::max(1u32, desc.width as u32 >> mip_index);
                let mip_height = std::cmp::max(1u32, desc.height as u32 >> mip_index);
                let mip_depth = std::cmp::max(1u32, desc.depth as u32 >> mip_index);

                let row_pitch = initial_data.row_pitch;
                let slice_pitch = match initial_data.slice_pitch {
                    0 => row_pitch * mip_height,
                    _ => initial_data.slice_pitch,
                };

                let layout_info = get_texture_layout_info(desc.format, mip_width, mip_height);
                assert_eq!(layout_info.pitch, row_pitch);
                assert_eq!(layout_info.slice_pitch, slice_pitch);

                let scratch_allocation = {
                    let mut frames = self.frames.write().unwrap();
                    let frame_index = frames.frame_index;
                    let allocator = &mut frames.frames[frame_index].linear_allocator;
                    let allocation = allocator.write().unwrap().allocate(size, None);
                    allocation
                };

                let src_data = &initial_data.data[0..slice_pitch as usize];
                let mut data_slice = unsafe {
                    ash::util::Align::new(
                        scratch_allocation.address as *mut ::std::ffi::c_void,
                        align_of::<u8>() as u64,
                        //size as u64,
                        slice_pitch as u64,
                    )
                };
                data_slice.copy_from_slice(&src_data);

                let copy_region = ash::vk::BufferImageCopy::builder()
                    .buffer_row_length(row_pitch)
                    .buffer_offset(scratch_allocation.offset as u64)
                    .buffer_image_height(mip_height)
                    .image_offset(ash::vk::Offset3D { x: 0, y: 0, z: 0 })
                    .image_extent(ash::vk::Extent3D {
                        width: mip_width,
                        height: mip_height,
                        depth: mip_depth,
                    })
                    .image_subresource(ash::vk::ImageSubresourceLayers {
                        aspect_mask: ash::vk::ImageAspectFlags::COLOR,
                        base_array_layer: slice_index, // TODO
                        layer_count: 1,
                        mip_level: mip_index,
                    })
                    .build();

                unsafe {
                    vk_device.cmd_copy_buffer_to_image(
                        *transfer_list,
                        scratch_allocation.buffer,
                        image,
                        ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        &[copy_region],
                    );
                }
            }
        } else {
            if channel_format_has_depth(desc.format.into()) {
                unsafe {
                    vk_device.cmd_clear_depth_stencil_image(
                        *transfer_list,
                        image,
                        ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        &ash::vk::ClearDepthStencilValue {
                            depth: 0f32,
                            stencil: 0,
                        },
                        &[image_range],
                    );
                }
            } else {
                unsafe {
                    vk_device.cmd_clear_color_image(
                        *transfer_list,
                        image,
                        ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        &ash::vk::ClearColorValue {
                            float32: [0.0, 0.0, 0.0, 0.0],
                        },
                        &[image_range],
                    );
                }
            }
        }

        vk_sync::cmd::pipeline_barrier(
            &vk_device.fp_v1_0(),
            *transfer_list,
            None,
            &[],
            &[post_write_barrier],
        );

        self.present_command_list.borrow_mut().close()?;
        if let Some(ref queue) = self.get_universal_queue() {
            self.present_command_list.borrow_mut().submit(
                queue.clone(),
                &[],
                &[],
                None,
                ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            )?;
        }

        let resource: Arc<RwLock<Box<RenderResourceBase>>> =
            Arc::new(RwLock::new(Box::new(RenderTextureVk {
                name: debug_name.to_string().into(),
                desc: desc.clone(),
                supported_states,
                default_state,
                image,
                allocation: Some(allocation),
            })));

        self.storage.put(handle, resource)?;
        Ok(())
    }

    fn create_sampler_state(
        &self,
        handle: RenderResourceHandle,
        desc: &RenderSamplerState,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        info!("Creating sampler: {}, {:?}", debug_name, desc);
        let device = Arc::clone(&self.logical_device);
        let raw_device = device.device();

        let create_info = make_vulkan_sampler(desc);
        let sampler = unsafe { raw_device.create_sampler(&create_info, None).unwrap() };

        let resource: Arc<RwLock<Box<RenderResourceBase>>> =
            Arc::new(RwLock::new(Box::new(RenderSamplerStateVk {
                name: debug_name.to_string().into(),
                sampler,
            })));

        self.storage.put(handle, resource)?;
        Ok(())
    }

    fn create_shader(
        &self,
        handle: RenderResourceHandle,
        desc: &RenderShaderDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        info!("Creating shader: {}, {:?}", debug_name, desc.shader_type);
        let device = Arc::clone(&self.logical_device);
        let raw_device = device.device();

        let mut remaps: Vec<BindingSetRemap> = Vec::new();
        match spirv_reflect::ShaderModule::load_u8_data(&desc.shader_data) {
            Ok(mut reflect_module) => {
                let descriptor_sets = reflect_module.enumerate_descriptor_sets(None).unwrap();
                for set_index in 0..descriptor_sets.len() {
                    let set = &descriptor_sets[set_index];
                    for binding_index in 0..set.bindings.len() {
                        let binding = &set.bindings[binding_index];
                        assert_ne!(
                            binding.resource_type,
                            spirv_reflect::types::resource::ReflectResourceType::Undefined
                        );
                        match binding.resource_type {
							spirv_reflect::types::resource::ReflectResourceType::ConstantBufferView => {
								assert_eq!(binding.binding, 0);
								remaps.push(BindingSetRemap {
									binding_index: binding_index as u32,
									old_binding: binding.binding,
									new_binding: binding.binding + CBV_OFFSET + set_index as u32,
									old_set: set_index as u32,
									new_set: SET_OFFSET + set_index as u32,
								});
								assert!(SET_OFFSET as usize + set_index >= descriptor_sets.len()); // Make sure we don't use more spaces/sets than 5
							},
							spirv_reflect::types::resource::ReflectResourceType::ShaderResourceView => {
								remaps.push(BindingSetRemap {
									binding_index: binding_index as u32,
									old_binding: binding.binding,
									new_binding: binding.binding + SRV_OFFSET,
									old_set: set_index as u32,
									new_set: ARG_OFFSET + set_index as u32,
								});
							},
							spirv_reflect::types::resource::ReflectResourceType::Sampler => {
								remaps.push(BindingSetRemap {
									binding_index: binding_index as u32,
									old_binding: binding.binding,
									new_binding: binding.binding + SMP_OFFSET,
									old_set: set_index as u32,
									new_set: ARG_OFFSET + set_index as u32,
								});
							},
							spirv_reflect::types::resource::ReflectResourceType::UnorderedAccessView => {
								remaps.push(BindingSetRemap {
									binding_index: binding_index as u32,
									old_binding: binding.binding,
									new_binding: binding.binding + UAV_OFFSET,
									old_set: set_index as u32,
									new_set: ARG_OFFSET + set_index as u32,
								});
							},
							_ => unimplemented!(),
						}
                    }

                    for remap in &remaps {
                        let binding = &set.bindings[remap.binding_index as usize];
                        match reflect_module.change_descriptor_binding_numbers(
                            binding,
                            remap.new_binding,
                            Some(remap.new_set),
                        ) {
                            Ok(_) => {}
                            Err(err) => {
                                return Err(Error::backend(format!(
                                    "failed to patch descriptor binding - {:?}",
                                    err
                                )));
                            }
                        }
                    }
                }

                // Create descriptor set layouts
                let descriptor_sets = reflect_module.enumerate_descriptor_sets(None).unwrap();
                let mut set_layouts: Vec<(
                    u32, /* set index */
                    Vec<ash::vk::DescriptorSetLayoutBinding>,
                )> = Vec::with_capacity(descriptor_sets.len());
                for set_index in 0..descriptor_sets.len() {
                    let reflected_set = &descriptor_sets[set_index];
                    let mut layout_bindings: Vec<ash::vk::DescriptorSetLayoutBinding> =
                        Vec::with_capacity(reflected_set.bindings.len());
                    for binding_index in 0..reflected_set.bindings.len() {
                        let reflected_binding = &reflected_set.bindings[binding_index];
                        let mut layout_binding = ash::vk::DescriptorSetLayoutBinding::default();
                        layout_binding.binding = reflected_binding.binding;
                        layout_binding.descriptor_type = match reflected_binding.descriptor_type {
                            spirv_reflect::types::ReflectDescriptorType::Sampler => {
                                ash::vk::DescriptorType::SAMPLER
                            }
                            spirv_reflect::types::ReflectDescriptorType::CombinedImageSampler => {
                                ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER
                            }
                            spirv_reflect::types::ReflectDescriptorType::SampledImage => {
                                ash::vk::DescriptorType::SAMPLED_IMAGE
                            }
                            spirv_reflect::types::ReflectDescriptorType::StorageImage => {
                                ash::vk::DescriptorType::STORAGE_IMAGE
                            }
                            spirv_reflect::types::ReflectDescriptorType::UniformTexelBuffer => {
                                ash::vk::DescriptorType::UNIFORM_TEXEL_BUFFER
                            }
                            spirv_reflect::types::ReflectDescriptorType::StorageTexelBuffer => {
                                ash::vk::DescriptorType::STORAGE_TEXEL_BUFFER
                            }
                            spirv_reflect::types::ReflectDescriptorType::UniformBuffer => {
                                ash::vk::DescriptorType::UNIFORM_BUFFER
                            }
                            spirv_reflect::types::ReflectDescriptorType::StorageBuffer => {
                                ash::vk::DescriptorType::STORAGE_BUFFER
                            }
                            spirv_reflect::types::ReflectDescriptorType::UniformBufferDynamic => {
                                ash::vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC
                            }
                            spirv_reflect::types::ReflectDescriptorType::StorageBufferDynamic => {
                                ash::vk::DescriptorType::STORAGE_BUFFER_DYNAMIC
                            }
                            spirv_reflect::types::ReflectDescriptorType::InputAttachment => {
                                ash::vk::DescriptorType::INPUT_ATTACHMENT
                            }
                            _ => unimplemented!(),
                        };

                        layout_binding.descriptor_count = 1;
                        for dim in &reflected_binding.array.dims {
                            layout_binding.descriptor_count *= dim;
                        }

                        let shader_stage = reflect_module.get_shader_stage();
                        if shader_stage
                            .contains(spirv_reflect::types::ReflectShaderStageFlags::VERTEX)
                        {
                            layout_binding.stage_flags |= ash::vk::ShaderStageFlags::VERTEX;
                        }

                        if shader_stage.contains(
                            spirv_reflect::types::ReflectShaderStageFlags::TESSELLATION_CONTROL,
                        ) {
                            layout_binding.stage_flags |=
                                ash::vk::ShaderStageFlags::TESSELLATION_CONTROL;
                        }

                        if shader_stage.contains(
                            spirv_reflect::types::ReflectShaderStageFlags::TESSELLATION_EVALUATION,
                        ) {
                            layout_binding.stage_flags |=
                                ash::vk::ShaderStageFlags::TESSELLATION_EVALUATION;
                        }

                        if shader_stage
                            .contains(spirv_reflect::types::ReflectShaderStageFlags::GEOMETRY)
                        {
                            layout_binding.stage_flags |= ash::vk::ShaderStageFlags::GEOMETRY;
                        }

                        if shader_stage
                            .contains(spirv_reflect::types::ReflectShaderStageFlags::FRAGMENT)
                        {
                            layout_binding.stage_flags |= ash::vk::ShaderStageFlags::FRAGMENT;
                        }

                        if shader_stage
                            .contains(spirv_reflect::types::ReflectShaderStageFlags::COMPUTE)
                        {
                            layout_binding.stage_flags |= ash::vk::ShaderStageFlags::COMPUTE;
                        }
                        layout_bindings.push(layout_binding);
                    }
                    set_layouts.push((reflected_set.set, layout_bindings));
                }

                let patched_spv = reflect_module.get_code();

                let shader_info = ash::vk::ShaderModuleCreateInfo {
                    flags: Default::default(),
                    code_size: patched_spv.len() * 4, // in bytes
                    p_code: patched_spv.as_ptr() as *const u32,
                    ..Default::default()
                };

                let entry_point_name = reflect_module.get_entry_point_name();

                let shader_module = unsafe { raw_device.create_shader_module(&shader_info, None) };
                match shader_module {
                    Ok(module) => {
                        let resource: Arc<RwLock<Box<RenderResourceBase>>> =
                            Arc::new(RwLock::new(Box::new(RenderShaderVk {
                                name: debug_name.to_string().into(),
                                module,
                                entry_point: std::ffi::CString::new(entry_point_name).unwrap(),
                                set_layouts,
                            })));

                        self.storage.put(handle, resource)?;
                        Ok(())
                    }
                    Err(err) => Err(Error::backend(format!(
                        "failed to create shader - {:?}",
                        err
                    ))),
                }
            }
            Err(err) => Err(Error::backend(format!(
                "failed to parse shader - {:?}",
                err
            ))),
        }
    }

    fn create_shader_views(
        &self,
        handle: RenderResourceHandle,
        desc: &RenderShaderViewsDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        info!("Creating shader views: {}, {:?}", debug_name, desc);
        let device = Arc::clone(&self.logical_device);
        let raw_device = device.device();

        let mut srvs: Vec<ShaderResourceViewBinding> =
            Vec::with_capacity(desc.shader_resource_views.len());

        let mut uavs: Vec<UnorderedAccessViewBinding> =
            Vec::with_capacity(desc.unordered_access_views.len());

        for srv in &desc.shader_resource_views {
            let format = convert_format(srv.base.format, false /* typeless */);
            let (buffer_view, image_view) = match srv.base.resource.get_type() {
                RenderResourceType::Texture => {
                    let resource_lock = self.storage.get(srv.base.resource)?;
                    let resource = resource_lock.read().unwrap();
                    let texture = resource.downcast_ref::<RenderTextureVk>().unwrap();
                    let mut create_info = ash::vk::ImageViewCreateInfo::builder()
                        .format(format)
                        .image(texture.image)
                        .components(ash::vk::ComponentMapping {
                            r: ash::vk::ComponentSwizzle::R,
                            g: ash::vk::ComponentSwizzle::G,
                            b: ash::vk::ComponentSwizzle::B,
                            a: ash::vk::ComponentSwizzle::A,
                        })
                        .view_type(convert_view_dimension_to_view_type(srv.base.dimension))
                        .subresource_range(ash::vk::ImageSubresourceRange {
                            aspect_mask: get_image_aspect_flags(
                                srv.base.format,
                                true, /* ignore stencil */
                            ),
                            base_mip_level: srv.most_detailed_mip_first_element,
                            level_count: std::cmp::max(srv.mip_levels_element_count, 1),
                            base_array_layer: match texture.desc.texture_type {
                                RenderTextureType::Cube | RenderTextureType::CubeArray => {
                                    srv.first_array_slice * 6
                                }
                                _ => srv.first_array_slice,
                            },
                            layer_count: match texture.desc.texture_type {
                                RenderTextureType::Cube | RenderTextureType::CubeArray => {
                                    std::cmp::max(srv.array_size, 1) * 6
                                }
                                _ => std::cmp::max(srv.array_size, 1),
                            },
                        })
                        .build();

                    // TODO: Temporary hack until a better solution is developed
                    if texture.desc.format == RenderFormat::D32Float
                        && create_info.format == ash::vk::Format::R32_SFLOAT
                    {
                        create_info.format = ash::vk::Format::D32_SFLOAT;
                        create_info.subresource_range.aspect_mask =
                            ash::vk::ImageAspectFlags::DEPTH;
                    }

                    let image_view =
                        unsafe { raw_device.create_image_view(&create_info, None).unwrap() };
                    (ash::vk::BufferView::null(), image_view)
                }
                RenderResourceType::Buffer => {
                    let resource_lock = self.storage.get(srv.base.resource)?;
                    let resource = resource_lock.read().unwrap();
                    let buffer = resource.downcast_ref::<RenderBufferVk>().unwrap();
                    let buffer_view = match format {
                        ash::vk::Format::UNDEFINED => ash::vk::BufferView::null(),
                        _ => {
                            // Typed
                            let create_info = ash::vk::BufferViewCreateInfo::builder()
                                .buffer(buffer.buffer)
                                .offset(0)
                                .range(ash::vk::WHOLE_SIZE)
                                .format(format)
                                .build();
                            unsafe { raw_device.create_buffer_view(&create_info, None).unwrap() }
                        }
                    };
                    (buffer_view, ash::vk::ImageView::null())
                }
                RenderResourceType::RayTracingAcceleration => {
                    // TODO: Ray tracing support
                    unimplemented!();
                }
                _ => {
                    unimplemented!();
                }
            };

            srvs.push(ShaderResourceViewBinding {
                desc: srv.clone(),
                buffer_view,
                image_view,
            });
        }

        for uav in &desc.unordered_access_views {
            let format = convert_format(uav.base.format, false /* typeless */);
            let (buffer_view, image_view) = match uav.base.resource.get_type() {
                RenderResourceType::Texture => {
                    let resource_lock = self.storage.get(uav.base.resource)?;
                    let resource = resource_lock.read().unwrap();
                    let texture = resource.downcast_ref::<RenderTextureVk>().unwrap();
                    let mut create_info = ash::vk::ImageViewCreateInfo::builder()
                        .format(format)
                        .image(texture.image)
                        .components(ash::vk::ComponentMapping {
                            r: ash::vk::ComponentSwizzle::R,
                            g: ash::vk::ComponentSwizzle::G,
                            b: ash::vk::ComponentSwizzle::B,
                            a: ash::vk::ComponentSwizzle::A,
                        })
                        .view_type(convert_view_dimension_to_view_type(uav.base.dimension))
                        .subresource_range(ash::vk::ImageSubresourceRange {
                            aspect_mask: get_image_aspect_flags(
                                uav.base.format,
                                true, /* ignore stencil */
                            ),
                            base_mip_level: uav.mip_slice_first_element,
                            level_count: std::cmp::max(
                                uav.first_array_slice_first_w_slice_element_count,
                                1,
                            ),
                            base_array_layer: match texture.desc.texture_type {
                                RenderTextureType::Cube | RenderTextureType::CubeArray => {
                                    uav.array_size_plane_slice_w_size * 6
                                }
                                _ => uav.array_size_plane_slice_w_size,
                            },
                            layer_count: match texture.desc.texture_type {
                                RenderTextureType::Cube | RenderTextureType::CubeArray => {
                                    /*std::cmp::max(uav.array_size, 1) * */
                                    6
                                }
                                _ =>
                                /*std::cmp::max(uav.array_size, 1) */
                                {
                                    1
                                }
                            },
                        })
                        .build();

                    // TODO: Temporary hack until a better solution is developed
                    if texture.desc.format == RenderFormat::D32Float
                        && create_info.format == ash::vk::Format::R32_SFLOAT
                    {
                        create_info.format = ash::vk::Format::D32_SFLOAT;
                        create_info.subresource_range.aspect_mask =
                            ash::vk::ImageAspectFlags::DEPTH;
                    }

                    let image_view =
                        unsafe { raw_device.create_image_view(&create_info, None).unwrap() };
                    (ash::vk::BufferView::null(), image_view)
                }
                RenderResourceType::Buffer => {
                    let resource_lock = self.storage.get(uav.base.resource)?;
                    let resource = resource_lock.read().unwrap();
                    let buffer = resource.downcast_ref::<RenderBufferVk>().unwrap();
                    let buffer_view = match format {
                        ash::vk::Format::UNDEFINED => ash::vk::BufferView::null(),
                        _ => {
                            // Typed
                            let create_info = ash::vk::BufferViewCreateInfo::builder()
                                .buffer(buffer.buffer)
                                .offset(0)
                                .range(ash::vk::WHOLE_SIZE)
                                .format(format)
                                .build();
                            unsafe { raw_device.create_buffer_view(&create_info, None).unwrap() }
                        }
                    };
                    (buffer_view, ash::vk::ImageView::null())
                }
                RenderResourceType::RayTracingAcceleration => {
                    // TODO: Ray tracing support
                    unimplemented!();
                }
                _ => {
                    unimplemented!();
                }
            };

            uavs.push(UnorderedAccessViewBinding {
                desc: uav.clone(),
                buffer_view,
                image_view,
            });
        }

        let resource: Arc<RwLock<Box<RenderResourceBase>>> =
            Arc::new(RwLock::new(Box::new(RenderShaderViewsVk {
                name: debug_name.to_string().into(),
                srvs,
                uavs,
                cached_descriptor_sets: Vec::new(),
            })));

        self.storage.put(handle, resource)?;
        Ok(())
    }

    // Ray tracing features are only supported on some devices
    fn create_ray_tracing_program(
        &self,
        handle: RenderResourceHandle,
        desc: &RayTracingProgramDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        info!("Creating ray tracing program: {}, {:?}", debug_name, desc);
        unimplemented!()
    }

    fn create_ray_tracing_geometry(
        &self,
        handle: RenderResourceHandle,
        desc: &RayTracingGeometryDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        info!("Creating ray tracing geometry: {}, {:?}", debug_name, desc);
        unimplemented!()
    }

    fn create_ray_tracing_top_acceleration(
        &self,
        handle: RenderResourceHandle,
        desc: &RayTracingTopAccelerationDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        info!(
            "Creating ray tracing top acceleration: {}, {:?}",
            debug_name, desc
        );

        unimplemented!()
    }

    fn create_ray_tracing_bottom_acceleration(
        &self,
        handle: RenderResourceHandle,
        desc: &RayTracingBottomAccelerationDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        info!(
            "Creating ray tracing bottom acceleration: {}, {:?}",
            debug_name, desc
        );

        unimplemented!()
    }

    fn create_ray_tracing_pipeline_state(
        &self,
        handle: RenderResourceHandle,
        desc: &RayTracingPipelineStateDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        info!(
            "Creating ray tracing pipeline state: {}, {:?}",
            debug_name, desc
        );

        unimplemented!()
    }

    fn create_ray_tracing_shader_table(
        &self,
        handle: RenderResourceHandle,
        desc: &RayTracingShaderTableDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        info!(
            "Creating ray tracing shader table: {}, {:?}",
            debug_name, desc
        );

        unimplemented!()
    }

    fn create_graphics_pipeline_state(
        &self,
        handle: RenderResourceHandle,
        desc: &RenderGraphicsPipelineStateDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        info!(
            "Creating graphics pipeline state: {}, {:?}",
            debug_name, desc
        );

        let device = Arc::clone(&self.logical_device);
        let raw_device = device.device();

        let mut data = RenderPipelineLayoutVk::default();

        // Add in static/immutable samplers
        // Also merge these in? Could allow for mixing immutable vs mutable, too
        data.static_samplers
            .reserve(desc.shader_signature.static_sampler_count as usize);
        data.sampler_layouts
            .reserve(desc.shader_signature.static_sampler_count as usize);
        for sampler_index in 0..desc.shader_signature.static_sampler_count {
            let sampler_state = &desc.shader_signature.static_samplers[sampler_index as usize];
            let sampler_info = make_vulkan_sampler(sampler_state);
            let sampler = unsafe { raw_device.create_sampler(&sampler_info, None).unwrap() };
            let sampler_binding = ash::vk::DescriptorSetLayoutBinding::builder()
                .stage_flags(ash::vk::ShaderStageFlags::ALL_GRAPHICS)
                .descriptor_type(ash::vk::DescriptorType::SAMPLER)
                .descriptor_count(1)
                .binding(SMP_OFFSET + sampler_index)
                .immutable_samplers(&[sampler])
                .build();
            data.sampler_layouts.push(sampler_binding);
            data.static_samplers.push(sampler);
        }

        let mut stage_create_info: Vec<ash::vk::PipelineShaderStageCreateInfo> = Vec::new();

        // Merge descriptor set layouts since the bindings
        // should all align due to explicit registers
        for stage_index in 0..MAX_SHADER_TYPE {
            let stage = RenderShaderType::from_u32(stage_index as u32).unwrap();
            let non_graphics = match stage {
                RenderShaderType::Vertex
                | RenderShaderType::Geometry
                | RenderShaderType::Hull
                | RenderShaderType::Domain
                | RenderShaderType::Pixel => false,
                _ => true,
            };

            if non_graphics {
                // Non-graphics shader type - ignore!
                continue;
            }

            let shader_handle = desc.shaders[stage_index];
            if shader_handle != RenderResourceHandle::default() {
                let resource_lock = self.storage.get(shader_handle)?;
                let resource = resource_lock.read().unwrap();
                let shader = resource.downcast_ref::<RenderShaderVk>().unwrap();

                merge_descriptor_set_layouts(shader, &mut data.combined_layouts);

                let stage_info = ash::vk::PipelineShaderStageCreateInfo::builder()
                    .stage(get_shader_stage(stage))
                    .module(shader.module)
                    .name(&shader.entry_point)
                    .build();
                stage_create_info.push(stage_info);
            }
        }

        // Make the set indices go in 0...N order
        data.combined_layouts
            .sort_by(|ref a, ref b| a.set_index.cmp(&b.set_index));

        // Create descriptor set layout objects
        for layout in &mut data.combined_layouts {
            // Add in global static sampler bindings
            for sampler_binding in &data.sampler_layouts {
                layout.bindings.push(*sampler_binding);
            }
        }

        let mut descriptor_layouts: Vec<ash::vk::DescriptorSetLayout> = Vec::new();
        descriptor_layouts.resize(
            MAX_SHADER_PARAMETERS + MAX_SHADER_PARAMETERS,
            ash::vk::DescriptorSetLayout::null(),
        );

        for index in 0..data.combined_layouts.len() {
            let mut combined_layout = &mut data.combined_layouts[index];
            let create_info = ash::vk::DescriptorSetLayoutCreateInfo::builder()
                .bindings(&combined_layout.bindings)
                .build();

            combined_layout.layout = unsafe {
                raw_device
                    .create_descriptor_set_layout(&create_info, None)
                    .unwrap()
            };

            assert_eq!(
                descriptor_layouts[combined_layout.set_index as usize],
                ash::vk::DescriptorSetLayout::null()
            );
            descriptor_layouts[combined_layout.set_index as usize] = combined_layout.layout;
            for binding in &combined_layout.bindings {
                tally_descriptor_pool_sizes(&mut data.pool_sizes, binding.descriptor_type);
            }
        }

        for index in 0..descriptor_layouts.len() {
            if index >= MAX_SHADER_PARAMETERS {
                descriptor_layouts[index] =
                    self.cbuffer_descriptor_set_layouts[index - MAX_SHADER_PARAMETERS];
            } else if descriptor_layouts[index] == ash::vk::DescriptorSetLayout::null() {
                descriptor_layouts[index] = self.empty_descriptor_set_layout;
            }
        }

        // Create pipeline layout
        let pipeline_layout_create_info = ash::vk::PipelineLayoutCreateInfo::builder()
            .push_constant_ranges(&[]) // TODO: Add support
            .set_layouts(&descriptor_layouts)
            .build();
        data.pipeline_layout = unsafe {
            raw_device
                .create_pipeline_layout(&pipeline_layout_create_info, None)
                .unwrap()
        };

        // Vertex Input
        let mut vertex_bindings: Vec<ash::vk::VertexInputBindingDescription> = Vec::new();
        let mut vertex_attributes: Vec<ash::vk::VertexInputAttributeDescription> = Vec::new();

        // Vulkan "binding" ~= stream
        for stream_index in 0..MAX_VERTEX_STREAMS {
            let stream_stride = desc.vertex_buffer_strides[stream_index];
            if stream_stride > 0 {
                let binding_desc = ash::vk::VertexInputBindingDescription::builder()
                    .binding(stream_index as u32)
                    .stride(stream_stride)
                    .input_rate(ash::vk::VertexInputRate::VERTEX) // TODO: No backends support per instance
                    .build();
                vertex_bindings.push(binding_desc);
            }
        }

        // Vulkan attribute ~= element
        for element_index in 0..desc.vertex_element_count {
            let element = &desc.vertex_elements[element_index as usize];
            let attrib_desc = ash::vk::VertexInputAttributeDescription::builder()
                .binding(element.stream)
                .offset(element.offset)
                .format(convert_format(element.format, false /* typeless */))
                .location(element_index)
                .build();
            vertex_attributes.push(attrib_desc);
        }

        let vertex_input = ash::vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&vertex_bindings)
            .vertex_attribute_descriptions(&vertex_attributes)
            .build();

        // Input Assembly
        let input_assembly = ash::vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(get_primitive_topology(desc.primitive_type))
            .primitive_restart_enable(false)
            .build();

        // Tessellation
        // TODO: Unimplemented
        let _tessellation = ash::vk::PipelineTessellationStateCreateInfo::builder()
            .patch_control_points(3) // TODO: Configurable for PN-AEN?
            .build();

        // Viewport & Scissor
        let viewport = ash::vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&[ash::vk::Viewport::default()]) // dynamic viewport state is used
            .scissors(&[ash::vk::Rect2D::default()]) // dynamic scissor state is used
            .build();

        // Rasterization
        let rasterization = ash::vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(desc.render_state.depth_clamp)
            .rasterizer_discard_enable(false)
            .polygon_mode(get_polygon_mode(desc.render_state.fill_mode))
            .cull_mode(get_cull_mode(desc.render_state.cull_mode))
            .front_face(ash::vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(desc.render_state.depth_bias != 0f32)
            .depth_bias_constant_factor(desc.render_state.depth_bias)
            .depth_bias_clamp(desc.render_state.depth_bias) // (render_state.depth_bias * 0xffffff) as u32 // TODO: Use depth format
            .depth_bias_slope_factor(desc.render_state.slope_scaled_depth_bias)
            .line_width(1f32)
            .build();

        // Multi-Sample
        let multi_sample = ash::vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(ash::vk::SampleCountFlags::TYPE_1) // TODO: MSAA support
            .sample_shading_enable(false)
            .min_sample_shading(0f32)
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false)
            .build();

        // Depth / Stencil
        let depth_stencil = ash::vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(desc.render_state.depth_enable)
            .depth_write_enable(desc.render_state.depth_write_mask != 0)
            .depth_compare_op(get_compare_op(desc.render_state.depth_func))
            .stencil_test_enable(desc.render_state.stencil.mode != RenderStencilMode::Disabled) // TODO: Support double sided stencil
            .front(get_stencil_op_state(
                &desc.render_state.stencil.front,
                desc.render_state.stencil.read_mask,
                desc.render_state.stencil.write_mask,
            ))
            .back(get_stencil_op_state(
                &desc.render_state.stencil.back,
                desc.render_state.stencil.read_mask,
                desc.render_state.stencil.write_mask,
            ))
            .depth_bounds_test_enable(false) // TODO: Support
            .build();

        // Color Blend
        let mut color_attachments: Vec<ash::vk::PipelineColorBlendAttachmentState> = Vec::new();
        let color_blend_workaround = true;
        if color_blend_workaround {
            // Workaround this error (it *should* be using the desc. array for validation, not the references) by
            // matching 1:1 with the attachment references. However, this problem is all the way in the VU.
            /*
                Object: 0x0 | vkCreateGraphicsPipelines(): Render pass (0x2a) subpass 0 has colorAttachmentCount of 8 which
                doesn't match the pColorBlendState->attachmentCount of 1. The spec valid usage text states
                'If rasterization is not disabled and the subpass uses color attachments, the attachmentCount member of
                pColorBlendState must be equal to the colorAttachmentCount used to create subpass'
                (https://www.khronos.org/registry/vulkan/specs/1.0/html/vkspec.html#VUID-VkGraphicsPipelineCreateInfo-attachmentCount-00746)
            */
            color_attachments.resize(
                MAX_RENDER_TARGET_COUNT,
                ash::vk::PipelineColorBlendAttachmentState::default(),
            );
        } else {
            color_attachments.reserve(MAX_RENDER_TARGET_COUNT);
        }

        for index in 0..MAX_RENDER_TARGET_COUNT {
            let rtv_format = desc.render_target_formats[index as usize];
            if rtv_format == RenderFormat::Unknown {
                if color_blend_workaround {
                    color_attachments[index] = Default::default();
                    color_attachments[index].blend_enable = ash::vk::FALSE;
                }
                continue;
            }

            let blend_state = &desc.render_state.blend_states[index as usize];
            if !color_blend_workaround {
                color_attachments.push(Default::default());
            }

            let mut attachment = match color_blend_workaround {
                true => &mut color_attachments[index],
                false => {
                    let count = color_attachments.len();
                    &mut color_attachments[count - 1]
                }
            };

            attachment.blend_enable = bool_to_vk(blend_state.blend_enable);
            attachment.src_color_blend_factor = get_blend_factor(blend_state.source_color);
            attachment.src_alpha_blend_factor = get_blend_factor(blend_state.source_alpha);
            attachment.dst_color_blend_factor = get_blend_factor(blend_state.dest_color);
            attachment.dst_alpha_blend_factor = get_blend_factor(blend_state.dest_alpha);
            attachment.color_blend_op = get_blend_op(blend_state.op_color);
            attachment.alpha_blend_op = get_blend_op(blend_state.op_alpha);

            let color_write_mask = desc.render_target_write_masks[index];
            //color_write_mask = 0xF; // TODO: Why does DX12 ignore the above mask?

            if color_write_mask.contains(RenderWriteMask::RED) {
                attachment.color_write_mask |= ash::vk::ColorComponentFlags::R;
            }

            if color_write_mask.contains(RenderWriteMask::GREEN) {
                attachment.color_write_mask |= ash::vk::ColorComponentFlags::G;
            }

            if color_write_mask.contains(RenderWriteMask::BLUE) {
                attachment.color_write_mask |= ash::vk::ColorComponentFlags::B;
            }

            if color_write_mask.contains(RenderWriteMask::ALPHA) {
                attachment.color_write_mask |= ash::vk::ColorComponentFlags::A;
            }
        }

        let color_blend = ash::vk::PipelineColorBlendStateCreateInfo::builder()
            .attachments(&color_attachments)
            .logic_op(ash::vk::LogicOp::CLEAR)
            .logic_op_enable(false) // TODO: Is this enabled on DX12?
            .build();

        // Dynamic State
        let dynamic_state = ash::vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&[
                ash::vk::DynamicState::VIEWPORT,
                ash::vk::DynamicState::SCISSOR,
                ash::vk::DynamicState::BLEND_CONSTANTS,
                ash::vk::DynamicState::STENCIL_REFERENCE,
                ash::vk::DynamicState::DEPTH_BOUNDS,
            ])
            .build();

        // Layout and Render Pass
        let mut color_attachment_references: Vec<ash::vk::AttachmentReference> = Vec::new();
        color_attachment_references.resize(MAX_RENDER_TARGET_COUNT, Default::default());

        let mut attachment_info: Vec<ash::vk::AttachmentDescription> =
            Vec::with_capacity(MAX_RENDER_TARGET_COUNT);

        for index in 0..MAX_RENDER_TARGET_COUNT {
            let rtv_format = desc.render_target_formats[index as usize];
            if rtv_format == RenderFormat::Unknown {
                color_attachment_references[index as usize] = ash::vk::AttachmentReference {
                    attachment: ash::vk::ATTACHMENT_UNUSED,
                    ..Default::default()
                };
                continue;
            }

            attachment_info.push(
                ash::vk::AttachmentDescription::builder()
                    .format(convert_format(rtv_format, false /* typeless */))
                    .samples(ash::vk::SampleCountFlags::TYPE_1) // TODO: MSAA
                    .initial_layout(ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL) // doesn't matter
                    .final_layout(ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL) // doesn't matter
                    .load_op(ash::vk::AttachmentLoadOp::DONT_CARE)
                    .store_op(ash::vk::AttachmentStoreOp::DONT_CARE)
                    .stencil_load_op(ash::vk::AttachmentLoadOp::DONT_CARE)
                    .stencil_store_op(ash::vk::AttachmentStoreOp::DONT_CARE)
                    .build(),
            );

            color_attachment_references[index as usize] = ash::vk::AttachmentReference {
                attachment: attachment_info.len() as u32 - 1,
                layout: ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL, // doesn't matter
            };
        }

        assert_eq!(attachment_info.len(), desc.render_target_count as usize);

        // TODO: Should also have indirection table to be 100% correct
        /*let mut hasher = meowhash::MeowHasher::new();
        for attachment in &attachment_info {
            hasher.input(&any_as_u8_slice(&attachment));
        }
        let hasher_result = hasher.result();
        let hasher_slice = hasher_result.as_slice();
        assert_eq!(hasher_slice.len(), 8); // make sure u64
        let attachment_hash: u64 =
            unsafe { std::slice::from_raw_parts(hasher_slice.as_ptr() as *const u64, 1) }[0];*/
        let mut hasher = twox_hash::XxHash32::with_seed(0);
        for attachment in &attachment_info {
            hasher.write(&any_as_u8_slice(&attachment));
        }
        let attachment_hash: u64 = hasher.finish();

        let mut render_pass = match self.compat_render_passes.borrow().get(&attachment_hash) {
            Some(render_pass) => render_pass.clone(),
            None => ash::vk::RenderPass::null(),
        };
        if render_pass == ash::vk::RenderPass::null() {
            // TODO: Only format matters, so create a compatible render pass but
            // don't use for rendering. Hash the render passes on the format, etc..
            // and reuse
            // TODO: Shouldn't the depth attachment reference also go into the hash?
            let depth_attachment_reference = match desc.depth_stencil_format {
                RenderFormat::Unknown => ash::vk::AttachmentReference {
                    attachment: ash::vk::ATTACHMENT_UNUSED,
                    ..Default::default()
                },
                _ => {
                    let reference = ash::vk::AttachmentReference {
                        attachment: attachment_info.len() as u32,
                        layout: ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL, // doesn't matter
                    };

                    attachment_info.push(
                        ash::vk::AttachmentDescription::builder()
                            .format(convert_format(
                                desc.depth_stencil_format,
                                false, /* typeless */
                            ))
                            .samples(ash::vk::SampleCountFlags::TYPE_1) // TODO: MSAA
                            .initial_layout(ash::vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL) // doesn't matter
                            .final_layout(ash::vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL) // doesn't matter
                            .load_op(ash::vk::AttachmentLoadOp::DONT_CARE)
                            .store_op(ash::vk::AttachmentStoreOp::DONT_CARE)
                            .stencil_load_op(ash::vk::AttachmentLoadOp::DONT_CARE)
                            .stencil_store_op(ash::vk::AttachmentStoreOp::DONT_CARE)
                            .build(),
                    );

                    reference
                }
            };

            let sub_pass_description = ash::vk::SubpassDescription::builder()
                .pipeline_bind_point(ash::vk::PipelineBindPoint::GRAPHICS)
                .color_attachments(&color_attachment_references)
                .depth_stencil_attachment(&depth_attachment_reference)
                .build();

            let render_pass_info = ash::vk::RenderPassCreateInfo::builder()
                .attachments(&attachment_info)
                .subpasses(&[sub_pass_description])
                .build();

            render_pass = unsafe {
                raw_device
                    .create_render_pass(&render_pass_info, None)
                    .unwrap()
            };
            trace!("Created compatible render pass: {:?}", render_pass_info);
            self.compat_render_passes
                .borrow_mut()
                .insert(attachment_hash, render_pass);
        }

        // Construction
        let pipeline_create_info = ash::vk::GraphicsPipelineCreateInfo::builder()
            .stages(&stage_create_info)
            .layout(data.pipeline_layout)
            .vertex_input_state(&vertex_input)
            .input_assembly_state(&input_assembly)
            // TODO: .tessellation_state(&tessellation)
            .viewport_state(&viewport)
            .rasterization_state(&rasterization)
            .multisample_state(&multi_sample)
            .depth_stencil_state(&depth_stencil)
            .color_blend_state(&color_blend)
            .dynamic_state(&dynamic_state)
            .render_pass(render_pass)
            .build();

        let pipelines = unsafe {
            raw_device
                .create_graphics_pipelines(self.pipeline_cache, &[pipeline_create_info], None)
                .unwrap()
        };
        let pipeline = pipelines[0];

        let resource: Arc<RwLock<Box<RenderResourceBase>>> =
            Arc::new(RwLock::new(Box::new(RenderGraphicsPipelineStateVk {
                name: debug_name.to_string().into(),
                data,
                pipeline,
            })));

        self.storage.put(handle, resource)?;
        Ok(())
    }

    fn create_compute_pipeline_state(
        &self,
        handle: RenderResourceHandle,
        desc: &RenderComputePipelineStateDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        info!(
            "Creating compute pipeline state: {}, {:?}",
            debug_name, desc
        );

        unimplemented!()
    }

    fn create_draw_binding_set(
        &self,
        handle: RenderResourceHandle,
        desc: &RenderDrawBindingSetDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        info!("Creating draw binding set: {}, {:?}", debug_name, desc);

        let (index_buffer, index_buffer_offset, index_buffer_format) = match desc.index_buffer {
            Some(ref binding) => {
                let resource_lock = self.storage.get(binding.resource)?;
                let resource = resource_lock.read().unwrap();
                let buffer = resource.downcast_ref::<RenderBufferVk>().unwrap();
                assert!(buffer
                    .supported_states
                    .contains(RenderResourceStates::INDEX_BUFFER));
                let index_type = match binding.stride {
                    2 => ash::vk::IndexType::UINT16,
                    4 => ash::vk::IndexType::UINT32,
                    _ => unimplemented!(),
                };
                (
                    Some(binding.resource),
                    binding.offset as ash::vk::DeviceSize,
                    index_type,
                )
            }
            None => (Default::default(), 0, ash::vk::IndexType::UINT32),
        };

        let mut vertex_buffers: [Option<RenderResourceHandle>; MAX_VERTEX_STREAMS] =
            Default::default();
        let mut vertex_buffer_offsets: [ash::vk::DeviceSize; MAX_VERTEX_STREAMS] =
            Default::default();

        for stream in 0..MAX_VERTEX_STREAMS {
            match desc.vertex_buffers[stream] {
                Some(ref binding) => {
                    let resource_lock = self.storage.get(binding.resource)?;
                    let resource = resource_lock.read().unwrap();
                    let buffer = resource.downcast_ref::<RenderBufferVk>().unwrap();
                    assert!(buffer
                        .supported_states
                        .contains(RenderResourceStates::VERTEX_AND_CONSTANT_BUFFER));
                    vertex_buffers[stream] = Some(binding.resource);
                    vertex_buffer_offsets[stream] = binding.offset as ash::vk::DeviceSize;
                }
                None => {}
            }
        }

        let resource: Arc<RwLock<Box<RenderResourceBase>>> =
            Arc::new(RwLock::new(Box::new(RenderDrawBindingSetVk {
                name: debug_name.to_string().into(),
                desc: desc.clone(),
                vertex_buffers,
                vertex_buffer_offsets,
                index_buffer,
                index_buffer_offset,
                index_buffer_format,
            })));

        self.storage.put(handle, resource)?;
        Ok(())
    }

    fn create_frame_binding_set(
        &self,
        handle: RenderResourceHandle,
        desc: &RenderFrameBindingSetDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        info!("Creating frame binding set: {}, {:?}", debug_name, desc);
        assert_eq!(handle.get_type(), RenderResourceType::FrameBindingSet);

        let device = Arc::clone(&self.logical_device);
        let raw_device = device.device();

        let mut frame_binding = Box::new(RenderFrameBindingSetVk {
            name: debug_name.to_string().into(),
            desc: desc.clone(),
            render_target_handles: Default::default(),
            render_target_resources: Default::default(),
            depth_stencil_handle: Default::default(),
            depth_stencil_resource: Default::default(),
            image_views: Vec::new(),
            swap_chain: Default::default(),
            frame_buffer_info: ash::vk::FramebufferCreateInfo {
                s_type: ash::vk::StructureType::FRAMEBUFFER_CREATE_INFO,
                p_next: ptr::null(),
                flags: Default::default(),
                render_pass: ash::vk::RenderPass::null(),
                attachment_count: 0,
                p_attachments: ptr::null(),
                width: 0,
                height: 0,
                layers: 1, // If we ever support VS/GS output of viewport/layer index
            },
            render_target_count: 0,
        });

        let mut view_width: u32 = 0;
        let mut view_height: u32 = 0;
        let mut view_mip: u32 = 0;

        for target_index in 0..MAX_RENDER_TARGET_COUNT {
            if let Some(ref render_target) = desc.render_target_views[target_index] {
                assert!(
                    target_index == 0
                        || render_target.base.resource.get_type() == RenderResourceType::Texture
                );
                assert!(target_index == 0 || frame_binding.swap_chain.is_none());
                assert!(target_index == frame_binding.render_target_count as usize);

                frame_binding.render_target_count += 1;

                let resource_lock = self.storage.get(render_target.base.resource)?;
                let resource = resource_lock.read().unwrap();
                let texture = resource.downcast_ref::<RenderTextureVk>().unwrap();

                assert!(texture
                    .supported_states
                    .contains(RenderResourceStates::RENDER_TARGET));
                assert!(view_width == 0 || view_width == texture.desc.width); // RTV and DSV dimensions must all match each other
                assert!(view_height == 0 || view_height == texture.desc.height); // RTV and DSV dimensions must all match each other
                assert!(view_mip == 0 || view_mip == render_target.mip_slice); // RTV and DSV dimensions must all match each other

                view_width = texture.desc.width;
                view_height = texture.desc.height;
                view_mip = render_target.mip_slice;

                frame_binding.render_target_handles[target_index] =
                    Some(render_target.base.resource);
                frame_binding.render_target_resources[target_index] =
                    Some(Arc::clone(&resource_lock));

                let mut view_create_info = ash::vk::ImageViewCreateInfo {
                    s_type: ash::vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                    p_next: ptr::null(),
                    flags: Default::default(),
                    view_type: image::convert_view_dimension_to_view_type(
                        render_target.base.dimension,
                    ),
                    format: convert_format(render_target.base.format, false /* typeless */),
                    components: ash::vk::ComponentMapping {
                        r: ash::vk::ComponentSwizzle::R,
                        g: ash::vk::ComponentSwizzle::G,
                        b: ash::vk::ComponentSwizzle::B,
                        a: ash::vk::ComponentSwizzle::A,
                    },
                    subresource_range: ash::vk::ImageSubresourceRange {
                        aspect_mask: ash::vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1, // Can only render to a single mip at a given time
                        base_array_layer: 0,
                        layer_count: 1, // Can only render to a single face\slice at a given time
                    },
                    image: texture.image,
                };

                match render_target.base.dimension {
                    RenderViewDimension::Tex1d => {
                        view_create_info.subresource_range.base_mip_level = render_target.mip_slice;
                    }
                    RenderViewDimension::Tex1dArray => {
                        view_create_info.subresource_range.base_mip_level = render_target.mip_slice;
                        view_create_info.subresource_range.base_array_layer =
                            render_target.first_array_slice;
                    }
                    RenderViewDimension::Tex2d => {
                        view_create_info.subresource_range.base_mip_level = render_target.mip_slice;
                        //rtvDesc.Texture2D.PlaneSlice = rtv.planeSliceFirstWSlice;
                    }
                    RenderViewDimension::Tex2dArray => {
                        view_create_info.subresource_range.base_mip_level = render_target.mip_slice;
                        view_create_info.subresource_range.base_array_layer =
                            render_target.first_array_slice;
                        //rtvDesc.Texture2DArray.PlaneSlice = rtv.planeSliceFirstWSlice;
                    }
                    RenderViewDimension::Tex3d => {
                        //rtvDesc.Texture3D.FirstWSlice = rtv.planeSliceFirstWSlice;
                        view_create_info.subresource_range.base_mip_level = render_target.mip_slice;
                        //rtvDesc.Texture3D.WSize = rtv.wSize;
                    }
                    _ => unimplemented!(),
                }

                let image_view = unsafe {
                    raw_device
                        .create_image_view(&view_create_info, None)
                        .unwrap()
                };
                frame_binding.image_views.push(image_view);
            }
        }

        if let Some(ref depth_stencil) = desc.depth_stencil_view {
            assert_eq!(
                depth_stencil.base.resource.get_type(),
                RenderResourceType::Texture
            );

            let resource_lock = self.storage.get(depth_stencil.base.resource)?;
            let resource = resource_lock.read().unwrap();
            let texture = resource.downcast_ref::<RenderTextureVk>().unwrap();

            assert!(
                texture
                    .supported_states
                    .contains(RenderResourceStates::DEPTH_WRITE)
                    || texture
                        .supported_states
                        .contains(RenderResourceStates::DEPTH_READ)
            );
            assert!(view_width == 0 || view_width == texture.desc.width); // RTV and DSV dimensions must all match each other
            assert!(view_height == 0 || view_height == texture.desc.height); // RTV and DSV dimensions must all match each other
            assert!(view_mip == 0 || view_mip == depth_stencil.mip_slice); // RTV and DSV dimensions must all match each other

            view_width = texture.desc.width;
            view_height = texture.desc.height;
            view_mip = depth_stencil.mip_slice;

            assert_ne!(depth_stencil.base.format, RenderFormat::D24UnormS8Uint); // Not supported on AMD anymore....

            frame_binding.depth_stencil_handle = Some(depth_stencil.base.resource);
            frame_binding.depth_stencil_resource = Some(Arc::clone(&resource_lock));

            let view_create_info = ash::vk::ImageViewCreateInfo {
                s_type: ash::vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                p_next: ptr::null(),
                flags: Default::default(),
                view_type: image::convert_view_dimension_to_view_type(depth_stencil.base.dimension),
                format: convert_format(depth_stencil.base.format, false /* typeless */),
                components: ash::vk::ComponentMapping {
                    r: ash::vk::ComponentSwizzle::R,
                    g: ash::vk::ComponentSwizzle::G,
                    b: ash::vk::ComponentSwizzle::B,
                    a: ash::vk::ComponentSwizzle::A,
                },
                subresource_range: ash::vk::ImageSubresourceRange {
                    aspect_mask: match depth_stencil.base.format {
                        RenderFormat::D16Unorm | RenderFormat::D32Float => {
                            ash::vk::ImageAspectFlags::DEPTH
                        }
                        RenderFormat::D24UnormS8Uint | RenderFormat::D32FloatS8Uint => {
                            ash::vk::ImageAspectFlags::DEPTH | ash::vk::ImageAspectFlags::STENCIL
                        }
                        _ => unimplemented!(),
                    },
                    base_mip_level: 0,
                    level_count: 1, // Can only render to a single mip at a given time
                    base_array_layer: 0,
                    layer_count: 1, // Can only render to a single face\slice at a given time
                },
                image: texture.image,
            };
        }

        frame_binding.frame_buffer_info.attachment_count = frame_binding.image_views.len() as u32;
        frame_binding.frame_buffer_info.p_attachments = frame_binding.image_views.as_ptr();
        frame_binding.frame_buffer_info.width = view_width >> view_mip;
        frame_binding.frame_buffer_info.height = view_height >> view_mip;

        //info!("Created frame binding: {:?}", frame_binding);

        let resource: Arc<RwLock<Box<RenderResourceBase>>> = Arc::new(RwLock::new(frame_binding));
        self.storage.put(handle, resource)?;
        Ok(())
    }

    fn create_render_pass(
        &self,
        handle: RenderResourceHandle,
        desc: &RenderPassDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        info!("Creating render pass: {}", debug_name);
        //info!("Creating render pass: {}, {:?}", debug_name, desc);
        assert_eq!(handle.get_type(), RenderResourceType::RenderPass);
        assert_eq!(
            desc.frame_binding.get_type(),
            RenderResourceType::FrameBindingSet
        );

        let device = Arc::clone(&self.logical_device);
        let raw_device = device.device();

        let frame_binding_lock = self.storage.get(desc.frame_binding)?;
        let frame_binding_resource = frame_binding_lock.read().unwrap();
        let frame_binding = frame_binding_resource
            .downcast_ref::<RenderFrameBindingSetVk>()
            .unwrap();

        let mut render_pass = Box::new(RenderPassVk {
            name: debug_name.to_string().into(),
            desc: desc.clone(),
            render_pass: ash::vk::RenderPass::null(),
            frame_buffer: ash::vk::Framebuffer::null(),
            frame_binding: Arc::clone(&frame_binding_lock),
            depth_stencil_layout: ash::vk::ImageLayout::UNDEFINED,
            clear_values: Vec::new(),
        });

        // Attachments (color then D/S)
        let unused_attachment = ash::vk::AttachmentReference {
            attachment: ash::vk::ATTACHMENT_UNUSED,
            layout: ash::vk::ImageLayout::UNDEFINED,
        };

        let mut attachments: Vec<ash::vk::AttachmentDescription> =
            Vec::with_capacity(MAX_RENDER_TARGET_COUNT + 1);

        let mut depth_reference = unused_attachment.clone();

        let mut color_references: Vec<ash::vk::AttachmentReference> =
            vec![unused_attachment; MAX_RENDER_TARGET_COUNT];

        let mut clear_values: Vec<ash::vk::ClearValue> =
            Vec::with_capacity(MAX_RENDER_TARGET_COUNT + 1);

        for target_index in 0..MAX_RENDER_TARGET_COUNT {
            let color_format = match frame_binding.desc.render_target_views[target_index] {
                Some(ref view) => view.base.format,
                None => RenderFormat::Unknown,
            };

            if let Some(render_target) = &frame_binding.render_target_resources[target_index] {
                assert_ne!(color_format, RenderFormat::Unknown);
                let texture_lock = Arc::clone(&render_target);
                let texture_resource = texture_lock.read().unwrap();
                let texture = texture_resource.downcast_ref::<RenderTextureVk>().unwrap();

                let target_desc = &desc.render_target_info[target_index];

                attachments.push(ash::vk::AttachmentDescription {
                    format: convert_format(color_format, false /* typeless */),
                    flags: ash::vk::AttachmentDescriptionFlags::empty(),
                    samples: ash::vk::SampleCountFlags::TYPE_1, // get_sample_count_flags(desc.sample_count); // TODO: MSAA
                    load_op: match target_desc.load_op {
                        RenderLoadOp::Discard => ash::vk::AttachmentLoadOp::DONT_CARE,
                        RenderLoadOp::Load => ash::vk::AttachmentLoadOp::LOAD,
                        RenderLoadOp::Clear => ash::vk::AttachmentLoadOp::CLEAR,
                    },
                    store_op: match target_desc.store_op {
                        RenderStoreOp::Discard => ash::vk::AttachmentStoreOp::DONT_CARE,
                        RenderStoreOp::Store => ash::vk::AttachmentStoreOp::STORE,
                    },
                    stencil_load_op: ash::vk::AttachmentLoadOp::DONT_CARE,
                    stencil_store_op: ash::vk::AttachmentStoreOp::DONT_CARE,
                    initial_layout: ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                    final_layout: ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                });

                clear_values.push(ash::vk::ClearValue {
                    color: ash::vk::ClearColorValue {
                        float32: target_desc.clear_color,
                    },
                });

                color_references[target_index] = ash::vk::AttachmentReference {
                    attachment: target_index as u32,
                    layout: ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                };
            } else {
                assert_eq!(color_format, RenderFormat::Unknown);
            }
        }

        let depth_stencil_format = match frame_binding.desc.depth_stencil_view {
            Some(ref view) => view.base.format,
            None => RenderFormat::Unknown,
        };

        if let Some(ref depth_stencil) = frame_binding.depth_stencil_resource {
            assert_ne!(depth_stencil_format, RenderFormat::Unknown);

            let (read_only_depth, read_only_stencil) = match frame_binding.desc.depth_stencil_view {
                Some(ref view) => (
                    view.flags
                        .contains(RenderDepthStencilViewFlags::READ_ONLY_DEPTH),
                    view.flags
                        .contains(RenderDepthStencilViewFlags::READ_ONLY_STENCIL),
                ),
                None => (false, false),
            };

            //let default_layout = ash::vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL;

            render_pass.depth_stencil_layout = if format_has_stencil(depth_stencil_format) {
                if read_only_depth && read_only_stencil {
                    ash::vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL
                } else if read_only_depth {
                    ash::vk::ImageLayout::DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL
                } else if read_only_stencil {
                    ash::vk::ImageLayout::DEPTH_ATTACHMENT_STENCIL_READ_ONLY_OPTIMAL
                } else {
                    ash::vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
                }
            } else {
                match read_only_depth {
                    true => ash::vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL,
                    false => ash::vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                }
            };

            attachments.push(ash::vk::AttachmentDescription {
                format: convert_format(depth_stencil_format, false /* typeless */),
                flags: ash::vk::AttachmentDescriptionFlags::empty(),
                samples: ash::vk::SampleCountFlags::TYPE_1, // get_sample_count_flags(desc.sample_count); // TODO: MSAA
                load_op: if read_only_depth {
                    ash::vk::AttachmentLoadOp::LOAD
                } else {
                    match desc.depth_stencil_target_info.load_op {
                        RenderLoadOp::Discard => ash::vk::AttachmentLoadOp::DONT_CARE,
                        RenderLoadOp::Load => ash::vk::AttachmentLoadOp::LOAD,
                        RenderLoadOp::Clear => ash::vk::AttachmentLoadOp::CLEAR,
                    }
                },
                store_op: match desc.depth_stencil_target_info.store_op {
                    RenderStoreOp::Discard => ash::vk::AttachmentStoreOp::DONT_CARE,
                    RenderStoreOp::Store => ash::vk::AttachmentStoreOp::STORE,
                },
                stencil_load_op: if read_only_stencil {
                    ash::vk::AttachmentLoadOp::LOAD
                } else {
                    match desc.depth_stencil_target_info.load_op {
                        RenderLoadOp::Discard => ash::vk::AttachmentLoadOp::DONT_CARE,
                        RenderLoadOp::Load => ash::vk::AttachmentLoadOp::LOAD,
                        RenderLoadOp::Clear => ash::vk::AttachmentLoadOp::CLEAR,
                    }
                },
                stencil_store_op: match desc.depth_stencil_target_info.store_op {
                    RenderStoreOp::Discard => ash::vk::AttachmentStoreOp::DONT_CARE,
                    RenderStoreOp::Store => ash::vk::AttachmentStoreOp::STORE,
                },
                initial_layout: render_pass.depth_stencil_layout,
                final_layout: render_pass.depth_stencil_layout,
            });

            clear_values.push(ash::vk::ClearValue {
                depth_stencil: ash::vk::ClearDepthStencilValue {
                    depth: desc.depth_stencil_target_info.clear_depth,
                    stencil: desc.depth_stencil_target_info.clear_stencil as u32,
                },
            });

            depth_reference = ash::vk::AttachmentReference {
                attachment: (attachments.len() - 1) as u32,
                layout: render_pass.depth_stencil_layout,
            };
        }

        // Sub-pass (only a single one currently)
        // NOTE: We currently never use input attachments and preserve attachments,
        // as those are only useful with multiple sub-passes.
        let subpass = ash::vk::SubpassDescription {
            color_attachment_count: color_references.len() as u32,
            p_color_attachments: color_references.as_ptr(),
            p_depth_stencil_attachment: &depth_reference,
            flags: Default::default(),
            pipeline_bind_point: ash::vk::PipelineBindPoint::GRAPHICS,
            input_attachment_count: 0,
            p_input_attachments: ptr::null(),
            p_resolve_attachments: ptr::null(),
            preserve_attachment_count: 0,
            p_preserve_attachments: ptr::null(),
        };

        // Also no subpass dependencies with only one subpass
        let create_info = ash::vk::RenderPassCreateInfo {
            s_type: ash::vk::StructureType::RENDER_PASS_CREATE_INFO,
            flags: Default::default(),
            p_next: ptr::null(),
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            subpass_count: 1,
            p_subpasses: &subpass,
            dependency_count: 0,
            p_dependencies: ptr::null(),
        };

        render_pass.render_pass =
            unsafe { raw_device.create_render_pass(&create_info, None).unwrap() };

        // Annoyingly, Vulkan requires a render pass object when creating a frame buffer.
        let mut frame_buffer_info = frame_binding.frame_buffer_info.clone();
        frame_buffer_info.render_pass = render_pass.render_pass;
        render_pass.frame_buffer = unsafe {
            raw_device
                .create_framebuffer(&frame_buffer_info, None)
                .unwrap()
        };

        render_pass.clear_values = clear_values;

        trace!("Created render pass: {:?}", render_pass);

        let resource: Arc<RwLock<Box<RenderResourceBase>>> = Arc::new(RwLock::new(render_pass));
        self.storage.put(handle, resource)?;
        Ok(())
    }

    fn create_command_list(
        &self,
        handle: RenderResourceHandle,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        info!("Creating command list: {}", debug_name);
        assert_eq!(handle.get_type(), RenderResourceType::CommandList);

        let command_list = Box::new(RenderCommandListVk::new(
            Arc::clone(&self.logical_device),
            self.frames.clone(),
            RenderCommandListType::Universal,
            debug_name.into(),
        ));

        let resource: Arc<RwLock<Box<RenderResourceBase>>> = Arc::new(RwLock::new(command_list));
        self.storage.put(handle, resource)?;
        Ok(())
    }

    fn create_fence(
        &self,
        handle: RenderResourceHandle,
        desc: &RenderFenceDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        info!("Creating fence: {}, {:?}", debug_name, desc);
        let device = Arc::clone(&self.logical_device);
        let raw_device = device.device();

        let create_info = ash::vk::FenceCreateInfo {
            s_type: ash::vk::StructureType::FENCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: ash::vk::FenceCreateFlags::empty(),
        };

        let fence = unsafe {
            raw_device
                .create_fence(&create_info, None)
                .expect("create fence failed.")
        };

        let resource: Arc<RwLock<Box<RenderResourceBase>>> =
            Arc::new(RwLock::new(Box::new(RenderFenceVk {
                name: debug_name.to_string().into(),
                fence,
            })));

        self.storage.put(handle, resource)?;
        Ok(())
    }

    fn create_timing_heap(
        &self,
        handle: RenderResourceHandle,
        desc: &RenderTimingHeapDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        info!("Creating timing heap: {}, {:?}", debug_name, desc);
        unimplemented!()
    }

    // Timing Heap Management
    fn get_timing_frequency(&self) -> Result<f64> {
        let properties = self.physical_device.properties();
        let timestamp_period = properties.limits.timestamp_period as f64;
        // Convert to dx12-standard ticks per second
        let frequency = 1f64 / (timestamp_period * 1e-9);
        Ok(frequency)
    }

    // CommandList Management
    fn submit_command_list(
        &self,
        handle: RenderResourceHandle,
        flush: bool,
        wait_before: Option<&[RenderResourceHandle]>,
        signal_after: Option<RenderResourceHandle>,
    ) -> Result<()> {
        assert_eq!(handle.get_type(), RenderResourceType::CommandList);
        self.flush_transfers();
        let resource_lock = self.storage.get(handle)?;
        let mut resource = resource_lock.write().unwrap();
        let native_command_list = resource.downcast_mut::<RenderCommandListVk>().unwrap();
        if let Some(ref queue) = self.get_list_queue(native_command_list.list_type()) {
            let wait_stage = ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
            native_command_list.submit(queue.clone(), &[], &[], None, wait_stage)?;
            Ok(())
        } else {
            Err(Error::backend("no queue available for command list"))
        }
    }

    fn compile_command_list(
        &self,
        handle: RenderResourceHandle,
        command_list: &RenderCommandList,
    ) -> Result<()> {
        assert_eq!(handle.get_type(), RenderResourceType::CommandList);
        let resource_lock = self.storage.get(handle)?;
        let mut resource = resource_lock.write().unwrap();
        let mut native_command_list = resource.downcast_mut::<RenderCommandListVk>().unwrap();
        if let Some(ref queue) = self.get_list_queue(native_command_list.list_type()) {
            let mut compile_context = RenderCompileContext::new(
                self.logical_device.clone(),
                self.descriptor_cache.clone(),
                self.storage.clone(),
                queue.clone(),
            );
            Ok(compile_context.compile_list(&mut native_command_list, &command_list)?)
        } else {
            Err(Error::backend("no queue available for command list"))
        }
    }

    fn compile_command_lists(
        &self,
        handle: RenderResourceHandle,
        command_lists: &[RenderCommandList],
    ) -> Result<()> {
        assert_eq!(handle.get_type(), RenderResourceType::CommandList);
        let resource_lock = self.storage.get(handle)?;
        let mut resource = resource_lock.write().unwrap();
        let mut native_command_list = resource.downcast_mut::<RenderCommandListVk>().unwrap();
        if let Some(ref queue) = self.get_list_queue(native_command_list.list_type()) {
            let mut compile_context = RenderCompileContext::new(
                self.logical_device.clone(),
                self.descriptor_cache.clone(),
                self.storage.clone(),
                queue.clone(),
            );
            compile_context.begin_compile(&mut native_command_list)?;
            for command_list in command_lists {
                compile_context.compile_list(&mut native_command_list, &command_list)?;
            }
            compile_context.finish_compile(&mut native_command_list)?;
            Ok(())
        } else {
            Err(Error::backend("no queue available for command list"))
        }
    }

    // Present Management
    fn present_swap_chain(
        &mut self,
        swap_chain: RenderResourceHandle,
        source_texture: RenderResourceHandle,
    ) -> Result<()> {
        trace!("Presenting swap chain - {:?}", swap_chain);

        let swap_chain = self.storage.get(swap_chain)?;
        let mut swap_chain = swap_chain.write().unwrap();
        let swap_chain = swap_chain.downcast_mut::<RenderSwapChainVk>().unwrap();

        let present_index = unsafe {
            self.swap_chain_loader.acquire_next_image(
                swap_chain.swap_chain,
                ::std::u64::MAX,
                swap_chain.acquire_image_semaphore,
                ash::vk::Fence::null(),
            )
        };

        match present_index {
            Ok(present_index) => {
                swap_chain.back_buffer_index = present_index.0;

                let src_texture = self.storage.get(source_texture)?;
                let src_texture = src_texture.read().unwrap();
                let src_texture = src_texture.downcast_ref::<RenderTextureVk>().unwrap();
                let dst_texture = &swap_chain.textures[swap_chain.back_buffer_index as usize];
                assert_eq!(src_texture.desc.width, dst_texture.desc.width);
                assert_eq!(src_texture.desc.height, dst_texture.desc.height);

                let vk_device = self.logical_device.device();
                let present_list = self.present_command_list.borrow_mut().open()?;

                let (src_before_access, src_before_layout) =
                    if src_texture.default_state == RenderResourceStates::COMMON {
                        (vk_sync::AccessType::Nothing, vk_sync::ImageLayout::General)
                    } else if src_texture
                        .default_state
                        .contains(RenderResourceStates::RENDER_TARGET)
                    {
                        (
                            vk_sync::AccessType::ColorAttachmentWrite,
                            vk_sync::ImageLayout::Optimal,
                        )
                    } else if src_texture
                        .default_state
                        .contains(RenderResourceStates::DEPTH_WRITE)
                    {
                        // TODO: New depth/stencil read/write combinations
                        (
                            vk_sync::AccessType::DepthStencilAttachmentWrite,
                            vk_sync::ImageLayout::Optimal,
                        )
                    } else if src_texture
                        .default_state
                        .contains(RenderResourceStates::DEPTH_READ)
                    {
                        // TODO: New depth/stencil read/write combinations
                        // VK_IMAGE_LAYOUT_DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL_KHR
                        // VK_IMAGE_LAYOUT_DEPTH_ATTACHMENT_STENCIL_READ_ONLY_OPTIMAL_KHR
                        (
                            vk_sync::AccessType::DepthStencilAttachmentRead,
                            vk_sync::ImageLayout::Optimal,
                        )
                    } else if src_texture.default_state.contains(
                        RenderResourceStates::PIXEL_SHADER_RESOURCE
                            | RenderResourceStates::NON_PIXEL_SHADER_RESOURCE,
                    ) {
                        (
                            vk_sync::AccessType::AnyShaderReadSampledImageOrUniformTexelBuffer,
                            vk_sync::ImageLayout::General,
                        )
                    } else if src_texture
                        .default_state
                        .contains(RenderResourceStates::COPY_DEST)
                    {
                        (
                            vk_sync::AccessType::TransferWrite,
                            vk_sync::ImageLayout::Optimal,
                        )
                    } else if src_texture
                        .default_state
                        .contains(RenderResourceStates::COPY_SOURCE)
                    {
                        (
                            vk_sync::AccessType::TransferRead,
                            vk_sync::ImageLayout::Optimal,
                        )
                    } else {
                        (vk_sync::AccessType::General, vk_sync::ImageLayout::General)
                    };

                // Source Image: <Unknown> -> Copy Source
                let src_to_copy = vk_sync::ImageBarrier {
                    previous_accesses: vec![src_before_access.clone()],
                    next_accesses: vec![vk_sync::AccessType::TransferRead],
                    previous_layout: src_before_layout.clone(),
                    next_layout: vk_sync::ImageLayout::Optimal,
                    discard_contents: false,
                    src_queue_family_index: ash::vk::QUEUE_FAMILY_IGNORED,
                    dst_queue_family_index: ash::vk::QUEUE_FAMILY_IGNORED,
                    image: src_texture.image,
                    range: ash::vk::ImageSubresourceRange {
                        aspect_mask: ash::vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                };
                vk_sync::cmd::pipeline_barrier(
                    &vk_device.fp_v1_0(),
                    *present_list,
                    None,
                    &[],
                    &[src_to_copy],
                );

                // SwapChain Image: Undefined -> Copy Dest
                let swap_to_copy = vk_sync::ImageBarrier {
                    previous_accesses: vec![vk_sync::AccessType::Nothing],
                    next_accesses: vec![vk_sync::AccessType::TransferWrite],
                    previous_layout: vk_sync::ImageLayout::General,
                    next_layout: vk_sync::ImageLayout::Optimal,
                    discard_contents: true,
                    src_queue_family_index: ash::vk::QUEUE_FAMILY_IGNORED,
                    dst_queue_family_index: ash::vk::QUEUE_FAMILY_IGNORED,
                    image: dst_texture.image,
                    range: ash::vk::ImageSubresourceRange {
                        aspect_mask: ash::vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                };
                vk_sync::cmd::pipeline_barrier(
                    &vk_device.fp_v1_0(),
                    *present_list,
                    None,
                    &[],
                    &[swap_to_copy],
                );

                // Use an image blit so the copy respects the underlying src and dst formats (i.e. RGBA->BGRA swizzling)
                let blit_region = ash::vk::ImageBlit {
                    src_subresource: ash::vk::ImageSubresourceLayers {
                        aspect_mask: ash::vk::ImageAspectFlags::COLOR,
                        mip_level: 0,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                    src_offsets: [
                        ash::vk::Offset3D { x: 0, y: 0, z: 0 },
                        ash::vk::Offset3D {
                            x: src_texture.desc.width as i32,
                            y: src_texture.desc.height as i32,
                            z: 1,
                        },
                    ],
                    dst_subresource: ash::vk::ImageSubresourceLayers {
                        aspect_mask: ash::vk::ImageAspectFlags::COLOR,
                        mip_level: 0,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                    dst_offsets: [
                        ash::vk::Offset3D { x: 0, y: 0, z: 0 },
                        ash::vk::Offset3D {
                            x: dst_texture.desc.width as i32,
                            y: dst_texture.desc.height as i32,
                            z: 1,
                        },
                    ],
                };

                unsafe {
                    vk_device.cmd_blit_image(
                        *present_list,
                        src_texture.image,
                        ash::vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                        dst_texture.image,
                        ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        &[blit_region],
                        ash::vk::Filter::NEAREST,
                    );
                }

                // Source Image: Copy Source -> <Unknown>
                let copy_to_src = vk_sync::ImageBarrier {
                    previous_accesses: vec![vk_sync::AccessType::TransferRead],
                    next_accesses: vec![src_before_access.clone()],
                    previous_layout: vk_sync::ImageLayout::Optimal,
                    next_layout: src_before_layout.clone(),
                    discard_contents: false,
                    src_queue_family_index: ash::vk::QUEUE_FAMILY_IGNORED,
                    dst_queue_family_index: ash::vk::QUEUE_FAMILY_IGNORED,
                    image: src_texture.image,
                    range: ash::vk::ImageSubresourceRange {
                        aspect_mask: ash::vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                };
                vk_sync::cmd::pipeline_barrier(
                    &vk_device.fp_v1_0(),
                    *present_list,
                    None,
                    &[],
                    &[copy_to_src],
                );

                // SwapChain Image: Copy Dest -> Present
                let copy_to_swap = vk_sync::ImageBarrier {
                    previous_accesses: vec![vk_sync::AccessType::TransferWrite],
                    next_accesses: vec![vk_sync::AccessType::Present],
                    previous_layout: vk_sync::ImageLayout::Optimal,
                    next_layout: vk_sync::ImageLayout::General,
                    discard_contents: false,
                    src_queue_family_index: ash::vk::QUEUE_FAMILY_IGNORED,
                    dst_queue_family_index: ash::vk::QUEUE_FAMILY_IGNORED,
                    image: dst_texture.image,
                    range: ash::vk::ImageSubresourceRange {
                        aspect_mask: ash::vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                };
                vk_sync::cmd::pipeline_barrier(
                    &vk_device.fp_v1_0(),
                    *present_list,
                    None,
                    &[],
                    &[copy_to_swap],
                );

                self.present_command_list.borrow_mut().close()?;

                if let Some(ref queue) = self.get_universal_queue() {
                    self.present_command_list.borrow_mut().submit(
                        queue.clone(),
                        &[swap_chain.acquire_image_semaphore],
                        &[swap_chain.render_done_semaphore],
                        None,
                        ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                    )?;

                    let queue = queue.write().unwrap();
                    let present_info = ash::vk::PresentInfoKHR {
                        s_type: ash::vk::StructureType::PRESENT_INFO_KHR,
                        p_next: ptr::null(),
                        wait_semaphore_count: 1,
                        p_wait_semaphores: &swap_chain.render_done_semaphore,
                        swapchain_count: 1,
                        p_swapchains: &swap_chain.swap_chain,
                        p_image_indices: &swap_chain.back_buffer_index,
                        p_results: ptr::null_mut(),
                    };

                    let present_result =
                        unsafe { self.swap_chain_loader.queue_present(*queue, &present_info) };

                    match present_result {
                        Ok(_) => {}
                        Err(ash::vk::Result::SUBOPTIMAL_KHR) => {
                            error!("SUBOPTIMAL_KHR error occurred")
                            // recreate_swap_chain
                        }
                        Err(ash::vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                            debug!("ERROR_OUT_OF_DATE_KHR error occurred");
                            // recreate_swap_chain
                        }
                        Err(err) => {
                            error!("Failed to present swap chain image! {:?}", err);
                        }
                    }
                }
            }
            Err(ash::vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                debug!("ERROR_OUT_OF_DATE_KHR error occurred");
                //panic!("ERROR_OUT_OF_DATE_KHR error occurred")
                //recreateSwapChain();
            }
            Err(ash::vk::Result::SUBOPTIMAL_KHR) => {
                error!("SUBOPTIMAL_KHR error occurred");
                //panic!("SUBOPTIMAL_KHR error occurred")
                //recreateSwapChain();
            }
            Err(err) => panic!("Failed to acquire swap chain image! {:?}", err),
        }

        Ok(())
    }

    fn resize_swap_chain(
        &self,
        swap_chain: RenderResourceHandle,
        width: u32,
        height: u32,
    ) -> Result<()> {
        warn!(
            "Resizing swap chain to width:{}, height:{} - {:?}",
            width, height, swap_chain
        );

        for _ in 0..MAX_GPU_FRAMES {
            // Flush any frames in flight
            match self.advance_frame() {
                Ok(_) => {}
                Err(err) => error!("Error flushing frames during swap chain resize: {:?}", err),
            }
        }

        // Make sure the GPU is done processing everything
        let vk_device = self.logical_device.device();
        unsafe {
            vk_device.device_wait_idle().unwrap();
        }

        let swap_chain = self.storage.get(swap_chain)?;
        let mut swap_chain = swap_chain.write().unwrap();
        let swap_chain = swap_chain.downcast_mut::<RenderSwapChainVk>().unwrap();

        let mut texture_desc = swap_chain.textures[0].desc.clone();
        texture_desc.width = width;
        texture_desc.height = height;

        // Drop old swap chain images
        swap_chain.textures.clear();

        // Destroy old swap chain
        unsafe {
            self.swap_chain_loader
                .destroy_swapchain(swap_chain.swap_chain, None);
        }
        swap_chain.swap_chain = ash::vk::SwapchainKHR::null(); // TODO: Needed?

        // Re-query capabilities as per the spec
        let surface_capabilities = unsafe {
            self.instance
                .get_surface_loader()
                .get_physical_device_surface_capabilities(
                    self.physical_device.raw,
                    swap_chain.surface,
                )
                .unwrap()
        };

        // Recreate swap chain
        swap_chain.swap_chain_info.image_extent.width = texture_desc.width;
        swap_chain.swap_chain_info.image_extent.height = texture_desc.height;
        swap_chain.swap_chain_info.old_swapchain = swap_chain.swap_chain;

        swap_chain.swap_chain = unsafe {
            self.swap_chain_loader
                .create_swapchain(&swap_chain.swap_chain_info, None)
                .unwrap()
        };

        let images = unsafe {
            self.swap_chain_loader
                .get_swapchain_images(swap_chain.swap_chain)
                .unwrap()
        };

        swap_chain.textures = images
            .iter()
            .map(|&image| {
                RenderTextureVk {
                    name: "Swap Chain Texture".into(),
                    desc: texture_desc.clone(),
                    image,
                    allocation: None,
                    supported_states: get_resource_states(texture_desc.bind_flags),
                    default_state: RenderResourceStates::COMMON, //PRESENT,
                }
            })
            .collect();

        swap_chain.back_buffer_index = 0;
        Ok(())
    }

    fn advance_frame(&self) -> Result<()> {
        let mut frames = self.frames.write().unwrap();
        frames.frame_index = (frames.frame_index + 1) % MAX_GPU_FRAMES;
        trace!("Advancing device frame - index: {}", frames.frame_index);

        let render_ahead_limit = std::cmp::min(frames.frame_index, MAX_GPU_FRAMES);
        let wait_frame_index = frames.frame_index - render_ahead_limit;

        self.flush_transfers();

        unsafe {
            // TODO: Temp for testing
            self.logical_device.device().device_wait_idle().unwrap();
        }

        // TODO: reset linear allocator
        frames.frames[frames.frame_index]
            .linear_allocator
            .write()
            .unwrap()
            .reset();

        Ok(())
    }

    // Cross-Node Transfer [Prototype]
    fn device_transfer(
        &self,
        wait_value: u64,
        signal_value: u64,
        fence: RenderResourceHandle,
        command_list: &RenderCommandList,
    ) -> Result<()> {
        unimplemented!()
    }

    fn device_graphics_signal(&self, signal_value: u64, fence: RenderResourceHandle) -> Result<()> {
        unimplemented!()
    }

    fn device_graphics_wait(&self, wait_value: u64, fence: RenderResourceHandle) -> Result<()> {
        unimplemented!()
    }

    fn device_copy_signal(&self, signal_value: u64, fence: RenderResourceHandle) -> Result<()> {
        unimplemented!()
    }

    fn device_copy_wait(&self, wait_value: u64, fence: RenderResourceHandle) -> Result<()> {
        unimplemented!()
    }

    fn device_acquire(&self, resource: RenderResourceHandle) -> Result<()> {
        unimplemented!()
    }

    fn device_unacquire(&self, resource: RenderResourceHandle) -> Result<()> {
        unimplemented!()
    }

    fn device_flush(&self) -> Result<()> {
        unimplemented!()
    }

    fn get_device_info(&self) -> Result<RenderDeviceInfo> {
        Ok(self.device_info.clone())
    }

    fn shader_format(&self) -> Result<String> {
        Ok("spv".to_string())
    }

    fn ray_tracing_supported(&self) -> bool {
        return false;
    }
}

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
unsafe fn create_surface<E: ash::version::EntryV1_0, I: ash::version::InstanceV1_0>(
    entry: &E,
    instance: &I,
    window: &RenderSwapChainWindow,
) -> Result<ash::vk::SurfaceKHR> {
    use winit::os::unix::WindowExt;

    let x11_display = window.get_xlib_display().unwrap();
    let x11_window = window.get_xlib_window().unwrap();
    let x11_create_info = ash::vk::XlibSurfaceCreateInfoKHR {
        flags: Default::default(),
        window: x11_window as ash::vk::Window,
        dpy: x11_display as *mut ash::vk::Display,
        ..Default::default()
    };
    let xlib_surface_loader = XlibSurface::new(entry, instance);
    Ok(xlib_surface_loader
        .create_xlib_surface_khr(&x11_create_info, None)
        .unwrap() /* TODO */)
}

#[cfg(target_os = "macos")]
unsafe fn create_surface<E: ash::version::EntryV1_0, I: ash::version::InstanceV1_0>(
    entry: &E,
    instance: &I,
    window: &RenderSwapChainWindow,
) -> Result<ash::vk::SurfaceKHR> {
    //use winit::os::macos::WindowExt;

    //let wnd: cocoa_id = mem::transmute(window.get_nswindow());

    //let layer = CoreAnimationLayer::new();

    //layer.set_edge_antialiasing_mask(0);
    //layer.set_presents_with_transaction(false);
    //layer.remove_all_animations();

    //let view = wnd.contentView();

    //layer.set_contents_scale(view.backingScaleFactor());
    //view.setLayer(mem::transmute(layer.as_ref()));
    //view.setWantsLayer(YES);

    let create_info = ash::vk::MacOSSurfaceCreateInfoMVK {
        s_type: ash::vk::StructureType::MACOS_SURFACE_CREATE_INFO_M,
        p_next: ptr::null(),
        flags: Default::default(),
        //p_view: window.get_nsview() as *const std::os::raw::c_void,
        p_view: window.ns_view,
    };

    let macos_surface_loader = MacOSSurface::new(entry, instance);
    Ok(macos_surface_loader
        .create_mac_os_surface_mvk(&create_info, None)
        .unwrap()) // TODO:
}

#[cfg(target_os = "windows")]
unsafe fn create_surface<E: ash::version::EntryV1_0, I: ash::version::InstanceV1_0>(
    entry: &E,
    instance: &I,
    window: &RenderSwapChainWindow,
) -> Result<ash::vk::SurfaceKHR> {
    let win32_create_info = ash::vk::Win32SurfaceCreateInfoKHR {
        s_type: ash::vk::StructureType::WIN32_SURFACE_CREATE_INFO_KHR,
        p_next: ptr::null(),
        flags: Default::default(),
        hinstance: window.hinstance,
        hwnd: window.hwnd,
    };
    let win32_surface_loader = Win32Surface::new(entry, instance);
    Ok(win32_surface_loader
        .create_win32_surface(&win32_create_info, None)
        .unwrap() /* TODO */)
}
