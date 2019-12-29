#![allow(unused_variables)]
#![allow(dead_code)]

#[cfg(target_os = "macos")]
use ash::extensions::mvk::MacOSSurface;

use crate::device::RenderDeviceVk;
use crate::raw::device::PhysicalDevice;
use crate::raw::errors::InstanceError;
use crate::raw::instance::{Instance, InstanceConfig};
use crate::raw::surface::Surface as RawSurface;
use crate::raw::swap_chain::SwapChain as RawSwapChain;
use ash::extensions::{ext::DebugReport, khr::Surface, khr::Swapchain};
use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk;
use ash::Device;
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

impl From<InstanceError> for Error {
    fn from(kind: InstanceError) -> Error {
        Error::backend("instance error!")
        //Error::from(Context::new(kind))
    }
}

//#[derive(Debug)]
pub(crate) struct RenderBackendVk {
    instance: Arc<Instance>,
    adapters: Vec<Arc<PhysicalDevice>>,
    device_info: Vec<RenderDeviceInfo>,
    device_map: HashMap<u32, u32>,
    devices: Vec<Arc<RwLock<Option<Box<RenderDevice>>>>>,
}

impl fmt::Debug for RenderBackendVk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TODO: RenderBackendVk")
        //write!(f, "Point {{ x: {}, y: {} }}", self.x, self.y)
    }
}

/// Layer description
#[derive(Clone, Debug)]
pub struct Layer<'a> {
    pub name: &'a str,
    pub spec_version: u32,
    pub implementation_version: u32,
    pub description: &'a str,
}

/// Extension description
#[derive(Clone, Debug)]
pub struct Extension<'a> {
    pub name: &'a str,
    pub spec_version: u32,
}

fn set_up_logging() {
    use fern::colors::{Color, ColoredLevelConfig};

    // configure colors for the whole line
    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        // we actually don't need to specify the color for debug and info, they are white by default
        .info(Color::White)
        .debug(Color::White)
        // depending on the terminals color scheme, this is the same as the background color
        .trace(Color::BrightBlack);

    // configure colors for the name of the level.
    // since almost all of them are the some as the color for the whole line, we
    // just clone `colors_line` and overwrite our changes
    let colors_level = colors_line.clone().info(Color::Green);
    // here we set up our fern Dispatch
    let dispatch = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{color_line}[{date}][{target}][{level}{color_line}] {message}\x1B[0m",
                color_line = format_args!(
                    "\x1B[{}m",
                    colors_line.get_color(&record.level()).to_fg_str()
                ),
                date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                target = record.target(),
                level = colors_level.color(record.level()),
                message = message,
            ));
        })
        // set the default log level. to filter out verbose log messages from dependencies, set
        // this to Warn and overwrite the log level for your crate.
        .level(log::LevelFilter::Warn)
        // change log levels for individual modules. Note: This looks for the record's target
        // field which defaults to the module path but can be overwritten with the `target`
        // parameter:
        // `info!(target="special_target", "This log message is about special_target");`
        .level_for("render_hal_vk", log::LevelFilter::Trace)
        .level_for("render_hal_vk::compile", log::LevelFilter::Info)
        .level_for("render_hal_vk::backend", log::LevelFilter::Warn)
        .level_for("render_hal_vk::types", log::LevelFilter::Warn)
        .level_for("render_hal_vk::device", log::LevelFilter::Info)
        .level_for("render_hal_vk::raw", log::LevelFilter::Warn)
        // output to stdout
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log").unwrap())
        .apply();
    match dispatch {
        Ok(_) => {
            debug!("finished setting up logging! yay!");
        }
        Err(err) => {
            //println!("error setting up logging! {:?}", err);
        }
    }
}

impl RenderBackendVk {
    pub fn new() -> Result<Self> {
        set_up_logging();

        let surface_extensions = RawSurface::extensions();
        let swapchain_extensions = RawSwapChain::extensions();

        trace!("Creating Instance");

        let instance = Instance::new(|layers, extensions| {
            debug!("Available instance layers: {:#?}", layers);
            debug!("Available instance extensions: {:#?}", extensions);

            assert!(surface_extensions
                .iter()
                .all(|&surface_extension| extensions
                    .iter()
                    .find(|extension| extension.name == surface_extension)
                    .is_some()));

            let mut explicit_layers: Vec<String> = Vec::new();
            explicit_layers.push(String::from("VK_LAYER_LUNARG_standard_validation"));

            let mut explicit_extensions: Vec<String> = surface_extensions
                .clone()
                .into_iter()
                .map(String::from)
                .collect();
            explicit_extensions.push(String::from("VK_EXT_debug_report"));
            //explicit_extensions.push(String::from("VK_EXT_debug_utils"));
            trace!("Selected extensions: {:#?}", explicit_extensions);

            InstanceConfig {
                app_name: "render-hal-vk".into(),
                app_version: vk_make_version!(1, 0, 0),
                layers: layers.iter().map(|layer| layer.name.into()).collect(),
                //layers: explicit_layers,
                extensions: explicit_extensions,
            }
        })
        .unwrap();

        Ok(RenderBackendVk {
            instance: Arc::new(instance),
            adapters: Vec::new(),
            device_info: Vec::new(),
            device_map: HashMap::new(),
            devices: Vec::new(),
        })
    }
}

