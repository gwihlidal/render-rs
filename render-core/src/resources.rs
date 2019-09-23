use crate::error::{Error, Result};
use crate::handles::RenderResourceHandle;
use crate::types::RenderResourceType;
use downcast_rs::Downcast;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};

pub trait RenderResourceBase: Downcast + fmt::Debug {
    fn get_type(&self) -> RenderResourceType;
    fn get_name(&self) -> &str;
}

impl_downcast!(RenderResourceBase);

pub struct RenderResourceStorage<T> {
    resources: Arc<RwLock<HashMap<RenderResourceHandle, Arc<RwLock<T>>>>>,
}

unsafe impl<T> Send for RenderResourceStorage<T> {}
unsafe impl<T> Sync for RenderResourceStorage<T> {}

impl<T> RenderResourceStorage<T> {
    pub fn new() -> Self {
        RenderResourceStorage {
            resources: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    #[inline(always)]
    pub fn put(&self, handle: RenderResourceHandle, resource: Arc<RwLock<T>>) -> Result<()> {
        let resources = Arc::clone(&self.resources);
        let mut resources_write = resources.write().unwrap();
        resources_write.insert(handle, resource);
        Ok(())
    }

    #[inline(always)]
    pub fn get(&self, handle: RenderResourceHandle) -> Result<Arc<RwLock<T>>> {
        let resources = Arc::clone(&self.resources);
        let resources_read = resources.read().unwrap();
        match resources_read.get(&handle) {
            Some(ref resource) => Ok(Arc::clone(&resource)),
            _ => Err(Error::backend(format!("resource not found: {:?}", handle))),
        }
    }

    #[inline(always)]
    pub fn get_or_none(&self, handle: RenderResourceHandle) -> Option<Arc<RwLock<T>>> {
        let resources = Arc::clone(&self.resources);
        let resources_read = resources.read().unwrap();
        match resources_read.get(&handle) {
            Some(ref resource) => Some(Arc::clone(&resource)),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn remove(&self, handle: RenderResourceHandle) -> Result<Arc<RwLock<T>>> {
        let resources = Arc::clone(&self.resources);
        let mut resources_write = resources.write().unwrap();
        match resources_write.remove(&handle) {
            Some(ref resource) => Ok(Arc::clone(&resource)),
            _ => Err(Error::backend(format!("resource not found: {:?}", handle))),
        }
    }

    #[inline(always)]
    pub fn valid(&self, handle: RenderResourceHandle) -> bool {
        let resources = Arc::clone(&self.resources);
        let resources_read = resources.read().unwrap();
        match resources_read.get(&handle) {
            Some(_) => true,
            _ => false,
        }
        /*match handle.get_type() {
            RenderResourceType::SwapChain -> {

            },
            RenderResourceType::Buffer -> {

            },
            RenderResourceType::Texture -> {

            },
            RenderResourceType::SamplerState -> {

            },
            RenderResourceType::Shader -> {

            },
            RenderResourceType::ShaderViews -> {

            },
            RenderResourceType::GraphicsPipelineState -> {

            },
            RenderResourceType::ComputePipelineState -> {

            },
            RenderResourceType::RayTracingGeometry -> {

            },
            RenderResourceType::RayTracingProgram -> {

            },
            RenderResourceType::RayTracingAcceleration -> {

            },
            RenderResourceType::RayTracingPipelineState -> {

            },
            RenderResourceType::RayTracingShaderTable -> {

            },
            RenderResourceType::DrawBindingSet -> {

            },
            RenderResourceType::FrameBindingSet -> {

            },
            RenderResourceType::RenderPass -> {

            },
            RenderResourceType::CommandList -> {

            },
            RenderResourceType::Fence -> {

            },
            RenderResourceType::TimingHeap -> {

            },
        }*/
    }

    /*#[inline(always)]
    pub fn get_typed<U: RenderResourceBase>(&self, handle: RenderResourceHandle) -> Result<U> {
        let resource = self.get(handle)?;
        let resource = resource.read().unwrap();
        let typed = resource.downcast_ref::<U>().unwrap();
        Ok(typed)
    }*/
}

impl<T> fmt::Debug for RenderResourceStorage<T> {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
        fmt.debug_struct("RenderResourceStorage")
            .field("TODO", &"TODO_HERE")
            .finish()
    }
}
