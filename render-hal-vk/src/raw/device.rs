#![allow(dead_code)]
#![allow(unused_variables)]

use std::fmt;

use ash::{
    self,
    version::{DeviceV1_0, EntryV1_0, InstanceV1_0},
};
use std::{
    any::Any,
    borrow::Borrow,
    collections::LinkedList,
    ffi::{CStr, CString},
    ops::{Deref, Range},
    ptr::null,
    sync::Arc,
};
//use render_core::error::{Error, Result};
use crate::raw::errors::*;
use crate::raw::escape::Terminal;
use crate::raw::instance::Instance;
use crate::raw::object::VulkanObjects;
use crate::raw::surface::{Surface, SurfaceFn};
use relevant::Relevant;
//use winit::Window;

use crate::raw::buffer;
use crate::raw::command;
use crate::raw::format;
use crate::raw::image;
use crate::raw::memory;

/// Properties of the physical device.
#[derive(Clone, Debug)]
pub struct PhysicalDeviceProperties {
    pub api_version: u32,
    pub driver_version: u32,
    pub vendor_id: u32,
    pub device_id: u32,
    pub device_type: ash::vk::PhysicalDeviceType,
    pub device_name: String,
    pub pipeline_cache_uuid: [u8; 16],
    pub limits: ash::vk::PhysicalDeviceLimits,
    pub sparse_properties: ash::vk::PhysicalDeviceSparseProperties,
}

/// Properties of the command queue family.
#[derive(Clone, Copy, Debug)]
pub struct QueueFamilyProperties {
    pub index: u32,
    pub capability: command::Capability,
    pub queue_count: u32,
}

/// Request for creating command queues.
#[derive(Clone, Copy, Default, Debug)]
pub struct CreateQueueFamily {
    pub family: u32,
    pub count: u32,
}

//#[derive(Clone, Debug)]
pub struct PhysicalDevice {
    pub(crate) instance: Arc<Instance>,
    pub(crate) raw: ash::vk::PhysicalDevice,
    pub(crate) memory_properties: ash::vk::PhysicalDeviceMemoryProperties,
}

impl PhysicalDevice {
    /// Enumerate physical devices
    pub fn enumerate(
        instance: Arc<Instance>,
    ) -> Result<impl IntoIterator<Item = PhysicalDevice>, InstanceError> {
        let physicals = unsafe {
            instance
                .get_instance()
                .enumerate_physical_devices()
                .map_err(InstanceError::from_vk_result)?
        };
        trace!("Physical device enumerated");
        Ok(physicals.into_iter().map(move |physical| {
            let memory_properties =
                unsafe { instance.get_physical_device_memory_properties(physical) };
            PhysicalDevice {
                instance: Arc::clone(&instance),
                raw: physical,
                memory_properties,
            }
        }))
    }

    pub fn properties(&self) -> PhysicalDeviceProperties {
        let instance = self.instance.get_instance();
        let instance = &*instance;

        let properties = unsafe { instance.get_physical_device_properties(self.raw) };
        PhysicalDeviceProperties {
            api_version: properties.api_version,
            driver_version: properties.driver_version,
            vendor_id: properties.vendor_id,
            device_id: properties.device_id,
            device_type: properties.device_type,
            device_name: unsafe {
                CStr::from_ptr(&properties.device_name[0])
                    .to_str()
                    .unwrap()
                    .to_string()
            },
            pipeline_cache_uuid: properties.pipeline_cache_uuid,
            limits: properties.limits.clone(),
            sparse_properties: properties.sparse_properties.clone(),
        }
    }

    pub fn families(&self) -> impl IntoIterator<Item = QueueFamilyProperties> {
        let instance = self.instance.get_instance();
        let instance = &*instance;
        unsafe {
            instance
                .get_physical_device_queue_family_properties(self.raw)
                .into_iter()
                .enumerate()
                .map(|(index, properties)| QueueFamilyProperties {
                    index: index as u32,
                    capability: properties.queue_flags.into(),
                    queue_count: properties.queue_count,
                })
        }
    }

