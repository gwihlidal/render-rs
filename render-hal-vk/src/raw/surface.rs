#![allow(dead_code)]

use ash;
use std::{
    ffi::CStr,
    ptr::{null, null_mut},
};
//use winit::Window;

use crate::raw::device::PhysicalDevice;
use crate::raw::errors::{OomError, SurfaceError};
use crate::raw::instance::Instance;

pub struct Surface {
    pub(crate) raw: ash::vk::SurfaceKHR,
    //pub(crate) window: Window,
}

impl Surface {
    /// Surface extensions.
    /// This extension must be enabled to create surfaces.
    pub fn extensions() -> Vec<&'static str> {
        SurfaceFn::extensions()
            .into_iter()
            .map(|string| string.to_str().unwrap())
            .collect()
    }

    /// Create surface.
    pub fn create(_instance: &Instance) -> Result<Self, SurfaceError> {
        unimplemented!()
        //let raw = instance.inner.surface.as_ref().unwrap().create_surface(instance.handle(), &window)?;

        //Ok(Surface {
        //	raw,
        //	window,
        //})
    }

    /// Check if surface presentation is supported by queue family.
    pub fn supports_queue_family(
        &self,
        physical_device: &PhysicalDevice,
        family_index: u32,
    ) -> Result<bool, SurfaceError> {
        physical_device
            .instance
            .inner
            .surface
            .as_ref()
            .unwrap()
            .supports_queue_family(physical_device.raw, self.raw, family_index)
    }

    pub fn supported_formats(
        &self,
        physical_device: &PhysicalDevice,
    ) -> Result<impl IntoIterator<Item = ash::vk::Format>, SurfaceError> {
        physical_device
            .instance
            .inner
            .surface
            .as_ref()
            .unwrap()
            .supported_formats(physical_device.raw, self.raw)
    }
}

#[cfg(target_os = "macos")]
type PlatformFn = ash::vk::MvkMacosSurfaceFn;

#[cfg(target_os = "ios")]
type PlatformFn = ash::vk::IOSSurfaceFn;

#[cfg(windows)]
type PlatformFn = ash::vk::KhrWin32SurfaceFn;

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
type PlatformFn = ash::vk::KhrXlibSurfaceFn;

pub struct SurfaceFn {
    fp: ash::vk::KhrSurfaceFn,
    platform: PlatformFn,
}

impl SurfaceFn {
    /// Surface extensions.
    pub fn extensions() -> Vec<&'static CStr> {
        vec![
            ash::extensions::khr::Surface::name(),
            #[cfg(target_os = "macos")]
            ash::extensions::mvk::MacOSSurface::name(),
            #[cfg(target_os = "windows")]
            ash::extensions::khr::Win32Surface::name(),
            #[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
            ash::extensions::XlibSurface::name(),
            //ash::extensions::XcbSurface::name(),
            //ash::extensions::WaylandSurface::name(),
        ]
    }

    pub fn load<F>(mut f: F) -> Result<Self, Vec<&'static str>>
    where
        F: FnMut(&CStr) -> *const std::os::raw::c_void,
    {
        Ok(SurfaceFn {
            fp: ash::vk::KhrSurfaceFn::load(&mut f),
            platform: PlatformFn::load(&mut f),
        })
    }

    pub fn new(
        instance: ash::vk::Instance,
        entry: &ash::vk::StaticFn,
    ) -> Result<Self, Vec<&'static str>> {
        Self::load(|name| unsafe {
            ::std::mem::transmute(entry.get_instance_proc_addr(instance, name.as_ptr()))
        })
    }

    /*	#[cfg(target_os = "macos")]
    fn create_surface(
        &self,
        instance: ash::vk::Instance,
        window: &Window,
    ) -> Result<ash::vk::SurfaceKHR, SurfaceError> {
        use cocoa::appkit::NSView;
        use objc::runtime::{Class, Object, BOOL};
        use winit::os::macos::WindowExt;

        let nsview: *mut Object = window.get_nsview() as _;
        unsafe {
            let layer = NSView::layer(nsview);
            let layer_class = class!(CAMetalLayer);
            let is_kind: BOOL = msg_send![layer, isKindOfClass: layer_class];
            if is_kind == 0 {
                let render_layer: *mut Object = msg_send![layer_class, new];
                msg_send![nsview, setLayer: render_layer];
            }
        }

        let mut surface = ash::vk::SurfaceKHR::null();
        let result = unsafe {
            self.platform.create_mac_os_surface_mvk(
                instance,
                &ash::vk::MacOSSurfaceCreateInfoMVK {
                    s_type: ash::vk::StructureType::MACOS_SURFACE_CREATE_INFO_M,
                    p_next: null(),
                    flags: ash::vk::MacOSSurfaceCreateFlagsMVK::empty(),
                    p_view: nsview as _,
                },
                null(),
                &mut surface,
            )
        };

        match result {
            ash::vk::Result::SUCCESS => {
                trace!("MacOS surface created");
                Ok(surface)
            }
            _ => Err(SurfaceError::from_vk_result(result)),
        }
    }*/

    pub fn supports_queue_family(
        &self,
        physical_device: ash::vk::PhysicalDevice,
        surface: ash::vk::SurfaceKHR,
        family_index: u32,
    ) -> Result<bool, SurfaceError> {
        let mut b = 0;
        let result = unsafe {
            self.fp.get_physical_device_surface_support_khr(
                physical_device,
                family_index,
                surface,
                &mut b,
            )
        };

        match result {
            ash::vk::Result::SUCCESS => Ok(b > 0),
            error => Err(SurfaceError::from_vk_result(error)),
        }
    }

    pub fn supported_formats(
        &self,
        physical_device: ash::vk::PhysicalDevice,
        surface: ash::vk::SurfaceKHR,
    ) -> Result<impl IntoIterator<Item = ash::vk::Format>, SurfaceError> {
        unsafe {
            let mut count = 0;
            let result = self.fp.get_physical_device_surface_formats_khr(
                physical_device,
                surface,
                &mut count,
                null_mut(),
            );

            match result {
                ash::vk::Result::SUCCESS => {}
                error => return Err(SurfaceError::from_vk_result(error)),
            }

            let mut formats = Vec::with_capacity(count as usize);
            let result = self.fp.get_physical_device_surface_formats_khr(
                physical_device,
                surface,
                &mut count,
                formats.as_mut_ptr(),
            );

            match result {
                ash::vk::Result::SUCCESS => {
                    formats.set_len(count as usize);
                    Ok(formats.into_iter().map(|format| format.format))
                }
                error => Err(SurfaceError::from_vk_result(error)),
            }
        }
    }
}
