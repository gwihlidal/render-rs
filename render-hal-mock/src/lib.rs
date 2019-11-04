#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate crossbeam_channel;
extern crate smallvec;
#[macro_use]
extern crate enum_primitive;
extern crate render_core;
#[macro_use]
extern crate downcast_rs;

use crate::module::RenderBackendModuleMock;
use render_core::backend::{RenderBackend, RenderBackendModule};

mod backend;
mod device;
mod module;
mod types;

#[no_mangle]
pub extern "C" fn render_backend_factory() -> Box<dyn RenderBackendModule> {
    Box::new(RenderBackendModuleMock::new())
}
