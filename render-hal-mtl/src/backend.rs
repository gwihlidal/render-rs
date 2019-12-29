#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use failure::Fail;
use render_core::backend::RenderBackend;
use render_core::device::RenderDevice;
use render_core::device::RenderDeviceId;
use render_core::device::RenderDeviceInfo;
use render_core::error::{Error, Result};
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub(crate) struct RenderBackendMtl {}

impl RenderBackendMtl {
    pub fn new() -> Self {
        RenderBackendMtl {}
    }
}

impl Drop for RenderBackendMtl {
    fn drop(&mut self) {
        trace!("Drop called for RenderBackendMtl!");
    }
}

impl RenderBackend for RenderBackendMtl {
    fn is_initialized(&self) -> bool {
        unimplemented!()
    }

    fn enumerate_devices(
        &mut self,
        max_devices: u32,
        mirror_count: u32,
        software: bool,
    ) -> Result<Vec<RenderDeviceInfo>> {
        unimplemented!()
    }

    fn create_device(&mut self, device_id: RenderDeviceId) -> Result<()> {
        unimplemented!()
    }

    fn destroy_device(&mut self, device_id: RenderDeviceId) -> Result<()> {
        unimplemented!()
    }

    fn get_device(
        &self,
        device_id: RenderDeviceId,
    ) -> Result<Arc<RwLock<Option<Box<dyn RenderDevice>>>>> {
        unimplemented!()
    }

    fn begin_debug_capture(&self, name: &str) -> Result<()> {
        unimplemented!()
    }

    fn finish_debug_capture(&self) -> Result<()> {
        unimplemented!()
    }

    fn trigger_debug_capture(&self) -> Result<()> {
        unimplemented!()
    }

    fn launch_debug_capture(&self, quit: bool) -> Result<()> {
        unimplemented!()
    }
}
