use crate::encoder::*;
use crate::error::{Error, Result};
use crate::format::*;
use crate::handles::RenderResourceHandle;
use crate::state::*;
use crate::types::*;
use failure::Fail;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::sync::RwLock;

pub type RenderDeviceId = u32;
pub type RenderDeviceEntry = Arc<RwLock<Option<Box<dyn RenderDevice>>>>;

enum_from_primitive! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
    #[repr(u32)]
    pub enum RenderDeviceVendor {
        Nvidia		= 0x10DE,
        Amd			= 0x1002,
        Intel		= 0x8086,
        Unknown		= 0
    }
}

enum_from_primitive! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub enum RenderDeviceType {
        /// The device does not match any other available types.
        Other = 0,

        /// The device is typically running on the same processors as the host.
        Cpu = 1,

        /// The device is typically a virtual node in a virtualization environment.
        Virtual = 2,

        /// The device is typically one embedded in or tightly coupled with the host.
        Integrated = 3,

        /// The device is typically running on the same processors as the host.
        Discrete = 4,
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct RenderDeviceCaps {
    /// Does the adapter support RenderPrimitiveType::QuadList
    pub supports_quads: bool,

    /// Does the adapter support RenderPrimitiveType::RectList
    pub supports_rect_list: bool,

    /// Does does the adapter support row major cross adapter textures
    pub supports_row_major_cross_adapter: bool,

    pub supports_typed_uav_load_r11g11b10_float: bool,
    pub supports_typed_uav_load_r16g16b16a16_float: bool,

    /// Capabilities of each format
    pub format_capabilities: HashMap<RenderFormat, RenderFormatCapability>,

    /// Max width and height on textures
    pub max_texture_dimension: u32,

    /// Max texture array size for. Used by Tex1dArray, Tex2dArray, CubeArray.
    /// If set to 0 then texture arrays are not supported
    pub max_texture_array_size: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderDeviceInfo {
    /// Name of the device. Should only be used for logging / debugging purposes and may not be available
    pub name: String,

    /// Which vendor created this GPU (typically: AMD, Nvidia, Intel).
    /// This can also contain other integer values for new/unknown vendor ids
    pub vendor: RenderDeviceVendor,

    /// Unique ID for the specific GPU model for identification purposes
    pub device_id: u32,

    /// Device index returned from a backend during enumeration
    pub device_index: RenderDeviceId,

    pub device_type: RenderDeviceType,

    pub caps: RenderDeviceCaps,
}

pub trait RenderDevice: fmt::Debug {
    fn valid_resource(&self, handle: RenderResourceHandle) -> bool;
    fn destroy_resource(&self, handle: RenderResourceHandle) -> Result<()>;

    // Resource Management
    fn create_swap_chain(
        &self,
        handle: RenderResourceHandle,
        desc: &RenderSwapChainDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()>;

    fn create_buffer(
        &self,
        handle: RenderResourceHandle,
        desc: &RenderBufferDesc,
        initial_data: Option<&[u8]>,
        debug_name: Cow<'static, str>,
    ) -> Result<()>;

    fn create_texture(
        &self,
        handle: RenderResourceHandle,
        desc: &RenderTextureDesc,
        initial_data: Option<RenderTextureSubResourceData>,
        debug_name: Cow<'static, str>,
    ) -> Result<()>;

    fn create_sampler_state(
        &self,
        handle: RenderResourceHandle,
        state: &RenderSamplerState,
        debug_name: Cow<'static, str>,
    ) -> Result<()>;

    fn create_shader(
        &self,
        handle: RenderResourceHandle,
        desc: &RenderShaderDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()>;

    fn create_shader_views(
        &self,
        handle: RenderResourceHandle,
        desc: &RenderShaderViewsDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()>;

    // Ray tracing features are only supported on some devices
    fn create_ray_tracing_program(
        &self,
        handle: RenderResourceHandle,
        desc: &RayTracingProgramDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()>;

    fn create_ray_tracing_geometry(
        &self,
        handle: RenderResourceHandle,
        desc: &RayTracingGeometryDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()>;

    fn create_ray_tracing_top_acceleration(
        &self,
        handle: RenderResourceHandle,
        desc: &RayTracingTopAccelerationDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()>;

    fn create_ray_tracing_bottom_acceleration(
        &self,
        handle: RenderResourceHandle,
        desc: &RayTracingBottomAccelerationDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()>;

    fn create_ray_tracing_pipeline_state(
        &self,
        handle: RenderResourceHandle,
        desc: &RayTracingPipelineStateDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()>;

    fn create_ray_tracing_shader_table(
        &self,
        handle: RenderResourceHandle,
        desc: &RayTracingShaderTableDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()>;

    fn create_graphics_pipeline_state(
        &self,
        handle: RenderResourceHandle,
        desc: &RenderGraphicsPipelineStateDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()>;

    fn create_compute_pipeline_state(
        &self,
        handle: RenderResourceHandle,
        desc: &RenderComputePipelineStateDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()>;

    fn create_draw_binding_set(
        &self,
        handle: RenderResourceHandle,
        desc: &RenderDrawBindingSetDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()>;

    fn create_frame_binding_set(
        &self,
        handle: RenderResourceHandle,
        desc: &RenderFrameBindingSetDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()>;

    fn create_render_pass(
        &self,
        handle: RenderResourceHandle,
        desc: &RenderPassDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()>;

    fn create_command_list(
        &self,
        handle: RenderResourceHandle,
        debug_name: Cow<'static, str>,
    ) -> Result<()>;

    fn create_fence(
        &self,
        handle: RenderResourceHandle,
        desc: &RenderFenceDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()>;

    fn create_timing_heap(
        &self,
        handle: RenderResourceHandle,
        desc: &RenderTimingHeapDesc,
        debug_name: Cow<'static, str>,
    ) -> Result<()>;

    // Resource Management (Shared / CrossAdapter)
    // TODO:

    // Timing Heap Management
    fn get_timing_frequency(&self) -> Result<f64>;

    // CommandList Management
    fn submit_command_list(
        &self,
        handle: RenderResourceHandle,
        flush: bool,
        wait_before: Option<&[RenderResourceHandle]>,
        signal_after: Option<RenderResourceHandle>,
    ) -> Result<()>;

    fn compile_command_list(
        &self,
        handle: RenderResourceHandle,
        command_list: &RenderCommandList,
    ) -> Result<()>;

    fn compile_command_lists(
        &self,
        handle: RenderResourceHandle,
        command_lists: &[RenderCommandList],
    ) -> Result<()>;

    // Present Management
    fn present_swap_chain(
        &mut self,
        swap_chain: RenderResourceHandle,
        source_texture: RenderResourceHandle,
    ) -> Result<()>;

    fn resize_swap_chain(
        &self,
        swap_chain: RenderResourceHandle,
        width: u32,
        height: u32,
    ) -> Result<()>;

    fn advance_frame(&self) -> Result<()>;

    // Transfer Management
    // TODO:

    // Cross-Node Transfer [Prototype]
    fn device_transfer(
        &self,
        wait_value: u64,
        signal_value: u64,
        fence: RenderResourceHandle,
        command_list: &RenderCommandList,
    ) -> Result<()>;

    fn device_graphics_signal(&self, signal_value: u64, fence: RenderResourceHandle) -> Result<()>;

    fn device_graphics_wait(&self, wait_value: u64, fence: RenderResourceHandle) -> Result<()>;

    fn device_copy_signal(&self, signal_value: u64, fence: RenderResourceHandle) -> Result<()>;

    fn device_copy_wait(&self, wait_value: u64, fence: RenderResourceHandle) -> Result<()>;

    fn device_acquire(&self, resource: RenderResourceHandle) -> Result<()>;

    fn device_unacquire(&self, resource: RenderResourceHandle) -> Result<()>;

    fn device_flush(&self) -> Result<()>;

    fn get_device_info(&self) -> Result<RenderDeviceInfo>;

    fn shader_format(&self) -> Result<String>;

    fn ray_tracing_supported(&self) -> bool {
        return false;
    }
}

#[derive(Default, Debug, Clone)]
pub struct RenderDeviceGroup {
    pub primary: RenderDeviceEntry,
    pub secondaries: Vec<RenderDeviceEntry>,
}

pub type RenderDeviceGroupCallRefFn = Box<dyn Fn(&Box<dyn RenderDevice>)>;
pub type RenderDeviceGroupCallMutFn = Box<dyn Fn(&mut Box<dyn RenderDevice>)>;

impl RenderDeviceGroup {
    pub fn new(primary: RenderDeviceEntry, secondaries: &[RenderDeviceEntry]) -> Self {
        RenderDeviceGroup {
            primary,
            secondaries: secondaries.to_vec(),
        }
    }

    /*pub fn call_mut(&self, func: impl Fn(&mut Box<dyn RenderDevice>)) {
        let mut primary = self.primary.write();
        match primary {
            Ok(device) => match *device {
                Some(&mut device) => {
                    (func)(device);
                }
                None => {}
            },
            Err(_err) => {}
        }
    }*/

    pub fn call_ref(&self, func: impl Fn(&Box<dyn RenderDevice>) -> Result<()>) -> Result<()> {
        // Primary device
        {
            let primary = self.primary.read();
            match primary {
                Ok(device) => match *device {
                    Some(ref device) => {
                        (func)(device)?;
                    }
                    None => {}
                },
                Err(_err) => {}
            }
        }
        // Secondary devices
        for secondary in &self.secondaries {
            let secondary = secondary.read();
            match secondary {
                Ok(device) => match *device {
                    Some(ref device) => {
                        (func)(device)?;
                    }
                    None => {}
                },
                Err(_err) => {}
            }
        }

        Ok(())
    }
}
