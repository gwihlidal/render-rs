#![allow(dead_code)]

use crate::backend::RenderBackendDx12;
use render_core::backend::{RenderBackend, RenderBackendModule};

#[derive(Debug)]
pub(crate) struct RenderBackendModuleDx12 {}

impl RenderBackendModuleDx12 {
    pub fn new() -> Self {
        RenderBackendModuleDx12 {}
    }
}

impl Drop for RenderBackendModuleDx12 {
    fn drop(&mut self) {}
}

impl RenderBackendModule for RenderBackendModuleDx12 {
    fn name(&self) -> &'static str {
        "DirectX 12"
    }

    fn api(&self) -> &'static str {
        "dx12"
    }

    fn create(&self) -> Box<dyn RenderBackend> {
        Box::new(RenderBackendDx12::new())
    }
}
