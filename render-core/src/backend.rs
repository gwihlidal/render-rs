#![allow(dead_code)]
#![allow(unused_imports)]

use crate::device::{RenderDevice, RenderDeviceId, RenderDeviceInfo};
use crate::error::{Error, Result};
use failure::Fail;
use std::fmt;
use std::sync::{Arc, RwLock};

bitflags! {
    pub struct RenderDebugFlags: u32 {
        /// No debugger support.
        const NONE = 0x0;

        /// Enable RenderDoc integration.
        const RENDER_DOC = 0x1;

        /// Enable PIX integration.
        const PIX = 0x2;

        /// Enable CPU validation layer(s)
        const CPU_VALIDATION = 0x4;

        /// Enable GPU validation layer(s)
        const GPU_VALIDATION = 0x8;

        /// Enable post crash analysis layer
        const POST_CRASH_ANALYSIS = 0x10;
    }
}

/*pub enum RenderBackendApi {
    Dx12,
    Vulkan,
    Metal,
    Mock,
    Proxy(String),
}*/

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderBackendSettings {
    /// API to use (i.e. "Dx12", "Vulkan", etc)
    pub api: String, //RenderBackendApi,

    /// Address to use (i.e. when routing through proxy)
    /// i.e. x.x.x.x:50080
    pub address: Option<String>,

    //Handle deviceWindow = nullptr;
    pub debug_flags: RenderDebugFlags,
}

pub trait RenderBackend: fmt::Debug {
    fn is_initialized(&self) -> bool;

    fn enumerate_devices(
        &mut self,
        max_devices: u32,
        mirror_count: u32,
        software: bool,
    ) -> Result<Vec<RenderDeviceInfo>>;

    fn create_device(&mut self, device_id: RenderDeviceId) -> Result<()>;
    fn destroy_device(&mut self, device_id: RenderDeviceId) -> Result<()>;
    fn get_device(&self, device_id: RenderDeviceId) -> Result<Option<Box<dyn RenderDevice>>>;

    fn begin_debug_capture(&self, name: &str) -> Result<()>;
    fn finish_debug_capture(&self) -> Result<()>;
    fn trigger_debug_capture(&self) -> Result<()>;
    fn launch_debug_capture(&self, quit: bool) -> Result<()>;
}

pub struct RenderBackendRegistry {
    pub settings: RenderBackendSettings,
    pub backend: Arc<RwLock<Box<dyn RenderBackend>>>,
}

pub trait RenderBackendModule: fmt::Debug {
    fn name(&self) -> &'static str;
    fn api(&self) -> &'static str;
    fn create(&self) -> Box<dyn RenderBackend>;
}
