use crate::device::{SET_OFFSET, SRV_OFFSET, UAV_OFFSET};
use crate::raw::device::Device;
use crate::types::get_image_layout;
use crate::types::RenderBufferVk;
use crate::types::RenderTextureVk;
use crate::types::{
    DescriptorSetLayout, RenderPipelineLayoutVk, RenderShaderViewsVk, RenderShaderVk,
};
use ash;
use ash::version::DeviceV1_0;
use render_core::resources::RenderResourceBase;
use render_core::resources::RenderResourceStorage;
use render_core::state::RenderBindingShaderResourceView;
use render_core::types::{RenderResourceHandle, RenderResourceStates, RenderResourceType};
use render_core::utilities::any_as_u8_slice;
use std::collections::HashMap;
use std::hash::Hasher;
use std::sync::Arc;

#[derive(Debug, Default, Copy, Clone)]
pub struct CachedDescriptorSet {
    pub set_index: u32,
    pub srv_layout_hash: u64,
    pub uav_layout_hash: u64,
    pub pipeline_state: RenderResourceHandle,
    pub descriptor_pool: ash::vk::DescriptorPool,
    pub descriptor_set: ash::vk::DescriptorSet,
}

#[inline]
pub fn tally_descriptor_pool_sizes(
    pool_sizes: &mut Vec<ash::vk::DescriptorPoolSize>,
    descriptor_type: ash::vk::DescriptorType,
) {
    for type_index in 0..pool_sizes.len() {
        if pool_sizes[type_index].ty == descriptor_type {
            pool_sizes[type_index].descriptor_count += 1;
            return;
        }
    }

    pool_sizes.push(ash::vk::DescriptorPoolSize {
        ty: descriptor_type,
        descriptor_count: 1,
    });
}

#[inline]
pub fn find_layout_set(layouts: &Vec<DescriptorSetLayout>, set: u32) -> i32 {
    for slot_index in 0..layouts.len() {
        if layouts[slot_index].set_index == set {
            return slot_index as i32;
        }
    }

    // TODO: return Option<>
    return -1;
}

#[inline]
pub fn find_layout_binding(layout: &DescriptorSetLayout, binding: u32) -> i32 {
    for slot_index in 0..layout.bindings.len() {
        if layout.bindings[slot_index].binding == binding {
            return slot_index as i32;
        }
    }

    // TODO: return Option<>
    return -1;
}

#[inline]
pub fn merge_descriptor_set_layouts(
    shader: &RenderShaderVk,
    combined_layouts: &mut Vec<DescriptorSetLayout>,
) {
    let shader_layouts = &shader.set_layouts;
    for layout in shader_layouts {
        let set_index = layout.0;
        if set_index == SET_OFFSET {
            continue;
        }

        // Does combined layouts already contain this set?
        let mut combined_set_index: i32 = find_layout_set(combined_layouts, set_index);
        if combined_set_index == -1 {
            // Set doesn't yet exist in global mapping - add it
            combined_layouts.push(DescriptorSetLayout {
                set_index,
                ..Default::default()
            });
            combined_set_index = (combined_layouts.len() - 1) as i32;
        } else {
            // Set already exists in global mapping
            assert!(combined_set_index >= 0);
        }

        let global_layout = &mut combined_layouts[combined_set_index as usize];

        // Merge in shader stage's bindings and check for conflicts
        let bindings = &layout.1;
        for layout_binding in bindings {
            // Ignore reflected samplers, as we exclusively use own static sampler bindings
            if layout_binding.descriptor_type == ash::vk::DescriptorType::SAMPLER {
                continue;
            }

            // Does layout binding already exist in this layout?
            let combined_binding_index: i32 =
                find_layout_binding(global_layout, layout_binding.binding);
            if combined_binding_index == -1 {
                // Binding doesn't yet exist in layout binding - add it
                global_layout.bindings.push(*layout_binding);
            } else {
                // Since descriptors are shared across all stages, the bindings must exactly match
                assert_eq!(
                    global_layout.bindings[combined_binding_index as usize].binding,
                    layout_binding.binding
                );
                assert_eq!(
                    global_layout.bindings[combined_binding_index as usize].descriptor_type,
                    layout_binding.descriptor_type
                );
                assert_eq!(
                    global_layout.bindings[combined_binding_index as usize].descriptor_count,
                    layout_binding.descriptor_count
                );
                assert!(global_layout.bindings[combined_binding_index as usize]
                    .p_immutable_samplers
                    .is_null()); // Created later
                assert!(layout_binding.p_immutable_samplers.is_null()); // Created later

                // The stage flags is the only field which is allow to differ, as this will have the bit set
                // related to each shader stage (indicating a binding is used by that stage). Simply OR them together.
                global_layout.bindings[combined_binding_index as usize].stage_flags |=
                    layout_binding.stage_flags;
            }
        }
    }
}

