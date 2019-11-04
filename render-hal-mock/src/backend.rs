#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

use crate::device::RenderDeviceMock;
use enum_primitive::FromPrimitive;
use failure::Fail;
use render_core::backend::RenderBackend;
use render_core::device::RenderDevice;
use render_core::device::RenderDeviceCaps;
use render_core::device::RenderDeviceId;
use render_core::device::RenderDeviceInfo;
use render_core::device::RenderDeviceType;
use render_core::device::RenderDeviceVendor;
use render_core::error::{Error, Result};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fmt;
use std::ptr;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub(crate) struct RenderBackendMock {
    device_info: Vec<RenderDeviceInfo>,
    device_map: HashMap<u32, u32>,
    devices: Vec<Arc<RwLock<Option<Box<dyn RenderDevice>>>>>,
}

impl RenderBackendMock {
    pub fn new() -> Result<Self> {
        Ok(RenderBackendMock {
            device_info: Vec::new(),
            device_map: HashMap::new(),
            devices: Vec::new(),
        })
    }
}

impl Drop for RenderBackendMock {
    fn drop(&mut self) {}
}

impl RenderBackend for RenderBackendMock {
    fn is_initialized(&self) -> bool {
        true
    }

    fn enumerate_devices(
        &mut self,
        max_devices: u32,
        mirror_count: u32,
        software: bool,
    ) -> Result<Vec<RenderDeviceInfo>> {
        self.device_info.clear();
        self.device_map.clear();
        self.devices.clear();

        let info = RenderDeviceInfo {
            name: "MockDevice".to_string(),
            vendor: RenderDeviceVendor::Unknown,
            device_id: 12345,
            device_index: 0,
            device_type: RenderDeviceType::Cpu,
            caps: Default::default(),
        };

        self.device_map.insert(0u32, 0u32);
        self.device_info.push(info);

        self.devices
            .resize(self.device_info.len(), Arc::new(RwLock::new(None)));

        let actual_device_count = self.device_map.len();
        for mirror_index in 0..mirror_count {
            for device_index in 0..actual_device_count {
                let map_len = self.device_map.len() as u32;
                self.device_map.insert(map_len, device_index as u32);
            }
        }

        Ok(self.device_info.clone())
    }

    fn create_device(&mut self, device_id: RenderDeviceId) -> Result<()> {
        let device_index = device_id as usize;
        if device_index >= self.devices.len() {
            Err(Error::backend(format!(
                "no device found for id {}",
                device_index
            )))
        } else {
            let info = self.device_info[device_index].clone();
            let device = Box::new(RenderDeviceMock::new(info)?);
            self.devices[device_index] = Arc::new(RwLock::new(Some(device)));
            let _device = Arc::clone(&self.devices[device_index]);
            Ok(())
        }
    }

    fn destroy_device(&mut self, device_id: RenderDeviceId) -> Result<()> {
        let device_index = device_id as usize;
        if device_index >= self.devices.len() {
            Err(Error::backend(format!(
                "no device found for id {}",
                device_index
            )))
        } else {
            self.devices[device_index] = Arc::new(RwLock::new(None));
            Ok(())
        }
    }

    fn get_device(
        &self,
        device_id: RenderDeviceId,
    ) -> Result<Arc<RwLock<Option<Box<dyn RenderDevice>>>>> {
        let device_index = device_id as usize;
        if device_index >= self.devices.len() {
            Err(Error::backend(format!(
                "no device found for id {}",
                device_index
            )))
        } else {
            Ok(Arc::clone(&self.devices[device_index]))
        }
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
