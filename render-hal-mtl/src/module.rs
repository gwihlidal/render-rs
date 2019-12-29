#![allow(dead_code)]

use crate::backend::RenderBackendMtl;
use render_core::backend::{RenderBackend, RenderBackendModule};

#[derive(Debug)]
pub(crate) struct RenderBackendModuleMtl {}

impl RenderBackendModuleMtl {
    pub fn new() -> Self {
        RenderBackendModuleMtl {}
    }
}

impl Drop for RenderBackendModuleMtl {
    fn drop(&mut self) {}
}

impl RenderBackendModule for RenderBackendModuleMtl {
    fn name(&self) -> &'static str {
        "Metal 2"
    }

    fn api(&self) -> &'static str {
        "mtl"
    }

    fn create(&self) -> Box<dyn RenderBackend> {
        Box::new(RenderBackendMtl::new())
    }
}