pub struct DescriptorSetCache {
    pub logical_device: Arc<Device>,
    pub storage: Arc<RenderResourceStorage<Box<RenderResourceBase>>>,
}

impl DescriptorSetCache {
    pub fn new(
        device: Arc<Device>,
        storage: Arc<RenderResourceStorage<Box<RenderResourceBase>>>,
    ) -> Self {
        Self {
            logical_device: device,
            storage,
        }
    }

    #[inline]
    pub fn get_descriptor_image_layout(
        &self,
        resource: RenderResourceHandle,
        resource_tracker: &mut HashMap<RenderResourceHandle, RenderResourceStates>,
    ) -> ash::vk::ImageLayout {
        // Only textures can have an image layout
        let mut image_layout = ash::vk::ImageLayout::UNDEFINED;
        if resource.get_type() == RenderResourceType::Texture {
            image_layout = match resource_tracker.get(&resource) {
                Some(&states) => {
                    // Resource is in tracker, which means it is no longer
                    // set to the default state
                    get_image_layout(states)
                }
                None => {
                    let resource_lock = self.storage.get(resource).unwrap();
                    let resource = resource_lock.read().unwrap();
                    let texture = resource.downcast_ref::<RenderTextureVk>().unwrap();
                    // Use default state
                    get_image_layout(texture.default_state)
                }
            };

            /*println!(
                "Memoized texture layout: {:?} - {:?}",
                resource, image_layout
            );*/
        }
        image_layout
    }

