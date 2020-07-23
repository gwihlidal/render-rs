use crate::backend::RenderBackendVk;
use render_core::backend::{RenderBackend, RenderBackendModule};

#[derive(Debug)]
pub(crate) struct RenderBackendModuleVk {}

impl RenderBackendModuleVk {
    pub fn new() -> Self {
        RenderBackendModuleVk {}
    }
}

impl Drop for RenderBackendModuleVk {
    fn drop(&mut self) {}
}

impl RenderBackendModule for RenderBackendModuleVk {
    fn name(&self) -> &'static str {
        "Vulkan"
    }

    fn api(&self) -> &'static str {
        "vk"
    }

    fn create(&self) -> Box<dyn RenderBackend> {
        // TODO: Proper result here
        let backend = RenderBackendVk::new();
        match backend {
            Ok(backend) => Box::new(backend),
            Err(err) => {
                panic!("error occurred: {:?}", err);
            }
        }
    }
}