    pub fn extensions(&self) -> Result<impl IntoIterator<Item = String>, InstanceError> {
        let instance = self.instance.get_instance();
        let instance = &*instance;

        let properties = unsafe {
            instance
                .enumerate_device_extension_properties(self.raw)
                .map_err(InstanceError::from_vk_result)?
        };

        Ok(properties.into_iter().map(|extension| unsafe {
            CStr::from_ptr(&extension.extension_name[0])
                .to_str()
                .unwrap()
                .to_string()
        }))
    }

    pub fn features(&self) -> ash::vk::PhysicalDeviceFeatures {
        let instance = self.instance.get_instance();
        let instance = &*instance;
        unsafe { instance.get_physical_device_features(self.raw) }
    }
}

//#[derive(Clone, Debug)]
pub struct Device {
    //pub(crate) fp: Arc<ash::vk::DeviceFnV1_0>,
    pub(crate) raw: ash::Device,
    pub(crate) instance: Arc<Instance>,
    pub(crate) physical: ash::vk::PhysicalDevice,
    pub(crate) families: Vec<command::Family>,
    pub(crate) terminal: Terminal,
    pub(crate) tracker: Option<DeviceTracker>,
    pub(crate) swap_chain: Option<ash::vk::KhrSwapchainFn>,
    pub(crate) debug_marker: Option<ash::extensions::ext::DebugMarker>,
}

impl fmt::Debug for Device {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TODO: Device")
    }
}

impl Device {
    /// Create device from given physical device.
    pub fn create<Q, E>(
        physical_device: Arc<PhysicalDevice>,
        families: Q,
        extensions: E,
        features: ash::vk::PhysicalDeviceFeatures,
    ) -> Result<Self, DeviceError>
    where
        Q: IntoIterator,
        Q::Item: Borrow<CreateQueueFamily>,
        E: IntoIterator<Item = String>,
    {
        debug!(
            "Create device for physical device: {:#?}",
            physical_device.properties()
        );

        let families = families
            .into_iter()
            .map(|cqi| cqi.borrow().clone())
            .collect::<Vec<_>>();

        debug!("Families for create: {:#?}", &families);

        let max_queues = families.iter().map(|cqi| cqi.count).max().unwrap_or(0);
        let priorities = vec![1f32; max_queues as usize];

        let queue_create_infos = families
            .iter()
            .map(|cqi| ash::vk::DeviceQueueCreateInfo {
                s_type: ash::vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                p_next: null(),
                flags: ash::vk::DeviceQueueCreateFlags::empty(),
                queue_family_index: cqi.family,
                queue_count: cqi.count,
                p_queue_priorities: priorities.as_ptr(),
            })
            .collect::<Vec<_>>();

        let extensions = extensions
            .into_iter()
            .map(|extension| CString::new(extension).unwrap())
            .collect::<Vec<_>>();

        let swap_chain_enabled = extensions
            .iter()
            .find(|&name| &**name == ash::extensions::khr::Swapchain::name())
            .is_some();
        debug!("Swap Chain enabled: {}", swap_chain_enabled);

        debug!("Enabling extensions: {:#?}", &extensions);

        let enabled_extensions = extensions
            .iter()
            .map(|string| string.as_ptr())
            .collect::<Vec<_>>();

        let instance = physical_device.instance.get_instance();

        let device = unsafe {
            instance
                .create_device(
                    physical_device.raw,
                    &ash::vk::DeviceCreateInfo {
                        s_type: ash::vk::StructureType::DEVICE_CREATE_INFO,
                        p_next: null(),
                        flags: ash::vk::DeviceCreateFlags::empty(),
                        queue_create_info_count: queue_create_infos.len() as u32,
                        p_queue_create_infos: queue_create_infos.as_ptr(),
                        enabled_layer_count: 0,
                        pp_enabled_layer_names: null(),
                        enabled_extension_count: enabled_extensions.len() as u32,
                        pp_enabled_extension_names: enabled_extensions.as_ptr() as _,
                        p_enabled_features: &features,
                    },
                    None,
                )
                .unwrap()
            // TODO: .map_err(DeviceError::from_device_error)?
        };

        //let fp = Arc::new(device.fp_v1_0().clone());
        let raw = device.handle();
        debug!("Device {:?} created", raw);

        let swap_chain = if swap_chain_enabled {
            Some(ash::vk::KhrSwapchainFn::load(|name| unsafe {
                ::std::mem::transmute(instance.get_device_proc_addr(raw, name.as_ptr()))
            }))
        } else {
            None
        };

        let families = families
            .iter()
            .map(|cqi| {
                let id = command::FamilyId {
                    index: cqi.family,
                    capability: physical_device
                        .families()
                        .into_iter()
                        .nth(cqi.family as usize)
                        .unwrap()
                        .capability,
                };
                unsafe {
                    // Uses same values that was used in `Instance::create_device` method.
                    command::Family::from_device(
                        Arc::new(device.fp_v1_0().clone()),
                        raw,
                        id,
                        cqi.count,
                    )
                }
            })
            .collect::<Vec<_>>();

        //let debug_marker = ash::extensions::DebugMarker::new(instance, &raw);

        Ok(Device {
            raw: device,
            instance: Arc::clone(&physical_device.instance),
            physical: physical_device.raw,
            families,
            terminal: Terminal::new(),
            tracker: Some(DeviceTracker {
                relevant: Relevant,
                device: raw,
            }),
            swap_chain,
            debug_marker: None, //Some(debug_marker),
        })
    }

