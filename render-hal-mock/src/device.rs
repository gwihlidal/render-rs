#![allow(unused_variables)]
#![allow(dead_code)]

use crate::backend::RenderBackendMock;
use crate::types::*;
use render_core::device::*;
use render_core::encoder::*;
use render_core::error::{Error, Result};
use render_core::handles::RenderResourceHandle;
use render_core::resources::{RenderResourceBase, RenderResourceStorage};
use render_core::state::*;
use render_core::types::*;
use std::{
    borrow::Cow,
    collections::HashMap,
    fmt,
    iter::once,
    mem,
    mem::align_of,
    ptr,
    sync::{Arc, RwLock},
    u32,
};

#[derive(Debug)]
pub struct RenderDeviceMock {
    device_info: RenderDeviceInfo,
    storage: RenderResourceStorage<Box<dyn RenderResourceBase>>,
}

impl RenderDeviceMock {
    pub fn new(device_info: RenderDeviceInfo) -> Result<Self> {
        Ok(RenderDeviceMock {
            device_info,
            storage: RenderResourceStorage::new(),
        })
    }

    fn advance(&self) -> Result<()> {
        Ok(())
    }
}

impl RenderDevice for RenderDeviceMock {
    fn valid_resource(&self, handle: RenderResourceHandle) -> bool {
        self.storage.valid(handle)
    }

