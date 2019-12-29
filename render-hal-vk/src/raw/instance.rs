#![allow(dead_code)]
#![allow(unused_variables)]

use crate::raw::errors::*;
use crate::raw::surface::SurfaceFn;

use ash::{
    self,
    extensions::{ext::DebugMarker, ext::DebugReport, khr::Surface, khr::Swapchain},
    version::{DeviceV1_0, EntryV1_0, InstanceV1_0},
    Entry,
};

pub type VkInstance = ash::Instance;
pub type VkEntry = ash::Entry;

#[cfg(target_os = "windows")]
use ash::extensions::khr::Win32Surface;

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
use ash::extensions::khr::XlibSurface;

#[cfg(target_os = "macos")]
use ash::extensions::mvk::MacOSSurface;

use std::ptr;

use std::{
    any::Any,
    borrow::Borrow,
    collections::LinkedList,
    ffi::{CStr, CString},
    ops::{Deref, Range},
    ptr::{null, null_mut},
    sync::Arc,
};

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

/// Config for vulkan instance.
#[derive(Clone, Debug)]
pub struct InstanceConfig {
    pub app_name: String,
    pub app_version: u32,
    pub layers: Vec<String>,
    pub extensions: Vec<String>,
}

pub(crate) struct InnerInstance {
    pub(crate) entry: VkEntry,
    pub(crate) raw: VkInstance,
    pub(crate) surface: Option<SurfaceFn>,
    pub(crate) debug_callback: Option<ash::vk::DebugReportCallbackEXT>,
    pub(crate) debug_loader: Option<DebugReport>,
    pub(crate) surface_loader: Arc<ash::extensions::khr::Surface>,
}

impl Drop for InnerInstance {
    fn drop(&mut self) {
        unsafe {
            match self.debug_loader {
                Some(ref loader) => {
                    let callback = self.debug_callback.expect("invalid callback");
                    loader.destroy_debug_report_callback(callback, None);
                }
                None => {}
            }
        }

        unsafe { self.raw.destroy_instance(None) }
    }
}

/// Vulkan instance.
#[derive(Clone)]
pub struct Instance {
    pub(crate) inner: Arc<Box<InnerInstance>>,
}

impl Deref for Instance {
    type Target = VkInstance;

    fn deref(&self) -> &VkInstance {
        &self.inner.raw
    }
}

impl Drop for Instance {
    fn drop(&mut self) {}
}

impl Instance {
    pub fn get_instance(&self) -> &VkInstance {
        &self.inner.raw
    }

    pub fn get_entry(&self) -> &VkEntry {
        &self.inner.entry
    }

    pub fn get_surface(&self) -> &Option<SurfaceFn> {
        &self.inner.surface
    }

    pub fn get_surface_loader(&self) -> &ash::extensions::khr::Surface {
        &self.inner.surface_loader
    }

