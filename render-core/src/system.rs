use crate::backend::{RenderBackendModule, RenderBackendRegistry, RenderBackendSettings};
use crate::device::{RenderDevice, RenderDeviceId, RenderDeviceInfo};
use crate::error::{Error, Result};
use crate::handles::RenderResourceHandle;
use crate::handles::RenderResourceHandleAllocator;
use crate::modules::{create_backend_module, load_backend_modules};
use crate::types::RenderResourceType;
use failure::Fail;
use libloading::Library;
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};

pub struct RenderSystem {
    handles: Arc<RwLock<RenderResourceHandleAllocator>>,
    registry: Arc<RwLock<Vec<RenderBackendRegistry>>>,
    libraries: Vec<Box<Library>>,
    modules: Vec<Box<dyn RenderBackendModule>>,
    names: Arc<RwLock<HashMap<RenderResourceHandle, Cow<'static, str>>>>,
    // TODO: RenderResourceHeap<std::string> resourceNames[int32(RenderResourceType::Count)];
}

impl Drop for RenderSystem {
    fn drop(&mut self) {
        self.release()
            .expect("failed to called release on drop of render system");
    }
}

impl RenderSystem {
    pub fn new() -> Self {
        RenderSystem {
            handles: Arc::new(RwLock::new(RenderResourceHandleAllocator::new())),
            registry: Arc::new(RwLock::new(Vec::new())),
            libraries: Vec::new(),
            modules: Vec::new(),
            names: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // System Management
    pub fn initialize(
        &mut self,
        module_path: &Path,
        params: &[RenderBackendSettings],
    ) -> Result<()> {
        if self.is_initialized() {
            self.release()?;
        }

        self.libraries = load_backend_modules(&module_path)?;
        for library in &self.libraries {
            self.modules.push(create_backend_module(&library)?);
        }

        if self.modules.len() == 0 {
            return Err(Error::backend("no render backend modules found"));
        }

        let registry_arc = Arc::clone(&self.registry);
        let mut registry_write = registry_arc.write().unwrap();

        // Create matching backends
        for module in &self.modules {
            let module_api = module.api();
            if module_api.len() > 0 {
                for settings in params.iter() {
                    if settings.api.len() == 0 || settings.api == module_api {
                        registry_write.push(RenderBackendRegistry {
                            settings: settings.clone(),
                            backend: Arc::new(RwLock::new(module.create())),
                        });
                    }
                }
            }
        }

        if registry_write.len() == 0 {
            return Err(Error::backend(format!(
                "no render backend was created - available: {:?}",
                self.modules
            )));
        }

        Ok(())
    }

    pub fn release(&mut self) -> Result<()> {
        let registry_arc = Arc::clone(&self.registry);
        let mut registry_write = registry_arc.write().unwrap();
        registry_write.clear();
        self.modules.clear();
        self.libraries.clear();
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        let registry_arc = Arc::clone(&self.registry);
        let registry_read = registry_arc.read().unwrap();
        registry_read.len() > 0
    }

    pub fn get_registry(&self) -> Result<Arc<RwLock<Vec<RenderBackendRegistry>>>> {
        if !self.is_initialized() {
            Err(Error::backend(
                "render system must be initialized before calling get_registry",
            ))
        } else {
            let registry_arc = Arc::clone(&self.registry);
            Ok(registry_arc)
        }
    }

    /*pub fn get_backend_info(&self) -> Result<&Box<RenderBackendRegistry>> {
        if !self.is_initialized() {
            Err(Error::backend(
                "render system must be initialized before calling get_registry",
            ))
        } else {
            Ok(&self.registry[0])
        }
    }*/

    // Device Management
    pub fn enumerate_devices(
        &mut self,
        registry: &RenderBackendRegistry,
        allow_software: bool,
        max_devices: Option<u32>,
        mirror_count: Option<u32>,
    ) -> Result<Vec<RenderDeviceInfo>> {
        if !self.is_initialized() {
            Err(Error::backend(
                "render system must be initialized before calling enumerate_devices",
            ))
        } else {
            let backend_arc = Arc::clone(&registry.backend);
            let mut backend_write = backend_arc.write().unwrap();
            Ok(backend_write.enumerate_devices(
                max_devices.unwrap_or(u32::max_value()),
                mirror_count.unwrap_or(0),
                allow_software,
            )?)
        }
    }

    pub fn create_device(
        &mut self,
        registry: &RenderBackendRegistry,
        device_index: RenderDeviceId,
    ) -> Result<Arc<RwLock<Option<Box<dyn RenderDevice>>>>> {
        if !self.is_initialized() {
            Err(Error::backend(
                "render system must be initialized before calling create_device",
            ))
        } else {
            let backend_arc = Arc::clone(&registry.backend);
            let mut backend_write = backend_arc.write().unwrap();
            backend_write.create_device(device_index)?;
            Ok(backend_write.get_device(device_index)?)
        }
    }

    pub fn destroy_device(
        &self,
        registry: &RenderBackendRegistry,
        device_index: RenderDeviceId,
    ) -> Result<()> {
        if !self.is_initialized() {
            Err(Error::backend(
                "render system must be initialized before calling destroy_device",
            ))
        } else {
            let backend_arc = Arc::clone(&registry.backend);
            let mut backend_write = backend_arc.write().unwrap();
            Ok(backend_write.destroy_device(device_index)?)
        }
    }

    pub fn get_device(
        &self,
        registry: &RenderBackendRegistry,
        device_index: RenderDeviceId,
    ) -> Result<Arc<RwLock<Option<Box<dyn RenderDevice>>>>> {
        if !self.is_initialized() {
            Err(Error::backend(
                "render system must be initialized before calling get_device",
            ))
        } else {
            let backend_arc = Arc::clone(&registry.backend);
            let backend_read = backend_arc.read().unwrap();
            Ok(backend_read.get_device(device_index)?)
        }
    }

    // Handle Management
    pub fn get_handle_allocator(&self) -> Result<Arc<RwLock<RenderResourceHandleAllocator>>> {
        if !self.is_initialized() {
            Err(Error::backend(
                "render system must be initialized before calling get_handle_allocator",
            ))
        } else {
            let lock = Arc::clone(&self.handles);
            Ok(lock)
        }
    }

    pub fn is_handle_valid(&self, handle: RenderResourceHandle) -> Result<bool> {
        if !self.is_initialized() {
            Err(Error::backend(
                "render system must be initialized before calling is_handle_valid",
            ))
        } else {
            let lock = Arc::clone(&self.handles);
            let read = lock.read().unwrap();
            Ok(read.is_valid(&handle))
        }
    }

    pub fn create_swap_chain_handle(
        &self,
        resource_name: Cow<'static, str>,
    ) -> Result<RenderResourceHandle> {
        self.create_handle(RenderResourceType::SwapChain, resource_name)
    }

    pub fn create_buffer_handle(
        &self,
        resource_name: Cow<'static, str>,
    ) -> Result<RenderResourceHandle> {
        self.create_handle(RenderResourceType::Buffer, resource_name)
    }

    pub fn create_texture_handle(
        &self,
        resource_name: Cow<'static, str>,
    ) -> Result<RenderResourceHandle> {
        self.create_handle(RenderResourceType::Texture, resource_name)
    }

    pub fn create_sampler_state_handle(
        &self,
        resource_name: Cow<'static, str>,
    ) -> Result<RenderResourceHandle> {
        self.create_handle(RenderResourceType::SamplerState, resource_name)
    }

    pub fn create_shader_handle(
        &self,
        resource_name: Cow<'static, str>,
    ) -> Result<RenderResourceHandle> {
        self.create_handle(RenderResourceType::Shader, resource_name)
    }

    pub fn create_shader_views_handle(
        &self,
        resource_name: Cow<'static, str>,
    ) -> Result<RenderResourceHandle> {
        self.create_handle(RenderResourceType::ShaderViews, resource_name)
    }

    pub fn create_ray_tracing_program_handle(
        &self,
        resource_name: Cow<'static, str>,
    ) -> Result<RenderResourceHandle> {
        self.create_handle(RenderResourceType::RayTracingProgram, resource_name)
    }

    pub fn create_ray_tracing_geometry_handle(
        &self,
        resource_name: Cow<'static, str>,
    ) -> Result<RenderResourceHandle> {
        self.create_handle(RenderResourceType::RayTracingGeometry, resource_name)
    }

    pub fn create_ray_tracing_acceleration_handle(
        &self,
        resource_name: Cow<'static, str>,
    ) -> Result<RenderResourceHandle> {
        self.create_handle(RenderResourceType::RayTracingAcceleration, resource_name)
    }

    pub fn create_ray_tracing_pipeline_state_handle(
        &self,
        resource_name: Cow<'static, str>,
    ) -> Result<RenderResourceHandle> {
        self.create_handle(RenderResourceType::RayTracingPipelineState, resource_name)
    }

    pub fn create_ray_tracing_shader_table_handle(
        &self,
        resource_name: Cow<'static, str>,
    ) -> Result<RenderResourceHandle> {
        self.create_handle(RenderResourceType::RayTracingShaderTable, resource_name)
    }

    pub fn create_graphics_pipeline_state_handle(
        &self,
        resource_name: Cow<'static, str>,
    ) -> Result<RenderResourceHandle> {
        self.create_handle(RenderResourceType::GraphicsPipelineState, resource_name)
    }

    pub fn create_compute_pipeline_state_handle(
        &self,
        resource_name: Cow<'static, str>,
    ) -> Result<RenderResourceHandle> {
        self.create_handle(RenderResourceType::ComputePipelineState, resource_name)
    }

    pub fn create_draw_binding_set_handle(
        &self,
        resource_name: Cow<'static, str>,
    ) -> Result<RenderResourceHandle> {
        self.create_handle(RenderResourceType::DrawBindingSet, resource_name)
    }

    pub fn create_frame_binding_set_handle(
        &self,
        resource_name: Cow<'static, str>,
    ) -> Result<RenderResourceHandle> {
        self.create_handle(RenderResourceType::FrameBindingSet, resource_name)
    }

    pub fn create_render_pass_handle(
        &self,
        resource_name: Cow<'static, str>,
    ) -> Result<RenderResourceHandle> {
        self.create_handle(RenderResourceType::RenderPass, resource_name)
    }

    pub fn create_command_list_handle(
        &self,
        resource_name: Cow<'static, str>,
    ) -> Result<RenderResourceHandle> {
        self.create_handle(RenderResourceType::CommandList, resource_name)
    }

    pub fn create_fence_handle(
        &self,
        resource_name: Cow<'static, str>,
    ) -> Result<RenderResourceHandle> {
        self.create_handle(RenderResourceType::Fence, resource_name)
    }

    pub fn create_timing_heap_handle(
        &self,
        resource_name: Cow<'static, str>,
    ) -> Result<RenderResourceHandle> {
        self.create_handle(RenderResourceType::TimingHeap, resource_name)
    }

    pub fn create_handle(
        &self,
        resource_type: RenderResourceType,
        resource_name: Cow<'static, str>,
    ) -> Result<RenderResourceHandle> {
        if !self.is_initialized() {
            Err(Error::backend(
                "render system must be initialized before calling create_handle",
            ))
        } else {
            if resource_name.len() == 0 {
                Err(Error::backend(
                    "resource name must be valid when calling create_handle",
                ))
            } else {
                let handle = {
                    let lock = Arc::clone(&self.handles);
                    let mut write = lock.write().unwrap();
                    write.allocate(resource_type)
                };

                let lock = Arc::clone(&self.names);
                let mut write = lock.write().unwrap();
                write.insert(handle, resource_name);
                Ok(handle)
            }
        }
    }

    pub fn destroy_handle(&self, handle: RenderResourceHandle) -> Result<()> {
        if !self.is_initialized() {
            Err(Error::backend(
                "render system must be initialized before calling destroy_handle",
            ))
        } else {
            if !self.is_handle_valid(handle)? {
                Err(Error::backend(
                    "resource handle must be valid when calling destroy_handle",
                ))
            } else {
                {
                    let lock = Arc::clone(&self.handles);
                    let mut write = lock.write().unwrap();
                    write.release(handle);
                }

                let lock = Arc::clone(&self.names);
                let mut write = lock.write().unwrap();
                write.remove(&handle);
                Ok(())
            }
        }
    }

    pub fn get_handle_name(&self, _handle: RenderResourceHandle) -> Result<String> {
        unimplemented!()
    }

    // Diagnostics
    pub fn begin_debug_capture(&self, _name: &str) -> Result<()> {
        unimplemented!()
    }

    pub fn finish_debug_capture(&self) -> Result<()> {
        unimplemented!()
    }

    pub fn trigger_debug_capture(&self) -> Result<()> {
        unimplemented!()
    }

    pub fn launch_debug_capture(&self, _quit: bool) -> Result<()> {
        unimplemented!()
    }
}