    pub fn instance(&self) -> Arc<Instance> {
        let instance = Arc::clone(&self.instance);
        instance
    }

    pub fn physical_device(&self) -> ash::vk::PhysicalDevice {
        self.physical
    }

    pub fn device(&self) -> &ash::Device {
        &self.raw
    }

    /// Create new buffer.
    pub fn create_buffer(
        &mut self,
        align: u64,
        size: u64,
        usage: buffer::Usage,
        properties: memory::Properties,
    ) -> buffer::Buffer {
        unimplemented!()
    }

    /// Create new image.
    pub fn create_image(
        &mut self,
        kind: image::Kind,
        format: format::Format,
        layout: image::Layout,
        usage: image::Usage,
        properties: memory::Properties,
    ) -> image::Image {
        unimplemented!()
    }

    /// Take resource tracker from the device.
    /// `DeviceTracker` is unique for `Device`.
    /// It can't be taken again until returned.
    pub fn take_tracker(&mut self) -> Option<DeviceTracker> {
        self.tracker.take()
    }

    /// Return taken `DeviceTracker`.
    /// User should return `DeviceTracker` after submitting all one-shot command buffers that used it.
    pub fn return_tracker(&mut self, tracker: DeviceTracker) {
        // assert_eq!(tracker.device, self.raw); `Eq` must be implemented for handles.
        debug_assert!(self.tracker.is_none());
        self.tracker = Some(tracker);
    }

    /// Cleanup entities.
    /// This function is expected to be called one in a while to free memory.
    /// This function expects that `DeviceTracker` wasn't taken after returned last time.
    /// Otherwise it can't guarantee that resources are not in use by the device before deleting them.
    pub fn cleanup(&mut self) {
        if self.tracker.is_some() {
            /* TODO:
            let objects = Arc::new(self.terminal.drain().collect::<VulkanObjects>());
            for queue in self.families.iter_mut().flat_map(command::Family::queues) {
                queue.push_track(objects.clone());
            }
            */
            trace!("Resources cleaned");
        } else {
            warn!("Failed to cleanup resources. `DeviceTracker` is not returned");
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        self.families.clear();
        trace!("Queues stopped");

        unsafe {
            trace!("Objects destroyed");

            self.raw.destroy_device(None);
            trace!("Device destroyed");

            let tracker = self.tracker.take().expect("Tracker must be returned");
            tracker.relevant.dispose();
        }
    }
}

/// Device resource tracker.
/// This object catches dropped resources
/// and ensures that they aren't used by device before actually destroying them.
/// It can preserve a resource for longer time than needed
/// but never destroys resource before device stops using it.
#[derive(Clone, Debug)]
pub struct DeviceTracker {
    relevant: Relevant,
    device: ash::vk::Device,
}