impl Drop for RenderBackendVk {
    fn drop(&mut self) {
        trace!("Drop called for RenderBackendVk!");
        self.devices.clear();
        self.device_map.clear();
        self.device_info.clear();
        self.adapters.clear();
    }
}

impl<'a> From<&'a PhysicalDevice> for RenderDeviceInfo {
    fn from(device: &PhysicalDevice) -> Self {
        let physical_properties = device.properties();
        let /*mut*/ caps = RenderDeviceCaps {
			..Default::default()
		};
        RenderDeviceInfo {
            name: physical_properties.device_name,
            vendor: RenderDeviceVendor::from_u32(physical_properties.vendor_id)
                .unwrap_or(RenderDeviceVendor::Unknown),
            device_id: physical_properties.device_id,
            device_index: 0,
            device_type: match physical_properties.device_type {
                vk::PhysicalDeviceType::DISCRETE_GPU => RenderDeviceType::Discrete,
                vk::PhysicalDeviceType::INTEGRATED_GPU => RenderDeviceType::Integrated,
                vk::PhysicalDeviceType::VIRTUAL_GPU => RenderDeviceType::Virtual,
                vk::PhysicalDeviceType::CPU => RenderDeviceType::Cpu,
                vk::PhysicalDeviceType::OTHER => RenderDeviceType::Other,
                _ => unimplemented!(),
            },
            caps,
        }
    }
}

impl RenderBackend for RenderBackendVk {
    fn is_initialized(&self) -> bool {
        true
    }

    fn enumerate_devices(
        &mut self,
        max_devices: u32,
        mirror_count: u32,
        software: bool,
    ) -> Result<Vec<RenderDeviceInfo>> {
        let instance = Arc::clone(&self.instance);

        if self.adapters.len() == 0 {
            self.adapters = PhysicalDevice::enumerate(instance)
                .unwrap()
                .into_iter()
                .map(|device| Arc::new(device))
                .collect();

            self.device_info.clear();
            self.device_map.clear();
            self.devices.clear();
        }

        if self.adapters.len() == 0 {
            Err(Error::backend("no physical devices detected"))
        } else {
            if self.device_info.len() == 0 {
                for device_id in 0..self.adapters.len() {
                    let adapter = Arc::clone(&self.adapters[device_id]);
                    let mut info = RenderDeviceInfo::from(&*adapter);
                    info.device_index = device_id as u32;
                    self.device_map
                        .insert(self.device_info.len() as u32, self.device_info.len() as u32);
                    self.device_info.push(info);
                }

                self.devices
                    .resize(self.device_info.len(), Arc::new(RwLock::new(None)));

                let actual_device_count = self.device_map.len();
                for mirror_index in 0..mirror_count {
                    for device_index in 0..actual_device_count {
                        let map_len = self.device_map.len() as u32;
                        self.device_map.insert(map_len, device_index as u32);
                    }
                }
            }

            Ok(self.device_info.clone())
        }
    }

    fn create_device(&mut self, device_id: RenderDeviceId) -> Result<()> {
        let device_index = device_id as usize;
        if device_index >= self.devices.len() {
            Err(Error::backend(format!(
                "no device found for id {}",
                device_index
            )))
        } else {
            let adapter = Arc::clone(&self.adapters[device_index]);
            let instance = Arc::clone(&self.instance);
            let info = self.device_info[device_index].clone();
            let device = Box::new(RenderDeviceVk::new(info, instance, adapter)?);
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
    ) -> Result<Arc<RwLock<Option<Box<RenderDevice>>>>> {
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

#[cfg(target_os = "windows")]
use ash::extensions::khr::Win32Surface;

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
use ash::extensions::khr::XlibSurface;

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
fn extension_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        XlibSurface::name().as_ptr(),
        DebugReport::name().as_ptr(),
    ]
}

#[cfg(target_os = "macos")]
fn extension_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        MacOSSurface::name().as_ptr(),
        DebugReport::name().as_ptr(),
    ]
}

#[cfg(all(windows))]
fn extension_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        Win32Surface::name().as_ptr(),
        DebugReport::name().as_ptr(),
    ]
}
