use crate::backend::RenderBackendMock;
use render_core::backend::{RenderBackend, RenderBackendModule};

#[derive(Debug)]
pub(crate) struct RenderBackendModuleMock {}

impl RenderBackendModuleMock {
    pub fn new() -> Self {
        RenderBackendModuleMock {}
    }
}

impl Drop for RenderBackendModuleMock {
    fn drop(&mut self) {}
}

impl RenderBackendModule for RenderBackendModuleMock {
    fn name(&self) -> &'static str {
        "Mock"
    }

    fn api(&self) -> &'static str {
        "mock"
    }

    fn create(&self) -> Box<dyn RenderBackend> {
        Box::new(RenderBackendMock::new().unwrap())
    }
}