    pub fn memoize(
        &self,
        resource_tracker: &mut HashMap<RenderResourceHandle, RenderResourceStates>,
        set_index: u32,
        pipeline_state: RenderResourceHandle,
        layout_data: &RenderPipelineLayoutVk,
        shader_views: &mut RenderShaderViewsVk,
    ) -> Option<CachedDescriptorSet> {
        // TODO: Handle case where args are passed in but the shader doesn't define those
        // resources (dx12 doesn't fail in this case).

        let mut uav_hasher = twox_hash::XxHash32::with_seed(0);
        for uav in &shader_views.uavs {
            let resource = uav.desc.base.resource;
            let layout = self.get_descriptor_image_layout(resource, resource_tracker);
            uav_hasher.write(&any_as_u8_slice(&layout));
        }
        let uav_layout_hash: u64 = uav_hasher.finish();

        let mut srv_hasher = twox_hash::XxHash32::with_seed(0);
        for srv in &shader_views.srvs {
            let resource = srv.desc.base.resource;
            let layout = self.get_descriptor_image_layout(resource, resource_tracker);
            srv_hasher.write(&any_as_u8_slice(&layout));
        }
        let srv_layout_hash: u64 = srv_hasher.finish();

        // Cache hit?
        let mut cached_descriptor_set: Option<CachedDescriptorSet> = None;
        for cache_entry in &shader_views.cached_descriptor_sets {
            if cache_entry.pipeline_state == pipeline_state
                && cache_entry.set_index == set_index
                && cache_entry.srv_layout_hash == srv_layout_hash
                && cache_entry.uav_layout_hash == uav_layout_hash
            {
                // Hit!
                cached_descriptor_set = Some(cache_entry.clone());
                break;
            }
        }

        // Miss..
        if cached_descriptor_set.is_none() {
            // Handle case where shader views have at least one UAV or SRV, but the shader itself has
            // none defined (common when debugging and commenting out code to test things)
            if layout_data.combined_layouts.len() == 0 {
                return None;
            }

            let mut descriptor_layout: Option<DescriptorSetLayout> = None;
            for layout in &layout_data.combined_layouts {
                if layout.set_index == set_index {
                    descriptor_layout = Some(layout.clone());
                    break;
                }
            }

            if let Some(descriptor_layout) = descriptor_layout {
                let mut descriptor_set = CachedDescriptorSet {
                    pipeline_state,
                    set_index,
                    srv_layout_hash,
                    uav_layout_hash,
                    ..Default::default()
                };

                // Create descriptor pool
                let pool_info = ash::vk::DescriptorPoolCreateInfo::builder()
                    .pool_sizes(&layout_data.pool_sizes)
                    .max_sets(1)
                    .build();

                // TODO: Yeah, this is pretty inefficient :) Render graph also creates
                // shader views multiple times per frame.
                let device = self.logical_device.clone();
                descriptor_set.descriptor_pool = unsafe {
                    device
                        .device()
                        .create_descriptor_pool(&pool_info, None)
                        .unwrap()
                };

                let allocate_info = ash::vk::DescriptorSetAllocateInfo::builder()
                    .descriptor_pool(descriptor_set.descriptor_pool)
                    .set_layouts(&[descriptor_layout.layout])
                    .build();

                descriptor_set.descriptor_set = unsafe {
                    device
                        .device()
                        .allocate_descriptor_sets(&allocate_info)
                        .unwrap()
                }[0];

                let mut writes: Vec<ash::vk::WriteDescriptorSet> =
                    Vec::with_capacity(shader_views.srvs.len() + shader_views.uavs.len());
                let mut texel_buffers: Vec<ash::vk::BufferView> =
                    Vec::with_capacity(shader_views.srvs.len() + shader_views.uavs.len());
                let mut buffer_info: Vec<ash::vk::DescriptorBufferInfo> =
                    Vec::with_capacity(shader_views.srvs.len() + shader_views.uavs.len());
                let mut image_info: Vec<ash::vk::DescriptorImageInfo> =
                    Vec::with_capacity(shader_views.srvs.len() + shader_views.uavs.len());

                // Update SRV descriptors
                for srv_index in 0..shader_views.srvs.len() {
                    let srv = &shader_views.srvs[srv_index];
                    let binding_index = SRV_OFFSET + srv_index as u32;
                    let mut layout_info: Option<ash::vk::DescriptorSetLayoutBinding> = None;
                    for binding in &descriptor_layout.bindings {
                        if binding.binding == binding_index {
                            layout_info = Some(binding.clone());
                            break;
                        }
                    }

                    if let Some(layout_info) = layout_info {
                        let layout = self
                            .get_descriptor_image_layout(srv.desc.base.resource, resource_tracker);
                        let mut write = ash::vk::WriteDescriptorSet::builder();

                        match srv.desc.base.resource.get_type() {
                            RenderResourceType::Buffer => {
                                let resource_lock =
                                    self.storage.get(srv.desc.base.resource).unwrap();
                                let resource = resource_lock.read().unwrap();
                                let buffer = resource.downcast_ref::<RenderBufferVk>().unwrap();
                                if layout_info.descriptor_type
                                    != ash::vk::DescriptorType::UNIFORM_TEXEL_BUFFER
                                {
                                    let slot = buffer_info.len();
                                    buffer_info.push(ash::vk::DescriptorBufferInfo {
                                        buffer: buffer.buffer,
                                        offset: 0,
                                        range: buffer.desc.size as u64,
                                    });

                                    write = write.buffer_info(&buffer_info[slot..1]);
                                }
                            }
                            RenderResourceType::Texture => {
                                let slot = image_info.len();
                                image_info.push(ash::vk::DescriptorImageInfo {
                                    image_layout: layout,
                                    image_view: srv.image_view,
                                    ..Default::default()
                                });

                                write = write.image_info(&image_info[slot..1]);
                            }
                            _ => unimplemented!(),
                        }

                        write = write
                            .dst_set(descriptor_set.descriptor_set)
                            .dst_array_element(0)
                            .dst_binding(binding_index)
                            //.descriptor_count(1)
                            .descriptor_type(layout_info.descriptor_type);

                        if shader_views.srvs[srv_index].buffer_view != ash::vk::BufferView::null() {
                            let texel_slot = texel_buffers.len() - 1;
                            texel_buffers.push(shader_views.srvs[srv_index].buffer_view.clone());
                            write = write.texel_buffer_view(&texel_buffers[texel_slot..1]);
                        }

                        writes.push(write.build());
                    } else {
                        // This is usually a parameter specified in the arguments, but not used in the shader itself
                        continue;
                    }
                }

                // Update UAV descriptors
                for uav_index in 0..shader_views.uavs.len() {
                    let uav = &shader_views.uavs[uav_index];
                    let binding_index = UAV_OFFSET + uav_index as u32;
                    let mut layout_info: Option<ash::vk::DescriptorSetLayoutBinding> = None;
                    for binding in &descriptor_layout.bindings {
                        if binding.binding == binding_index {
                            layout_info = Some(binding.clone());
                            break;
                        }
                    }

                    if let Some(layout_info) = layout_info {
                        let layout = self
                            .get_descriptor_image_layout(uav.desc.base.resource, resource_tracker);
                        let mut write = ash::vk::WriteDescriptorSet::builder();

                        match uav.desc.base.resource.get_type() {
                            RenderResourceType::Buffer => {
                                let resource_lock =
                                    self.storage.get(uav.desc.base.resource).unwrap();
                                let resource = resource_lock.read().unwrap();
                                let buffer = resource.downcast_ref::<RenderBufferVk>().unwrap();
                                if layout_info.descriptor_type
                                    != ash::vk::DescriptorType::UNIFORM_TEXEL_BUFFER
                                {
                                    let slot = buffer_info.len();
                                    buffer_info.push(ash::vk::DescriptorBufferInfo {
                                        buffer: buffer.buffer,
                                        offset: 0,
                                        range: buffer.desc.size as u64,
                                    });

                                    write = write.buffer_info(&buffer_info[slot..1]);
                                }
                            }
                            RenderResourceType::Texture => {
                                let slot = image_info.len();
                                image_info.push(ash::vk::DescriptorImageInfo {
                                    image_layout: layout,
                                    image_view: uav.image_view,
                                    ..Default::default()
                                });

                                write = write.image_info(&image_info[slot..1]);
                            }
                            _ => unimplemented!(),
                        }

                        write = write
                            .dst_set(descriptor_set.descriptor_set)
                            .dst_array_element(0)
                            .dst_binding(binding_index)
                            //.descriptor_count(1)
                            .descriptor_type(layout_info.descriptor_type);

                        if shader_views.uavs[uav_index].buffer_view != ash::vk::BufferView::null() {
                            let texel_slot = texel_buffers.len() - 1;
                            texel_buffers.push(shader_views.uavs[uav_index].buffer_view.clone());
                            write = write.texel_buffer_view(&texel_buffers[texel_slot..1]);
                        }

                        writes.push(write.build());
                    } else {
                        // This is usually a parameter specified in the arguments, but not used in the shader itself
                        continue;
                    }
                }

                if writes.len() > 0 {
                    unsafe {
                        device.device().update_descriptor_sets(&writes, &[]);
                    }
                }

                shader_views.cached_descriptor_sets.push(descriptor_set);
                cached_descriptor_set = Some(descriptor_set);
            } else {
                // Handle case where shader views have at least one UAV or SRV, but the shader itself has
                // none defined (common when debugging and commenting out code to test things)
                return None;
            }
        }

        cached_descriptor_set
    }
}

unsafe impl Send for DescriptorSetCache {}
unsafe impl Sync for DescriptorSetCache {}
