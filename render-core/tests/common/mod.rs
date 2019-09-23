use render_core::backend::*;
//use render_core::commands::*;
use render_core::device::*;
//use render_core::encoder::*;
use render_core::handles::*;
use render_core::system::*;
//use render_core::types::*;
//use render_core::utilities::*;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

pub fn get_render_debug_flags() -> RenderDebugFlags {
    RenderDebugFlags::NONE
}

pub fn get_render_backend_settings() -> Vec<RenderBackendSettings> {
    //let backends = ["mock", "vk", "dx12", "mtl", "proxy"];
    //let backends = ["mock", "vk"];
    let backends = ["vk"];
    let mut backend_settings: Vec<RenderBackendSettings> = Vec::new();
    for backend in backends.iter() {
        backend_settings.push(RenderBackendSettings {
            api: backend.to_string(),
            address: None, // TODO: Specify for proxy
            debug_flags: get_render_debug_flags(),
        });
    }
    backend_settings
}

pub fn get_render_module_path() -> PathBuf {
    let exe_path = env::current_exe().unwrap();
    let module_path = exe_path.parent().unwrap();
    module_path.to_path_buf()
}

pub struct SystemHarness {
    pub render_system: Arc<RwLock<RenderSystem>>,
    pub device_info: Arc<Vec<RenderDeviceInfo>>,
    pub handles: Arc<RwLock<RenderResourceHandleAllocator>>,
    pub device: Arc<RwLock<Option<Box<dyn RenderDevice>>>>,
}

impl SystemHarness {
    pub fn new() -> SystemHarness {
        let render_system = Arc::new(RwLock::new(RenderSystem::new()));
        let mut harness = SystemHarness {
            render_system,
            device_info: Arc::new(Vec::new()),
            handles: Arc::new(RwLock::new(RenderResourceHandleAllocator::new())),
            device: Arc::new(RwLock::new(None)),
        };

        harness.initialize(&get_render_module_path(), &get_render_backend_settings());
        harness
    }

    pub fn initialize(&mut self, module_path: &Path, backend_settings: &[RenderBackendSettings]) {
        let mut rs_write = self.render_system.write().unwrap();
        rs_write
            .initialize(&module_path, &backend_settings)
            .unwrap();
        assert!(rs_write.is_initialized());
        let registry = Arc::clone(&rs_write.get_registry().unwrap());
        let registry_read = registry.read().unwrap();
        if registry_read.len() == 0 {
            panic!("no registry entries");
        } else {
            let backend_registry = &registry_read[0];
            self.device_info = Arc::new(
                rs_write
                    .enumerate_devices(&backend_registry, false, None, None)
                    .unwrap(),
            );
            rs_write.create_device(&backend_registry, 0).unwrap();
            self.device = rs_write.get_device(&backend_registry, 0).unwrap();
        }
    }

    pub fn release(&mut self) {
        // Need to release this reference before the render system (TODO: solve lifetimes)
        self.device = Arc::new(RwLock::new(None));
        let mut rs_write = self.render_system.write().unwrap();
        rs_write.release().expect("failed to release render system");
    }
}

impl Drop for SystemHarness {
    fn drop(&mut self) {
        self.release();
    }
}
