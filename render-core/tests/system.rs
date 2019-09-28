extern crate render_core;
//use render_core::backend::*;
//use render_core::commands::*;
//use render_core::device::*;
//use render_core::encoder::*;
//use render_core::handles::*;
use render_core::system::*;
//use render_core::types::*;
//use render_core::utilities::*;
//use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
mod common;

#[test]
fn system_initialize() {
    let render_system = Arc::new(RwLock::new(RenderSystem::new()));

    // Initialize
    {
        let mut rs_write = render_system.write().unwrap();
        rs_write
            .initialize(
                &common::get_render_module_path(),
                &common::get_render_backend_settings(),
            )
            .unwrap();
        assert!(rs_write.is_initialized());
        let registry = &rs_write.get_registry().unwrap();
        assert!(registry.len() > 0);
    }

    // Release
    {
        let mut rs_write = render_system.write().unwrap();
        rs_write.release().expect("failed to release render system");
    }
}

#[test]
fn system_harness() {
    common::SystemHarness::new();
}

#[test]
fn system_multi_harness() {
    common::SystemHarness::new();
    common::SystemHarness::new();
    common::SystemHarness::new();
}

#[test]
fn system_enumerate() {
    let harness = common::SystemHarness::new();
    let registry = &harness.render_system.get_registry().unwrap();

    assert!(registry.len() > 0);

    for entry in registry.iter() {
        let device_info = Arc::new(
            harness
                .render_system
                .enumerate_devices(&entry, false, None, None)
                .unwrap(),
        );
        let info_list = Arc::clone(&device_info);
        assert!(info_list.len() > 0);
    }
}

#[test]
fn system_devices() {
    let harness = common::SystemHarness::new();
    let registry = &harness.render_system.get_registry().unwrap();
    assert!(registry.len() > 0);

    for entry in registry.iter() {
        let device_info = Arc::new(
            harness.render_system
                .enumerate_devices(&entry, false, None, None)
                .unwrap(),
        );
        let info_list = Arc::clone(&device_info);
        assert!(info_list.len() > 0);
        for info in &*info_list {
            assert!(harness.render_system.create_device(&entry, info.device_index).is_ok());
            assert!(harness.render_system.get_device(&entry, info.device_index).is_ok());
            assert!(harness.render_system.destroy_device(&entry, info.device_index).is_ok());
        }
    }
}
