#![allow(unused_imports)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate chrono;
extern crate crossbeam_channel;
extern crate fern;
extern crate relevant;
extern crate smallvec;
#[macro_use]
extern crate enum_primitive;
extern crate digest;
extern crate meowhash;
extern crate num_traits;
extern crate render_core;
#[cfg(not(target_os = "macos"))]
extern crate renderdoc;
extern crate spirv_reflect;
extern crate twox_hash;

#[macro_use]
extern crate downcast_rs;

#[macro_use]
extern crate ash;
extern crate vk_mem;
extern crate vk_sync;

#[cfg(target_os = "windows")]
extern crate winapi;

#[cfg(target_os = "macos")]
extern crate cocoa;
#[cfg(target_os = "macos")]
extern crate metal_rs as metal;
#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;
#[cfg(target_os = "macos")]
use cocoa::appkit::{NSView, NSWindow};
#[cfg(target_os = "macos")]
use cocoa::base::id as cocoa_id;
#[cfg(target_os = "macos")]
use metal::CoreAnimationLayer;
#[cfg(target_os = "macos")]
use objc::runtime::YES;
#[cfg(target_os = "macos")]
use std::mem;

#[cfg(target_os = "windows")]
use ash::extensions::khr::Win32Surface;
#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
use ash::extensions::khr::XlibSurface;
#[cfg(target_os = "macos")]
use ash::extensions::mvk::MacOSSurface;

use crate::module::RenderBackendModuleVk;
use render_core::backend::{RenderBackend, RenderBackendModule};

mod allocator;
mod backend;
mod compile;
mod debug;
mod descriptors;
mod device;
mod module;
mod queue;
mod raw;
mod shader_views;
mod types;

#[no_mangle]
pub extern "C" fn render_backend_factory() -> Box<dyn RenderBackendModule> {
    Box::new(RenderBackendModuleVk::new())
}