    /// Create vulkan instance.
    pub fn new<F>(configure: F) -> Result<Instance, InstanceError>
    where
        F: FnOnce(&[Layer], &[Extension]) -> InstanceConfig,
    {
        let entry = VkEntry::new().map_err(InstanceError::from_loading_error)?;

        let layer_properties = entry
            .enumerate_instance_layer_properties()
            .map_err(InstanceError::from_vk_result)?;

        let extension_properties = entry
            .enumerate_instance_extension_properties()
            .map_err(InstanceError::from_vk_result)?;

        let surface_enabled;
        let debug_enabled = true;

        trace!("Properties and extensions fetched");
        let instance = unsafe {
            let layers = layer_properties
                .iter()
                .map(|layer| Layer {
                    name: CStr::from_ptr(&layer.layer_name[0]).to_str().unwrap(),
                    spec_version: layer.spec_version,
                    implementation_version: layer.implementation_version,
                    description: CStr::from_ptr(&layer.description[0]).to_str().unwrap(),
                })
                .collect::<Vec<_>>();

            let extensions = extension_properties
                .iter()
                .map(|extension| Extension {
                    name: CStr::from_ptr(&extension.extension_name[0])
                        .to_str()
                        .unwrap(),
                    spec_version: extension.spec_version,
                })
                .collect::<Vec<_>>();

            let config = configure(&layers, &extensions);

            trace!("Config acquired");
            let app_name = CString::new(config.app_name).unwrap();
            let engine_name = CString::new("render-hal-vk").unwrap();

            let app_info = ash::vk::ApplicationInfo {
                p_application_name: app_name.as_ptr(),
                s_type: ash::vk::StructureType::APPLICATION_INFO,
                p_next: ptr::null(),
                application_version: config.app_version,
                p_engine_name: engine_name.as_ptr(),
                engine_version: vk_make_version!(1, 0, 0),
                api_version: vk_make_version!(1, 0, 36),
            };

            let layers: Vec<CString> = config
                .layers
                .into_iter()
                .map(|s| CString::new(s).unwrap())
                .collect();
            let extensions: Vec<CString> = config
                .extensions
                .into_iter()
                .map(|s| CString::new(s).unwrap())
                .collect();

            surface_enabled = SurfaceFn::extensions().iter().all(|&surface_extension| {
                extensions
                    .iter()
                    .find(|&name| &**name == surface_extension)
                    .is_some()
            });

            trace!("Surface enabled: {}", surface_enabled);

            let enabled_layers: Vec<*const std::os::raw::c_char> =
                layers.iter().map(|s| s.as_ptr()).collect();

            let mut layer_names: Vec<CString> = Vec::new();
            if debug_enabled {
                layer_names.push(CString::new("VK_LAYER_LUNARG_standard_validation").unwrap());
            }

            let layers_names_raw: Vec<*const i8> = layer_names
                .iter()
                .map(|raw_name| raw_name.as_ptr())
                .collect();

            let enabled_extensions: Vec<*const std::os::raw::c_char> =
                extensions.iter().map(|s| s.as_ptr()).collect();

            let extension_names_raw = extension_names();

            let create_info = ash::vk::InstanceCreateInfo {
                s_type: ash::vk::StructureType::INSTANCE_CREATE_INFO,
                p_next: ptr::null(),
                flags: ash::vk::InstanceCreateFlags::empty(),
                p_application_info: &app_info,
                pp_enabled_layer_names: layers_names_raw.as_ptr(),
                enabled_layer_count: layers_names_raw.len() as u32,
                pp_enabled_extension_names: extension_names_raw.as_ptr(),
                enabled_extension_count: extension_names_raw.len() as u32,
                //enabled_layer_count: enabled_layers.len() as u32,
                //pp_enabled_layer_names: enabled_layers.as_ptr(),
                //enabled_extension_count: enabled_extensions.len() as u32,
                //pp_enabled_extension_names: enabled_extensions.as_ptr(),
            };

            entry
                .create_instance(&create_info, None)
                .map_err(InstanceError::from_instance_error)?
        };

        let surface = if surface_enabled {
            Some(
                SurfaceFn::new(instance.handle(), entry.static_fn())
                    .map_err(InstanceError::LoadError)?,
            )
        } else {
            None
        };

        let (debug_loader, debug_callback) = if debug_enabled {
            let debug_info = ash::vk::DebugReportCallbackCreateInfoEXT {
                flags: ash::vk::DebugReportFlagsEXT::ERROR
                    | ash::vk::DebugReportFlagsEXT::WARNING
                    | ash::vk::DebugReportFlagsEXT::PERFORMANCE_WARNING,
                pfn_callback: Some(vulkan_debug_callback),
                ..Default::default()
            };

            let debug_loader = DebugReport::new(&entry, &instance);

            let debug_callback = unsafe {
                debug_loader
                    .create_debug_report_callback(&debug_info, None)
                    .unwrap()
            };

            (Some(debug_loader), Some(debug_callback))
        } else {
            (None, None)
        };

        let surface_loader = Arc::new(ash::extensions::khr::Surface::new(&entry, &instance));

        let result = Instance {
            inner: Arc::new(Box::new(InnerInstance {
                entry,
                raw: instance,
                surface,
                debug_callback,
                debug_loader,
                surface_loader,
            })),
        };

        Ok(result)
    }
}

unsafe extern "system" fn vulkan_debug_callback(
    _: ash::vk::DebugReportFlagsEXT,
    _: ash::vk::DebugReportObjectTypeEXT,
    _: u64,
    _: usize,
    _: i32,
    _: *const std::os::raw::c_char,
    p_message: *const std::os::raw::c_char,
    _: *mut std::os::raw::c_void,
) -> u32 {
    error!("{:?}\n", CStr::from_ptr(p_message));
    ash::vk::FALSE
}