    fn destroy_resource(&self, handle: RenderResourceHandle) -> Result<()> {
        let resource_lock = self.storage.remove(handle)?;
        let resource = resource_lock.write().unwrap();
        println!(
            "Destroying resource - name: {}, handle: {:?}",
            resource.get_name(),
            handle
        );
        assert!(resource.get_type() == handle.get_type());
        match handle.get_type() {
            RenderResourceType::SwapChain => {
                let mut _resource = resource.downcast_ref::<RenderSwapChainMock>().unwrap();
            }
            RenderResourceType::Buffer => {
                let mut _resource = resource.downcast_ref::<RenderBufferMock>().unwrap();
            }
            RenderResourceType::Texture => {
                let mut _resource = resource.downcast_ref::<RenderTextureMock>().unwrap();
            }
            RenderResourceType::SamplerState => {
                let mut _resource = resource.downcast_ref::<RenderSamplerStateMock>().unwrap();
            }
            RenderResourceType::Shader => {
                let mut _resource = resource.downcast_ref::<RenderShaderMock>().unwrap();
            }
            RenderResourceType::ShaderViews => {
                let mut _resource = resource.downcast_ref::<RenderShaderViewsMock>().unwrap();
            }
            RenderResourceType::GraphicsPipelineState => {
                let mut _resource = resource
                    .downcast_ref::<RenderGraphicsPipelineStateMock>()
                    .unwrap();
            }
            RenderResourceType::ComputePipelineState => {
                let mut _resource = resource
                    .downcast_ref::<RenderComputePipelineStateMock>()
                    .unwrap();
            }
            RenderResourceType::RayTracingGeometry => {
                let mut _resource = resource
                    .downcast_ref::<RenderRayTracingGeometryMock>()
                    .unwrap();
            }
            RenderResourceType::RayTracingProgram => {
                let mut _resource = resource
                    .downcast_ref::<RenderRayTracingProgramMock>()
                    .unwrap();
            }
            RenderResourceType::RayTracingAcceleration => {
                let mut _resource = resource
                    .downcast_ref::<RenderRayTracingAccelerationMock>()
                    .unwrap();
            }
            RenderResourceType::RayTracingPipelineState => {
                let mut _resource = resource
                    .downcast_ref::<RenderRayTracingPipelineStateMock>()
                    .unwrap();
            }
            RenderResourceType::RayTracingShaderTable => {
                let mut _resource = resource
                    .downcast_ref::<RenderRayTracingShaderTableMock>()
                    .unwrap();
            }
            RenderResourceType::DrawBindingSet => {
                let mut _resource = resource.downcast_ref::<RenderDrawBindingSetMock>().unwrap();
            }
            RenderResourceType::FrameBindingSet => {
                let mut _resource = resource
                    .downcast_ref::<RenderFrameBindingSetMock>()
                    .unwrap();
            }
            RenderResourceType::RenderPass => {
                let mut _resource = resource.downcast_ref::<RenderPassMock>().unwrap();
            }
            RenderResourceType::CommandList => {
                let mut _resource = resource.downcast_ref::<RenderCommandListMock>().unwrap();
            }
            RenderResourceType::Fence => {
                let mut _resource = resource.downcast_ref::<RenderFenceMock>().unwrap();
            }
            RenderResourceType::TimingHeap => {
                let mut _resource = resource.downcast_ref::<RenderTimingHeapMock>().unwrap();
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
        println!(
            "Creating swap chain: name:{}, format:{:?}, width:{}, height:{}, buffers:{}",
            debug_name, desc.format, desc.width, desc.height, desc.buffer_count
        );

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

        let mut textures: Vec<RenderTextureMock> = Vec::new();
        for index in 0..desc.buffer_count {
            textures.push(RenderTextureMock {
                name: "Swap Chain Texture".into(),
                desc: tex_desc.clone(),
            });
        }

        let resource: Arc<RwLock<Box<dyn RenderResourceBase>>> =
            Arc::new(RwLock::new(Box::new(RenderSwapChainMock {
                name: debug_name.to_string().into(),
                textures,
                back_buffer_index: 0,
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
        println!("Creating buffer: {}, {:?}", debug_name, desc);

        let data_size: u64 = match initial_data {
            Some(data) => mem::size_of_val(&data) as u64,
            None => 0u64,
        };

        if let Some(data) = initial_data {}

        let resource: Arc<RwLock<Box<dyn RenderResourceBase>>> =
            Arc::new(RwLock::new(Box::new(RenderBufferMock {
                name: debug_name.to_string().into(),
                desc: desc.clone(),
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
        println!("Creating texture: {}, {:?}", debug_name, desc);

        let resource: Arc<RwLock<Box<dyn RenderResourceBase>>> =
            Arc::new(RwLock::new(Box::new(RenderTextureMock {
                name: debug_name.to_string().into(),
                desc: desc.clone(),
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
        println!("Creating sampler: {}, {:?}", debug_name, desc);

        let resource: Arc<RwLock<Box<dyn RenderResourceBase>>> =
            Arc::new(RwLock::new(Box::new(RenderSamplerStateMock {
                name: debug_name.to_string().into(),
                //desc: desc.clone(),
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
        println!("Creating shader: {}, {:?}", debug_name, desc.shader_type);

        let resource: Arc<RwLock<Box<dyn RenderResourceBase>>> =
            Arc::new(RwLock::new(Box::new(RenderShaderMock {
                name: debug_name.to_string().into(),
            })));

        self.storage.put(handle, resource)?;
        Ok(())
    }

    fn create_shader_views(
        &self,
        handle: RenderResourceHandle,
        desc: &RenderShaderViewsDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        println!("Creating shader views: {}, {:?}", debug_name, desc);

        let resource: Arc<RwLock<Box<dyn RenderResourceBase>>> =
            Arc::new(RwLock::new(Box::new(RenderShaderViewsMock {
                name: debug_name.to_string().into(),
                //desc: desc.clone(),
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
        println!("Creating ray tracing program: {}, {:?}", debug_name, desc);

        let resource: Arc<RwLock<Box<dyn RenderResourceBase>>> =
            Arc::new(RwLock::new(Box::new(RenderRayTracingProgramMock {
                name: debug_name.to_string().into(),
                //desc: desc.clone(),
            })));

        self.storage.put(handle, resource)?;
        Ok(())
    }

    fn create_ray_tracing_geometry(
        &self,
        handle: RenderResourceHandle,
        desc: &RayTracingGeometryDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        println!("Creating ray tracing geometry: {}, {:?}", debug_name, desc);

        let resource: Arc<RwLock<Box<dyn RenderResourceBase>>> =
            Arc::new(RwLock::new(Box::new(RenderRayTracingGeometryMock {
                name: debug_name.to_string().into(),
                //desc: desc.clone(),
            })));

        self.storage.put(handle, resource)?;
        Ok(())
    }

    fn create_ray_tracing_top_acceleration(
        &self,
        handle: RenderResourceHandle,
        desc: &RayTracingTopAccelerationDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        println!(
            "Creating ray tracing top acceleration: {}, {:?}",
            debug_name, desc
        );

        let resource: Arc<RwLock<Box<dyn RenderResourceBase>>> =
            Arc::new(RwLock::new(Box::new(RenderRayTracingAccelerationMock {
                name: debug_name.to_string().into(),
                //desc: desc.clone(),
            })));

        self.storage.put(handle, resource)?;
        Ok(())
    }

    fn create_ray_tracing_bottom_acceleration(
        &self,
        handle: RenderResourceHandle,
        desc: &RayTracingBottomAccelerationDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        println!(
            "Creating ray tracing bottom acceleration: {}, {:?}",
            debug_name, desc
        );

        let resource: Arc<RwLock<Box<dyn RenderResourceBase>>> =
            Arc::new(RwLock::new(Box::new(RenderRayTracingAccelerationMock {
                name: debug_name.to_string().into(),
                //desc: desc.clone(),
            })));

        self.storage.put(handle, resource)?;
        Ok(())
    }

    fn create_ray_tracing_pipeline_state(
        &self,
        handle: RenderResourceHandle,
        desc: &RayTracingPipelineStateDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        println!(
            "Creating ray tracing pipeline state: {}, {:?}",
            debug_name, desc
        );

        let resource: Arc<RwLock<Box<dyn RenderResourceBase>>> =
            Arc::new(RwLock::new(Box::new(RenderRayTracingPipelineStateMock {
                name: debug_name.to_string().into(),
                //desc: desc.clone(),
            })));

        self.storage.put(handle, resource)?;
        Ok(())
    }

    fn create_ray_tracing_shader_table(
        &self,
        handle: RenderResourceHandle,
        desc: &RayTracingShaderTableDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        println!(
            "Creating ray tracing shader table: {}, {:?}",
            debug_name, desc
        );

        let resource: Arc<RwLock<Box<dyn RenderResourceBase>>> =
            Arc::new(RwLock::new(Box::new(RenderRayTracingShaderTableMock {
                name: debug_name.to_string().into(),
                //desc: desc.clone(),
            })));

        self.storage.put(handle, resource)?;
        Ok(())
    }

    fn create_graphics_pipeline_state(
        &self,
        handle: RenderResourceHandle,
        desc: &RenderGraphicsPipelineStateDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        println!(
            "Creating graphics pipeline state: {}, {:?}",
            debug_name, desc
        );

        let resource: Arc<RwLock<Box<dyn RenderResourceBase>>> =
            Arc::new(RwLock::new(Box::new(RenderGraphicsPipelineStateMock {
                name: debug_name.to_string().into(),
                //desc: desc.clone(),
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
        println!(
            "Creating compute pipeline state: {}, {:?}",
            debug_name, desc
        );

        let resource: Arc<RwLock<Box<dyn RenderResourceBase>>> =
            Arc::new(RwLock::new(Box::new(RenderComputePipelineStateMock {
                name: debug_name.to_string().into(),
                //desc: desc.clone(),
            })));

        self.storage.put(handle, resource)?;
        Ok(())
    }

    fn create_draw_binding_set(
        &self,
        handle: RenderResourceHandle,
        desc: &RenderDrawBindingSetDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        println!("Creating draw binding set: {}, {:?}", debug_name, desc);

        let resource: Arc<RwLock<Box<dyn RenderResourceBase>>> =
            Arc::new(RwLock::new(Box::new(RenderDrawBindingSetMock {
                name: debug_name.to_string().into(),
                //desc: desc.clone(),
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
        println!("Creating frame binding set: {}, {:?}", debug_name, desc);

        let resource: Arc<RwLock<Box<dyn RenderResourceBase>>> =
            Arc::new(RwLock::new(Box::new(RenderFrameBindingSetMock {
                name: debug_name.to_string().into(),
                desc: desc.clone(),
            })));

        self.storage.put(handle, resource)?;
        Ok(())
    }

    fn create_render_pass(
        &self,
        handle: RenderResourceHandle,
        desc: &RenderPassDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        println!("Creating render pass: {}, {:?}", debug_name, desc);

        let resource: Arc<RwLock<Box<dyn RenderResourceBase>>> =
            Arc::new(RwLock::new(Box::new(RenderPassMock {
                name: debug_name.to_string().into(),
                desc: desc.clone(),
            })));

        self.storage.put(handle, resource)?;
        Ok(())
    }

    fn create_command_list(
        &self,
        handle: RenderResourceHandle,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        println!("Creating command list: {}", debug_name);

        let resource: Arc<RwLock<Box<dyn RenderResourceBase>>> =
            Arc::new(RwLock::new(Box::new(RenderCommandListMock {
                name: debug_name.to_string().into(),
                //desc: desc.clone(),
                list_type: RenderCommandListType::Universal,
                //device: &self.device,
            })));

        self.storage.put(handle, resource)?;
        Ok(())
    }

    fn create_fence(
        &self,
        handle: RenderResourceHandle,
        desc: &RenderFenceDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()> {
        println!("Creating fence: {}, {:?}", debug_name, desc);

        let resource: Arc<RwLock<Box<dyn RenderResourceBase>>> =
            Arc::new(RwLock::new(Box::new(RenderFenceMock {
                name: debug_name.to_string().into(),
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
        println!("Creating timing heap: {}, {:?}", debug_name, desc);

        let resource: Arc<RwLock<Box<dyn RenderResourceBase>>> =
            Arc::new(RwLock::new(Box::new(RenderTimingHeapMock {
                name: debug_name.to_string().into(),
                //desc: desc.clone(),
            })));

        self.storage.put(handle, resource)?;
        Ok(())
    }

    // Timing Heap Management
    fn get_timing_frequency(&self) -> Result<f64> {
        //unimplemented!()
        Ok(0f64)
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
        Ok(())
    }

    fn compile_command_list(
        &self,
        handle: RenderResourceHandle,
        command_list: &RenderCommandList,
    ) -> Result<()> {
        assert_eq!(handle.get_type(), RenderResourceType::CommandList);
        Ok(())
    }

    fn compile_command_lists(
        &self,
        handle: RenderResourceHandle,
        command_lists: &[RenderCommandList],
    ) -> Result<()> {
        Ok(())
    }

    // Present Management
    fn present_swap_chain(
        &mut self,
        swap_chain: RenderResourceHandle,
        source_texture: RenderResourceHandle,
    ) -> Result<()> {
        //println!("Presenting swap chain - {:?}", swap_chain);
        Ok(())
    }

    fn resize_swap_chain(
        &self,
        swap_chain: RenderResourceHandle,
        width: u32,
        height: u32,
    ) -> Result<()> {
        //println!(
        //	"Resizing swap chain to width:{}, height:{} - {:?}",
        //	width, height, swap_chain
        //);
        Ok(())
    }

    fn advance_frame(&self) -> Result<()> {
        //println!("Advancing device frame");
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
        Ok("mock".to_string())
    }

    fn ray_tracing_supported(&self) -> bool {
        return true;
    }
}
