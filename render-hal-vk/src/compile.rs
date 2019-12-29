#![allow(dead_code)]
#![allow(unused_variables)]

use crate::descriptors::DescriptorSetCache;
use crate::raw::device::Device as RawDevice;
use crate::raw::format::get_image_aspect_flags;
use crate::types::*;
use ash;
use ash::extensions::ext::DebugMarker;
use ash::version::DeviceV1_0;
use render_core::commands::*;
use render_core::constants::*;
use render_core::device::*;
use render_core::encoder::*;
use render_core::error::{Error, Result};
use render_core::format::*;
use render_core::handles::RenderResourceHandle;
use render_core::resources::{RenderResourceBase, RenderResourceStorage};
use render_core::state::*;
use render_core::types::*;
use render_core::utilities::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use vk_sync;

struct RenderImageBarrier {
    previous_access: vk_sync::AccessType,  // AccessType::Nothing
    next_access: vk_sync::AccessType,      // AccessType::Nothing
    previous_layout: vk_sync::ImageLayout, // ImageLayout::General
    next_layout: vk_sync::ImageLayout,     // ImageLayout::General
    image: ash::vk::Image,
    image_range: ash::vk::ImageSubresourceRange,
}

struct RenderBufferBarrier {
    previous_access: vk_sync::AccessType, // AccessType::Nothing
    next_access: vk_sync::AccessType,     // AccessType::Nothing
    buffer: ash::vk::Buffer,
    offset: usize,
    size: usize,
}

pub struct RenderCompileContext {
    device: Arc<RawDevice>,
    descriptor_cache: Arc<DescriptorSetCache>,
    storage: Arc<RenderResourceStorage<Box<RenderResourceBase>>>,
    queue: Arc<RwLock<ash::vk::Queue>>,
    draw_state: RenderDrawState,
    cached_draw_state: Option<RenderDrawState>,
    cached_viewport: RenderViewportRect,
    cached_scissor: RenderScissorRect,
    cached_stencil_ref: u8,
    command_buffer: Option<Arc<ash::vk::CommandBuffer>>,
    active_render_pass: Option<Arc<RwLock<Box<RenderResourceBase>>>>,
    resource_tracker: RefCell<HashMap<RenderResourceHandle, RenderResourceStates>>,
    pending_image_barriers: HashMap<RenderResourceHandle, RenderImageBarrier>,
    pending_buffer_barriers: HashMap<RenderResourceHandle, RenderBufferBarrier>,
}

impl RenderCompileContext {
    pub fn new(
        device: Arc<RawDevice>,
        descriptor_cache: Arc<DescriptorSetCache>,
        storage: Arc<RenderResourceStorage<Box<RenderResourceBase>>>,
        queue: Arc<RwLock<ash::vk::Queue>>,
    ) -> Self {
        RenderCompileContext {
            device,
            descriptor_cache,
            storage,
            queue,
            draw_state: Default::default(),
            cached_draw_state: Default::default(),
            cached_viewport: Default::default(),
            cached_scissor: Default::default(),
            cached_stencil_ref: 0,
            command_buffer: None,
            active_render_pass: None,
            resource_tracker: RefCell::new(HashMap::new()),
            pending_image_barriers: HashMap::new(),
            pending_buffer_barriers: HashMap::new(),
        }
    }

    pub fn begin_compile(&mut self, native: &mut RenderCommandListVk) -> Result<()> {
        assert!(!native.is_open());
        assert!(self.command_buffer.is_none());
        self.command_buffer = Some(native.open()?);
        Ok(())
    }

    pub fn finish_compile(&mut self, native: &mut RenderCommandListVk) -> Result<()> {
        assert!(native.is_open());
        assert!(self.command_buffer.is_some());
        self.command_buffer = None;
        native.close()?;
        Ok(())
    }

    pub fn compile_list(
        &mut self,
        native: &mut RenderCommandListVk,
        encoder: &RenderCommandList,
    ) -> Result<()> {
        let batch_compile = native.is_open() && self.command_buffer.is_some();
        if !batch_compile {
            self.begin_compile(native)?;
        }

        assert!(self.command_buffer.is_some());
        let command_buffer = native.get()?;

        for command in encoder.get_commands() {
            match command.get_type() {
                RenderCommandType::Draw => {
                    self.draw(*command_buffer, *command)?;
                }
                RenderCommandType::DrawIndirect => {
                    self.draw_indirect(*command_buffer, *command)?;
                }
                RenderCommandType::Dispatch => {
                    self.dispatch(*command_buffer, *command)?;
                }
                RenderCommandType::DispatchIndirect => {
                    self.dispatch_indirect(*command_buffer, *command)?;
                }
                RenderCommandType::UpdateBuffer => {
                    self.update_buffer(*command_buffer, *command, &encoder)?;
                }
                RenderCommandType::UpdateTexture => {
                    self.update_texture(*command_buffer, *command, &encoder)?;
                }
                RenderCommandType::CopyBuffer => {
                    self.copy_buffer(*command_buffer, *command)?;
                }
                RenderCommandType::CopyTexture => {
                    self.copy_texture(*command_buffer, *command)?;
                }
                RenderCommandType::Barriers => {
                    self.barriers(*command_buffer, *command)?;
                }
                RenderCommandType::Transitions => {
                    self.transitions(*command_buffer, *command)?;
                }
                RenderCommandType::BeginTiming => {
                    self.begin_timing(*command_buffer, *command)?;
                }
                RenderCommandType::EndTiming => {
                    self.end_timing(*command_buffer, *command)?;
                }
                RenderCommandType::ResolveTimings => {
                    self.resolve_timings(*command_buffer, *command)?;
                }
                RenderCommandType::BeginEvent => {
                    self.begin_event(*command_buffer, *command)?;
                }
                RenderCommandType::EndEvent => {
                    self.end_event(*command_buffer, *command)?;
                }
                RenderCommandType::BeginRenderPass => {
                    self.begin_render_pass(*command_buffer, *command)?;
                }
                RenderCommandType::EndRenderPass => {
                    self.end_render_pass(*command_buffer, *command)?;
                }
                RenderCommandType::RayTrace => {
                    self.ray_trace(*command_buffer, *command)?;
                }
                RenderCommandType::UpdateTopLevelAcceleration => {
                    self.update_top_level_acceleration(*command_buffer, *command)?;
                }
                RenderCommandType::UpdateBottomLevelAcceleration => {
                    self.update_bottom_level_acceleration(*command_buffer, *command)?;
                }
                RenderCommandType::UpdateShaderTable => {
                    self.update_shader_table(*command_buffer, *command)?;
                }
            }
        }

        // Reset to defaults between command lists
        self.apply_default_state(*command_buffer)?;

        if !batch_compile {
            self.finish_compile(native)?;
        }

        Ok(())
    }

    #[inline(always)]
    fn draw(&mut self, native: ash::vk::CommandBuffer, command: &RenderCommand) -> Result<()> {
        let command_ptr = command as *const RenderCommand;
        let typed_command_ptr = command_ptr as *const RenderCommandDraw;
        let typed_command = unsafe { &*typed_command_ptr };
        let pipeline_state = self.storage.get(typed_command.pipeline_state)?;
        let pipeline_state = pipeline_state.read().unwrap();
        let pipeline_state = pipeline_state
            .downcast_ref::<RenderGraphicsPipelineStateVk>()
            .unwrap();
        self.apply_graphics_pipeline_state(native, &pipeline_state)?;
        self.apply_shader_arguments(
            native,
            typed_command.pipeline_state,
            &pipeline_state.data,
            ash::vk::PipelineBindPoint::GRAPHICS,
            &typed_command.shader_arguments,
        )?;
        self.apply_transitions(native); // TODO: Move this into the draw sections
        if let Some(ref draw_state) = typed_command.draw_state {
            self.apply_draw_state(native, draw_state)?; //, pipeline_state.primitive_topology
        }
        if let Some(draw_binding) = typed_command.draw_binding {
            let draw_binding = self.storage.get(draw_binding)?;
            let draw_binding = draw_binding.read().unwrap();
            let draw_binding = draw_binding
                .downcast_ref::<RenderDrawBindingSetVk>()
                .unwrap();
            self.apply_draw_binding(native, draw_binding)?;
            self.apply_transitions(native);
            if let Some(ref index_buffer) = draw_binding.index_buffer {
                unsafe {
                    self.device.raw.cmd_draw_indexed(
                        native,
                        typed_command.draw_packet.vertex_count,
                        typed_command.draw_packet.instance_count,
                        typed_command.draw_packet.index_offset,
                        typed_command.draw_packet.vertex_offset,
                        typed_command.draw_packet.first_instance,
                    );
                }
            } else {
                unsafe {
                    self.device.raw.cmd_draw(
                        native,
                        typed_command.draw_packet.vertex_count,
                        typed_command.draw_packet.instance_count,
                        typed_command.draw_packet.vertex_offset as u32,
                        typed_command.draw_packet.first_instance,
                    );
                }
            }
        } else {
            // Support null draw binding
            // TODO: Generate a binding for each prim type permutation?
            //commandList->IASetPrimitiveTopology(getVulkanPrimitiveTopology(command.primitive));
            // Assume triangle list, for now
            //commandList->IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
            self.apply_transitions(native);
            unsafe {
                self.device.raw.cmd_draw(
                    native,
                    typed_command.draw_packet.vertex_count,
                    typed_command.draw_packet.instance_count,
                    typed_command.draw_packet.vertex_offset as u32,
                    typed_command.draw_packet.first_instance,
                );
            }
            unimplemented!();
        }
        Ok(())
    }

    #[inline]
    fn draw_indirect(
        &mut self,
        native: ash::vk::CommandBuffer,
        command: &RenderCommand,
    ) -> Result<()> {
        error!("Calling draw_indirect - unimplemented");
        let command_ptr = command as *const RenderCommand;
        let typed_command_ptr = command_ptr as *const RenderCommandDrawIndirect;
        let typed_command = unsafe { &*typed_command_ptr };
        Ok(())
    }

    #[inline]
    fn dispatch(&mut self, native: ash::vk::CommandBuffer, command: &RenderCommand) -> Result<()> {
        let command_ptr = command as *const RenderCommand;
        let typed_command_ptr = command_ptr as *const RenderCommandDispatch;
        let typed_command = unsafe { &*typed_command_ptr };
        let pipeline_state = self.storage.get(typed_command.pipeline_state)?;
        let pipeline_state = pipeline_state.read().unwrap();
        let pipeline_state = pipeline_state
            .downcast_ref::<RenderComputePipelineStateVk>()
            .unwrap();
        self.apply_compute_pipeline_state(native, &pipeline_state)?;
        self.apply_shader_arguments(
            native,
            typed_command.pipeline_state,
            &pipeline_state.data,
            ash::vk::PipelineBindPoint::COMPUTE,
            &typed_command.shader_arguments,
        )?;
        self.apply_transitions(native); // TODO: Move this into the dispatch sections
        unsafe {
            self.device.raw.cmd_dispatch(
                native,
                typed_command.dispatch_x,
                typed_command.dispatch_y,
                typed_command.dispatch_z,
            )
        };
        Ok(())
    }

    #[inline]
    fn dispatch_indirect(
        &mut self,
        native: ash::vk::CommandBuffer,
        command: &RenderCommand,
    ) -> Result<()> {
        error!("Calling dispatch_indirect - unimplemented");
        let command_ptr = command as *const RenderCommand;
        let typed_command_ptr = command_ptr as *const RenderCommandDispatchIndirect;
        let typed_command = unsafe { &*typed_command_ptr };
        let pipeline_state = self.storage.get(typed_command.pipeline_state)?;
        let pipeline_state = pipeline_state.read().unwrap();
        let pipeline_state = pipeline_state
            .downcast_ref::<RenderComputePipelineStateVk>()
            .unwrap();
        Ok(())
    }

    #[inline]
    fn update_buffer(
        &mut self,
        native: ash::vk::CommandBuffer,
        command: &RenderCommand,
        encoder: &RenderCommandList,
    ) -> Result<()> {
        let command_ptr = command as *const RenderCommand;
        let typed_command_ptr = command_ptr as *const RenderCommandUpdateBuffer;
        let typed_command = unsafe { &*typed_command_ptr };

        let buffer = self.storage.get(typed_command.buffer)?;
        let buffer = buffer.read().unwrap();
        let buffer = buffer.downcast_ref::<RenderBufferVk>().unwrap();
        assert_ne!(buffer.buffer, ash::vk::Buffer::null());

        self.add_transition_to(typed_command.buffer, RenderResourceStates::COPY_DEST);
        self.apply_transitions(native);

        // Vulkan requires that buffer updates are no larger than 65536 bytes
        let chunk_limit = 64 * 1024;
        let chunk_count = divide_up_multiple_usize(typed_command.size, chunk_limit);
        let mut remaining_size = typed_command.size;
        for chunk_index in 0..chunk_count {
            let chunk_size = std::cmp::min(remaining_size, chunk_limit);
            let chunk_offset = chunk_index * chunk_limit;
            let chunk_window =
                encoder.get_command_data(typed_command.data + chunk_offset, chunk_size)?;
            unsafe {
                self.device.raw.cmd_update_buffer(
                    native,
                    buffer.buffer,
                    chunk_offset as u64 + typed_command.offset as u64,
                    chunk_window,
                );
            }
            remaining_size -= chunk_size;
        }

        self.add_transition_to(typed_command.buffer, buffer.default_state);
        self.apply_transitions(native);
        Ok(())
    }

    #[inline(always)]
    fn update_texture(
        &mut self,
        native: ash::vk::CommandBuffer,
        command: &RenderCommand,
        encoder: &RenderCommandList,
    ) -> Result<()> {
        error!("Calling update_texture - unimplemented");

        let command_ptr = command as *const RenderCommand;
        let typed_command_ptr = command_ptr as *const RenderCommandUpdateTexture;
        let typed_command = unsafe { &*typed_command_ptr };

        let texture = self.storage.get(typed_command.texture)?;
        let texture = texture.read().unwrap();
        let texture = texture.downcast_ref::<RenderTextureVk>().unwrap();
        assert_ne!(texture.image, ash::vk::Image::null());

        // Unsupported (currently)
        assert_ne!(texture.desc.texture_type, RenderTextureType::CubeArray);
        assert_ne!(texture.desc.texture_type, RenderTextureType::Tex1dArray);
        assert_ne!(texture.desc.texture_type, RenderTextureType::Tex2dArray);

        let mut sub_resource_count = texture.desc.levels * texture.desc.elements;
        let mut upload_size = get_texture_size(
            texture.desc.format,
            texture.desc.width,
            texture.desc.height,
            texture.desc.depth.into(),
            texture.desc.levels.into(),
            texture.desc.elements.into(),
        );
        if texture.desc.texture_type == RenderTextureType::Cube
            || texture.desc.texture_type == RenderTextureType::CubeArray
        {
            sub_resource_count *= 6;
            upload_size *= 6;
        }

        let sub_resource_range = ash::vk::ImageSubresourceRange {
            aspect_mask: ash::vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: texture.desc.levels as u32,
            base_array_layer: 0,
            layer_count: match texture.desc.texture_type {
                RenderTextureType::Cube | RenderTextureType::CubeArray => texture.desc.elements * 6,
                _ => texture.desc.elements,
            } as u32,
        };

        let mip_level = get_texture_sub_resource_slice_index(
            typed_command.sub_resource.into(),
            texture.desc.levels.into(),
        );

        let slice_index = get_texture_sub_resource_mip_index(
            typed_command.sub_resource.into(),
            texture.desc.levels.into(),
        );

        assert_eq!(
            calc_texture_sub_resource_index(mip_level, slice_index, texture.desc.levels.into()),
            typed_command.sub_resource as u32
        );

        let mip_width = texture.desc.width >> mip_level;
        let mip_height = texture.desc.height >> mip_level;

        let row_pitch = typed_command.sub_row_pitch;
        let slice_pitch = match typed_command.sub_slice_pitch {
            0 => row_pitch * texture.desc.height,
            _ => typed_command.sub_slice_pitch,
        };

        let src_data =
            encoder.get_command_data(typed_command.sub_data_mark, typed_command.sub_data_size)?;

        let layout_info = get_texture_layout_info(texture.desc.format, mip_width, mip_height);
        assert_eq!(layout_info.pitch, row_pitch);
        assert_eq!(layout_info.slice_pitch, slice_pitch);

        //auto allocation = device.getUploadAllocator().allocate(upload_size);
        //const VkDeviceSize srcOffset = allocation.offset;
        let src_offset = 0; // TODO
                            //byte* dstData = (byte*)allocation.address;
        let upload_buffer = ash::vk::Buffer::null(); // TODO

        for slice in 0..texture.desc.depth {
            //byte* rowSrcData = srcData;

            for row in 0..texture.desc.height {
                //memcpy(dstData, srcData, row_pitch);
                //dstData += row_pitch;
                //srcData += row_pitch;
            }

            //rowSrcData += slice_pitch;
            //allocation.offset += slice_pitch;
        }

        let copy_region = ash::vk::BufferImageCopy::builder()
            .buffer_row_length(row_pitch)
            .buffer_offset(src_offset)
            .buffer_image_height(mip_height)
            .image_offset(ash::vk::Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(ash::vk::Extent3D {
                width: mip_width,
                height: mip_height,
                depth: 1,
            })
            .image_subresource(ash::vk::ImageSubresourceLayers {
                aspect_mask: ash::vk::ImageAspectFlags::COLOR,
                base_array_layer: slice_index, // TODO
                layer_count: 1,
                mip_level,
            })
            .build();

        self.add_transition_to(typed_command.texture, RenderResourceStates::COPY_DEST);
        self.apply_transitions(native);

        unsafe {
            self.device.raw.cmd_copy_buffer_to_image(
                native,
                upload_buffer,
                texture.image,
                ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[copy_region],
            );
        }

        self.add_transition_to(typed_command.texture, texture.default_state);
        self.apply_transitions(native);

        Ok(())
    }

    #[inline]
    fn copy_buffer(
        &mut self,
        native: ash::vk::CommandBuffer,
        command: &RenderCommand,
    ) -> Result<()> {
        let command_ptr = command as *const RenderCommand;
        let typed_command_ptr = command_ptr as *const RenderCommandCopyBuffer;
        let typed_command = unsafe { &*typed_command_ptr };

        let src_buffer = self.storage.get(typed_command.src_buffer)?;
        let src_buffer = src_buffer.read().unwrap();
        let src_buffer = src_buffer.downcast_ref::<RenderBufferVk>().unwrap();
        assert_ne!(src_buffer.buffer, ash::vk::Buffer::null());

        let dst_buffer = self.storage.get(typed_command.dst_buffer)?;
        let dst_buffer = dst_buffer.read().unwrap();
        let dst_buffer = dst_buffer.downcast_ref::<RenderBufferVk>().unwrap();
        assert_ne!(dst_buffer.buffer, ash::vk::Buffer::null());

        self.add_transition_to(typed_command.src_buffer, RenderResourceStates::COPY_SOURCE);
        self.add_transition_to(typed_command.dst_buffer, RenderResourceStates::COPY_DEST);
        self.apply_transitions(native);

        let region = ash::vk::BufferCopy {
            src_offset: typed_command.src_offset as u64,
            dst_offset: typed_command.dst_offset as u64,
            size: typed_command.src_size as u64,
        };

        unsafe {
            self.device.raw.cmd_copy_buffer(
                native,
                src_buffer.buffer,
                dst_buffer.buffer,
                &[region],
            );
        }

        self.add_transition_to(typed_command.src_buffer, src_buffer.default_state);
        self.add_transition_to(typed_command.dst_buffer, dst_buffer.default_state);
        self.apply_transitions(native);

        Ok(())
    }

    #[inline]
    fn copy_texture(
        &mut self,
        native: ash::vk::CommandBuffer,
        command: &RenderCommand,
    ) -> Result<()> {
        let command_ptr = command as *const RenderCommand;
        let typed_command_ptr = command_ptr as *const RenderCommandCopyTexture;
        let typed_command = unsafe { &*typed_command_ptr };

        let src_texture = self.storage.get(typed_command.src_texture)?;
        let src_texture = src_texture.read().unwrap();
        let src_texture = src_texture.downcast_ref::<RenderTextureVk>().unwrap();
        assert_ne!(src_texture.image, ash::vk::Image::null());

        let dst_texture = self.storage.get(typed_command.dst_texture)?;
        let dst_texture = dst_texture.read().unwrap();
        let dst_texture = dst_texture.downcast_ref::<RenderTextureVk>().unwrap();
        assert_ne!(dst_texture.image, ash::vk::Image::null());

        self.add_transition_to(typed_command.src_texture, RenderResourceStates::COPY_SOURCE);
        self.add_transition_to(typed_command.dst_texture, RenderResourceStates::COPY_DEST);
        self.apply_transitions(native);

        let region = ash::vk::ImageCopy::builder()
            .extent(ash::vk::Extent3D {
                depth: typed_command.src_box.d as u32,
                width: typed_command.src_box.w as u32,
                height: typed_command.src_box.h as u32,
            })
            .dst_offset(ash::vk::Offset3D {
                x: typed_command.dst_point.x,
                y: typed_command.dst_point.y,
                z: typed_command.dst_point.z,
            })
            .src_subresource(ash::vk::ImageSubresourceLayers {
                aspect_mask: ash::vk::ImageAspectFlags::COLOR,
                base_array_layer: get_texture_sub_resource_slice_index(
                    typed_command.src_sub_resource.into(),
                    src_texture.desc.levels.into(),
                ),
                layer_count: 1,
                mip_level: get_texture_sub_resource_mip_index(
                    typed_command.src_sub_resource.into(),
                    src_texture.desc.levels.into(),
                ),
            })
            .dst_subresource(ash::vk::ImageSubresourceLayers {
                aspect_mask: ash::vk::ImageAspectFlags::COLOR,
                base_array_layer: get_texture_sub_resource_slice_index(
                    typed_command.dst_sub_resource.into(),
                    dst_texture.desc.levels.into(),
                ),
                layer_count: 1,
                mip_level: get_texture_sub_resource_mip_index(
                    typed_command.dst_sub_resource.into(),
                    dst_texture.desc.levels.into(),
                ),
            })
            .build();

        unsafe {
            self.device.raw.cmd_copy_image(
                native,
                src_texture.image,
                ash::vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                dst_texture.image,
                ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            );
        }

        self.add_transition_to(typed_command.src_texture, src_texture.default_state);
        self.add_transition_to(typed_command.dst_texture, dst_texture.default_state);
        self.apply_transitions(native);

        Ok(())
    }

    #[inline]
    fn barriers(&mut self, native: ash::vk::CommandBuffer, command: &RenderCommand) -> Result<()> {
        let command_ptr = command as *const RenderCommand;
        let typed_command_ptr = command_ptr as *const RenderCommandBarriers;
        let typed_command = unsafe { &*typed_command_ptr };
        if typed_command.barriers.len() > 0 {
            for barrier in &typed_command.barriers {
                self.add_uav_barrier(*barrier);
            }
            self.apply_transitions(native);
        }
        Ok(())
    }

    #[inline]
    fn transitions(
        &mut self,
        native: ash::vk::CommandBuffer,
        command: &RenderCommand,
    ) -> Result<()> {
        let command_ptr = command as *const RenderCommand;
        let typed_command_ptr = command_ptr as *const RenderCommandTransitions;
        let typed_command = unsafe { &*typed_command_ptr };
        if typed_command.transitions.len() > 0 {
            for transition in &typed_command.transitions {
                self.add_transition_to(transition.0, transition.1);
            }
            self.apply_transitions(native);
        }
        Ok(())
    }

    #[inline]
    fn begin_timing(
        &mut self,
        native: ash::vk::CommandBuffer,
        command: &RenderCommand,
    ) -> Result<()> {
        error!("Calling begin_timing - unimplemented");
        let command_ptr = command as *const RenderCommand;
        let typed_command_ptr = command_ptr as *const RenderCommandBeginTiming;
        let typed_command = unsafe { &*typed_command_ptr };
        // TODO
        Ok(())
    }

    #[inline]
    fn end_timing(
        &mut self,
        native: ash::vk::CommandBuffer,
        command: &RenderCommand,
    ) -> Result<()> {
        error!("Calling end_timing - unimplemented");
        let command_ptr = command as *const RenderCommand;
        let typed_command_ptr = command_ptr as *const RenderCommandEndTiming;
        let typed_command = unsafe { &*typed_command_ptr };
        // TODO
        Ok(())
    }

    #[inline]
    fn resolve_timings(
        &mut self,
        native: ash::vk::CommandBuffer,
        command: &RenderCommand,
    ) -> Result<()> {
        error!("Calling resolve_timings - unimplemented");
        let command_ptr = command as *const RenderCommand;
        let typed_command_ptr = command_ptr as *const RenderCommandResolveTimings;
        let typed_command = unsafe { &*typed_command_ptr };
        // TODO
        Ok(())
    }

    #[inline]
    fn begin_event(
        &mut self,
        native: ash::vk::CommandBuffer,
        command: &RenderCommand,
    ) -> Result<()> {
        if let Some(ref debug_marker) = self.device.debug_marker {
            let command_ptr = command as *const RenderCommand;
            let typed_command_ptr = command_ptr as *const RenderCommandBeginEvent;
            let typed_command = unsafe { &*typed_command_ptr };
            let event_str = typed_command.message.as_bytes();
            let event_name = std::ffi::CString::new(event_str).unwrap(); // TODO: the trait `std::convert::From<std::ffi::NulError>` is not implemented for `render_core::Error`
            let event_color = [1f32, 1f32, 1f32, 1f32];
            let marker = ash::vk::DebugMarkerMarkerInfoEXT::builder()
                .marker_name(&event_name)
                .color(event_color)
                .build();
            unsafe {
                debug_marker.cmd_debug_marker_begin(native, &marker);
            }
        }
        Ok(())
    }

    #[inline]
    fn end_event(
        &mut self,
        native: ash::vk::CommandBuffer,
        _command: &RenderCommand,
    ) -> Result<()> {
        if let Some(ref debug_marker) = self.device.debug_marker {
            //let command_ptr = command as *const RenderCommand;
            //let typed_command_ptr = command_ptr as *const RenderCommandEndEvent;
            //let typed_command = unsafe { &*typed_command_ptr };
            unsafe {
                debug_marker.cmd_debug_marker_end(native);
            }
        }
        Ok(())
    }

    #[inline]
    fn begin_render_pass(
        &mut self,
        native: ash::vk::CommandBuffer,
        command: &RenderCommand,
    ) -> Result<()> {
        let command_ptr = command as *const RenderCommand;
        let typed_command_ptr = command_ptr as *const RenderCommandBeginRenderPass;
        let typed_command = unsafe { &*typed_command_ptr };

        let render_pass = self.storage.get(typed_command.render_pass)?;
        self.active_render_pass = Some(render_pass.clone());

        let render_pass = render_pass.read().unwrap();
        let render_pass = render_pass.downcast_ref::<RenderPassVk>().unwrap();
        assert_ne!(render_pass.render_pass, ash::vk::RenderPass::null());

        let frame_binding = render_pass.frame_binding.read().unwrap();
        let frame_binding = frame_binding
            .downcast_ref::<RenderFrameBindingSetVk>()
            .unwrap();
        assert_ne!(render_pass.frame_buffer, ash::vk::Framebuffer::null());

        let view_width = frame_binding.frame_buffer_info.width;
        let view_height = frame_binding.frame_buffer_info.height;

        let rtv_base_index = match frame_binding.swap_chain {
            Some(ref swap_chain) => {
                let swap_chain = swap_chain.read().unwrap();
                let swap_chain = swap_chain.downcast_ref::<RenderSwapChainVk>().unwrap();
                swap_chain.back_buffer_index * MAX_RENDER_TARGET_COUNT as u32
            }
            None => 0,
        };

        for target_index in 0..frame_binding.render_target_count {
            if let Some(render_target) =
                frame_binding.render_target_handles[(rtv_base_index + target_index) as usize]
            {
                self.add_transition_to(render_target, RenderResourceStates::RENDER_TARGET);
            }
        }

        // Transition depth/stencil to read only if required
        if let Some(depth_stencil) = frame_binding.depth_stencil_handle {
            if let Some(ref depth_stencil_view) = frame_binding.desc.depth_stencil_view {
                if depth_stencil_view
                    .flags
                    .contains(RenderDepthStencilViewFlags::READ_ONLY_DEPTH)
                {
                    // Both stencil/depth read only is the only implemented mode, for now
                    assert!(!depth_stencil_view
                        .flags
                        .contains(RenderDepthStencilViewFlags::READ_ONLY_STENCIL));

                    self.add_transition_to(depth_stencil, RenderResourceStates::DEPTH_READ);
                } else {
                    self.add_transition_to(depth_stencil, RenderResourceStates::DEPTH_WRITE);
                }
            } else {
                unimplemented!()
            }
        }

        self.apply_transitions(native);

        let render_pass_begin_info = ash::vk::RenderPassBeginInfo {
            s_type: ash::vk::StructureType::RENDER_PASS_BEGIN_INFO,
            p_next: ::std::ptr::null(),
            render_pass: render_pass.render_pass,
            framebuffer: render_pass.frame_buffer,
            render_area: ash::vk::Rect2D {
                offset: ash::vk::Offset2D { x: 0, y: 0 },
                extent: ash::vk::Extent2D {
                    width: view_width,
                    height: view_height,
                },
            },
            clear_value_count: render_pass.clear_values.len() as u32,
            p_clear_values: render_pass.clear_values.as_ptr(),
        };

        let device = self.device.clone();
        unsafe {
            self.device.device().cmd_begin_render_pass(
                native,
                &render_pass_begin_info,
                ash::vk::SubpassContents::INLINE,
            );
        }

        Ok(())
    }

    #[inline]
    fn end_render_pass(
        &mut self,
        native: ash::vk::CommandBuffer,
        command: &RenderCommand,
    ) -> Result<()> {
        let command_ptr = command as *const RenderCommand;
        let typed_command_ptr = command_ptr as *const RenderCommandEndRenderPass;
        let typed_command = unsafe { &*typed_command_ptr };
        if let Some(active_pass) = self.active_render_pass.clone() {
            let render_pass = active_pass.clone();
            let render_pass = render_pass.read().unwrap();
            let render_pass = render_pass.downcast_ref::<RenderPassVk>().unwrap();

            let device = self.device.clone();
            unsafe {
                self.device.device().cmd_end_render_pass(native);
            }

            // Change DSV back to default state in case read-only modes were used
            if render_pass.depth_stencil_layout
                != ash::vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
            {
                let frame_binding = render_pass.frame_binding.read().unwrap();
                let frame_binding = frame_binding
                    .downcast_ref::<RenderFrameBindingSetVk>()
                    .unwrap();

                if let Some(ref depth_stencil) = frame_binding.depth_stencil_resource {
                    let depth_stencil_tex = depth_stencil.read().unwrap();
                    let depth_stencil_tex =
                        depth_stencil_tex.downcast_ref::<RenderTextureVk>().unwrap();
                    let depth_stencil_handle = frame_binding.depth_stencil_handle.unwrap();

                    self.add_transition_to(depth_stencil_handle, depth_stencil_tex.default_state);
                    self.apply_transitions(native);
                }
            }
        } else {
            unimplemented!();
        }

        self.active_render_pass = None;
        Ok(())
    }

    #[inline]
    fn ray_trace(&mut self, native: ash::vk::CommandBuffer, command: &RenderCommand) -> Result<()> {
        error!("Calling ray_trace - unimplemented");
        let command_ptr = command as *const RenderCommand;
        let typed_command_ptr = command_ptr as *const RenderCommandRayTrace;
        let typed_command = unsafe { &*typed_command_ptr };
        Ok(())
    }

    #[inline]
    fn update_top_level_acceleration(
        &mut self,
        native: ash::vk::CommandBuffer,
        command: &RenderCommand,
    ) -> Result<()> {
        error!("Calling update_top_level_acceleration - unimplemented");
        let command_ptr = command as *const RenderCommand;
        let typed_command_ptr = command_ptr as *const RenderCommandUpdateTopLevelAcceleration;
        let typed_command = unsafe { &*typed_command_ptr };
        Ok(())
    }

    #[inline]
    fn update_bottom_level_acceleration(
        &mut self,
        native: ash::vk::CommandBuffer,
        command: &RenderCommand,
    ) -> Result<()> {
        error!("Calling update_bottom_level_acceleration - unimplemented");
        let command_ptr = command as *const RenderCommand;
        let typed_command_ptr = command_ptr as *const RenderCommandUpdateBottomLevelAcceleration;
        let typed_command = unsafe { &*typed_command_ptr };
        Ok(())
    }

    #[inline]
    fn update_shader_table(
        &mut self,
        native: ash::vk::CommandBuffer,
        command: &RenderCommand,
    ) -> Result<()> {
        error!("Calling update_shader_table - unimplemented");
        let command_ptr = command as *const RenderCommand;
        let typed_command_ptr = command_ptr as *const RenderCommandUpdateShaderTable;
        let typed_command = unsafe { &*typed_command_ptr };
        Ok(())
    }

    #[inline]
    fn add_transition_to(
        &mut self,
        resource: RenderResourceHandle,
        after_state: RenderResourceStates,
    ) -> RenderResourceStates {
        let resource_name = self
            .name_for_handle(resource)
            .unwrap_or("INVALID".to_string());

        let (insert, before_state) =
            if let Some(state) = self.resource_tracker.borrow().get(&resource) {
                let before_state = *state;
                trace!(
                    "Using existing resource tracked state - {} - {}",
                    resource_name,
                    build_resource_state_text(before_state)
                );
                (false, *state)
            } else {
                let default_state = self.default_state_for_handle(resource).unwrap();
                trace!(
                    "Adding resource state to tracker - {} - {:?}",
                    resource_name,
                    build_resource_state_text(default_state)
                );
                (true, default_state)
            };

        if insert {
            self.resource_tracker
                .borrow_mut()
                .insert(resource, before_state);
        }

        if after_state != before_state {
            self.add_transition(resource, before_state, after_state);

            if let Some(entry) = self.resource_tracker.borrow_mut().get_mut(&resource) {
                *entry = after_state;
            }

            trace!(
                "Transitioning '{}' from {} to {}",
                resource_name,
                build_resource_state_text(before_state),
                build_resource_state_text(after_state)
            );
        } else {
            trace!(
                "Filtered redundant transition - {} - {}",
                resource_name,
                build_resource_state_text(after_state)
            );
        }

        before_state
    }

    #[inline]
    fn add_transition(
        &mut self,
        resource: RenderResourceHandle,
        before_state: RenderResourceStates,
        after_state: RenderResourceStates,
    ) {
        let resource_name = self
            .name_for_handle(resource)
            .unwrap_or("INVALID".to_string());
        trace!(
            "Adding transition on {} - {} -> {}",
            resource_name,
            build_resource_state_text(before_state),
            build_resource_state_text(after_state)
        );

        let resource_arc = self.storage.get(resource).unwrap();
        let resource_base = resource_arc.read().unwrap();

        if resource_base.get_type() == RenderResourceType::Texture {
            let texture = resource_base.downcast_ref::<RenderTextureVk>().unwrap();
            assert_ne!(texture.image, ash::vk::Image::null());

            let previous_access = get_image_access_type(before_state);
            let next_access = get_image_access_type(after_state);

            let barrier = RenderImageBarrier {
                previous_access,
                next_access,
                previous_layout: match previous_access {
                    vk_sync::AccessType::General => vk_sync::ImageLayout::General,
                    _ => vk_sync::ImageLayout::Optimal,
                },
                next_layout: match next_access {
                    vk_sync::AccessType::General => vk_sync::ImageLayout::General,
                    _ => vk_sync::ImageLayout::Optimal,
                },
                image: texture.image,
                image_range: ash::vk::ImageSubresourceRange {
                    aspect_mask: get_image_aspect_flags(
                        texture.desc.format,
                        false, /* ignore stencil */
                    ),
                    base_array_layer: 0,
                    base_mip_level: 0,
                    level_count: texture.desc.levels as u32,
                    layer_count: match texture.desc.texture_type {
                        RenderTextureType::Cube | RenderTextureType::CubeArray => {
                            texture.desc.elements as u32 * 6
                        }
                        _ => texture.desc.elements as u32,
                    },
                },
            };

            self.pending_image_barriers.insert(resource, barrier);
        } else if resource_base.get_type() == RenderResourceType::Buffer {
            let buffer = resource_base.downcast_ref::<RenderBufferVk>().unwrap();
            assert_ne!(buffer.buffer, ash::vk::Buffer::null());
            let previous_access = get_buffer_access_type(before_state);
            let next_access = get_buffer_access_type(after_state);
            let barrier = RenderBufferBarrier {
                previous_access,
                next_access,
                buffer: buffer.buffer,
                offset: 0,
                size: buffer.desc.size,
            };

            self.pending_buffer_barriers.insert(resource, barrier);
        } else {
            unimplemented!();
        }
    }

    #[inline]
    fn add_uav_barrier(&mut self, resource: RenderResourceHandle) {
        if self.add_transition_to(resource, RenderResourceStates::UNORDERED_ACCESS)
            == RenderResourceStates::UNORDERED_ACCESS
        {
            self.add_transition(
                resource,
                RenderResourceStates::UNORDERED_ACCESS,
                RenderResourceStates::UNORDERED_ACCESS,
            );
        }
    }

    #[inline]
    fn apply_default_state(&mut self, native: ash::vk::CommandBuffer) -> Result<()> {
        // Restore resources back to default state
        {
            let resources = &self.resource_tracker.borrow().clone();
            trace!("Applying default states: {}", resources.len());
            for (handle, states) in resources {
                let default_state = self.default_state_for_handle(*handle)?;
                self.add_transition_to(*handle, default_state);
            }
        }

        self.resource_tracker.borrow_mut().clear();
        self.apply_transitions(native);
        trace!("Done");
        Ok(())
    }

    #[inline]
    fn apply_transitions(&mut self, native: ash::vk::CommandBuffer) {
        let device = self.device.clone();
        let debugging = false;
        let total_barrier_count =
            self.pending_buffer_barriers.len() + self.pending_image_barriers.len();
        if total_barrier_count > 0 {
            let mut image_barriers: Vec<vk_sync::ImageBarrier> =
                Vec::with_capacity(self.pending_image_barriers.len());

            let mut buffer_barriers: Vec<vk_sync::BufferBarrier> =
                Vec::with_capacity(self.pending_buffer_barriers.len());

            for pending in &self.pending_image_barriers {
                assert_ne!(pending.1.image, ash::vk::Image::null());
                let barrier = vk_sync::ImageBarrier {
                    previous_accesses: vec![pending.1.previous_access],
                    next_accesses: vec![pending.1.next_access],
                    previous_layout: pending.1.previous_layout,
                    next_layout: pending.1.next_layout,
                    discard_contents: false,
                    src_queue_family_index: ash::vk::QUEUE_FAMILY_IGNORED,
                    dst_queue_family_index: ash::vk::QUEUE_FAMILY_IGNORED,
                    image: pending.1.image,
                    range: pending.1.image_range,
                };

                image_barriers.push(barrier);
            }

            for pending in &self.pending_buffer_barriers {
                assert_ne!(pending.1.buffer, ash::vk::Buffer::null());
                let barrier = vk_sync::BufferBarrier {
                    previous_accesses: vec![pending.1.previous_access],
                    next_accesses: vec![pending.1.next_access],
                    src_queue_family_index: ash::vk::QUEUE_FAMILY_IGNORED,
                    dst_queue_family_index: ash::vk::QUEUE_FAMILY_IGNORED,
                    buffer: pending.1.buffer,
                    offset: pending.1.offset,
                    size: pending.1.size,
                };
                buffer_barriers.push(barrier);
            }

            self.pending_image_barriers.clear();
            self.pending_buffer_barriers.clear();

            vk_sync::cmd::pipeline_barrier(
                &device.device().fp_v1_0(),
                native,
                None,             // global barrier
                &buffer_barriers, // buffer barriers
                &image_barriers,  // image barriers
            );
        }
    }

    #[inline]
    fn apply_draw_binding(
        &mut self,
        native: ash::vk::CommandBuffer,
        draw_binding: &RenderDrawBindingSetVk,
    ) -> Result<()> {
        let device = self.device.clone();
        if let Some(index_buffer_handle) = draw_binding.index_buffer {
            let index_buffer = self.storage.get(index_buffer_handle)?;
            let index_buffer = index_buffer.read().unwrap();
            let index_buffer = index_buffer.downcast_ref::<RenderBufferVk>().unwrap();
            self.add_transition_to(index_buffer_handle, RenderResourceStates::INDEX_BUFFER);
            unsafe {
                device.device().cmd_bind_index_buffer(
                    native,
                    index_buffer.buffer,
                    draw_binding.index_buffer_offset,
                    draw_binding.index_buffer_format,
                );
            }
        }

        let mut vertex_buffers: [ash::vk::Buffer; MAX_VERTEX_STREAMS] = Default::default();
        let mut stream_count = 0;
        for stream in 0..MAX_VERTEX_STREAMS {
            if let Some(vertex_buffer_handle) = draw_binding.vertex_buffers[stream] {
                let vertex_buffer = self.storage.get(vertex_buffer_handle)?;
                let vertex_buffer = vertex_buffer.read().unwrap();
                let vertex_buffer = vertex_buffer.downcast_ref::<RenderBufferVk>().unwrap();
                self.add_transition_to(
                    vertex_buffer_handle,
                    RenderResourceStates::VERTEX_AND_CONSTANT_BUFFER,
                );
                stream_count = std::cmp::max(stream_count, stream + 1);
                vertex_buffers[stream] = vertex_buffer.buffer;
                assert_ne!(vertex_buffers[stream], ash::vk::Buffer::null());
            }
        }

        if stream_count > 0 {
            unsafe {
                device.device().cmd_bind_vertex_buffers(
                    native,
                    0,
                    &vertex_buffers[0..stream_count],
                    &draw_binding.vertex_buffer_offsets[0..stream_count],
                );
            }
        }

        Ok(())
    }

    #[inline(always)]
    fn apply_draw_state(
        &mut self,
        native: ash::vk::CommandBuffer,
        draw_state: &RenderDrawState,
    ) -> Result<()> {
        #[inline(always)]
        fn make_viewport(viewport: &RenderViewportRect) -> ash::vk::Viewport {
            // With the KHR extension for negative viewport height, you
            // both negate the height *and* move the "origin" to the bottom left.
            ash::vk::Viewport {
                x: viewport.x,
                y: viewport.height - viewport.y,
                width: viewport.width,
                height: -viewport.height,
                min_depth: viewport.min_z,
                max_depth: viewport.max_z,
            }
        }

        #[inline(always)]
        fn make_scissor(scissor: &RenderScissorRect) -> ash::vk::Rect2D {
            ash::vk::Rect2D {
                offset: ash::vk::Offset2D {
                    x: scissor.x,
                    y: scissor.y,
                },
                extent: ash::vk::Extent2D {
                    width: scissor.width as u32,
                    height: scissor.height as u32,
                },
            }
        }

        match self.cached_draw_state {
            None => {
                // Nothing cached yet; set the cache
                self.cached_draw_state = Some(draw_state.clone());

                if let Some(ref viewport) = draw_state.viewport {
                    let viewport = make_viewport(viewport);
                    unsafe {
                        self.device.raw.cmd_set_viewport(native, 0, &[viewport]);
                    }
                }

                if let Some(ref scissor) = draw_state.scissor {
                    let scissor = make_scissor(scissor);
                    unsafe {
                        self.device.raw.cmd_set_scissor(native, 0, &[scissor]);
                    }
                }

                unsafe {
                    self.device.raw.cmd_set_stencil_reference(
                        native,
                        ash::vk::StencilFaceFlags::STENCIL_FRONT_AND_BACK,
                        draw_state.stencil_ref,
                    );
                }
            }
            Some(mut cached) => {
                if let Some(ref new_viewport) = draw_state.viewport {
                    let stale_viewport = if let Some(ref old_viewport) = cached.viewport {
                        // Viewport has changed
                        old_viewport != new_viewport
                    } else {
                        // No viewport cached
                        true
                    };
                    if stale_viewport {
                        let viewport = make_viewport(new_viewport);
                        unsafe {
                            self.device.raw.cmd_set_viewport(native, 0, &[viewport]);
                        }
                        cached.viewport = Some(*new_viewport);
                    }
                }

                if let Some(ref new_scissor) = draw_state.scissor {
                    let stale_scissor = if let Some(ref old_scissor) = cached.scissor {
                        // Scissor has changed
                        old_scissor != new_scissor
                    } else {
                        // No scissor cached
                        true
                    };
                    if stale_scissor {
                        let scissor = make_scissor(new_scissor);
                        unsafe {
                            self.device.raw.cmd_set_scissor(native, 0, &[scissor]);
                        }
                        cached.scissor = Some(*new_scissor);
                    }
                }

                if cached.stencil_ref != draw_state.stencil_ref {
                    unsafe {
                        self.device.raw.cmd_set_stencil_reference(
                            native,
                            ash::vk::StencilFaceFlags::STENCIL_FRONT_AND_BACK,
                            draw_state.stencil_ref,
                        );
                    }
                    cached.stencil_ref = draw_state.stencil_ref;
                }
            }
        }
        Ok(())
    }

    #[inline]
    fn apply_compute_pipeline_state(
        &mut self,
        native: ash::vk::CommandBuffer,
        pipeline_state: &RenderComputePipelineStateVk,
    ) -> Result<()> {
        assert_ne!(pipeline_state.pipeline, ash::vk::Pipeline::null());
        unsafe {
            // TODO: Check for redundancy
            self.device.raw.cmd_bind_pipeline(
                native,
                ash::vk::PipelineBindPoint::COMPUTE,
                pipeline_state.pipeline,
            );
        }
        Ok(())
    }

    #[inline]
    fn apply_graphics_pipeline_state(
        &mut self,
        native: ash::vk::CommandBuffer,
        pipeline_state: &RenderGraphicsPipelineStateVk,
    ) -> Result<()> {
        assert_ne!(pipeline_state.pipeline, ash::vk::Pipeline::null());
        unsafe {
            // TODO: Check for redundancy
            self.device.raw.cmd_bind_pipeline(
                native,
                ash::vk::PipelineBindPoint::GRAPHICS,
                pipeline_state.pipeline,
            );
        }
        Ok(())
    }

    #[inline]
    fn apply_shader_arguments(
        &mut self,
        native: ash::vk::CommandBuffer,
        pipeline_state: RenderResourceHandle,
        layout_data: &RenderPipelineLayoutVk,
        bind_point: ash::vk::PipelineBindPoint,
        arguments: &[RenderShaderArgument],
    ) -> Result<()> {
        //error!("Calling apply_shader_arguments - unimplemented");
        let pipeline_layout = layout_data.pipeline_layout;

        // TODO: Check for redundancy of each of the arguments, as the whole point is the
        // some of them will change at much higher frequencies than others.
        assert!(arguments.len() <= 4);

        let mut descriptor_sets: Vec<ash::vk::DescriptorSet> =
            Vec::with_capacity(arguments.len() + 1);
        for arg_index in 0..arguments.len() {
            let views_index = arguments.len() + arg_index;
            if let Some(shader_views) = arguments[arg_index].shader_views {
                let shader_views = self.storage.get(shader_views)?;
                let mut shader_views = shader_views.write().unwrap();
                let mut shader_views = shader_views.downcast_mut::<RenderShaderViewsVk>().unwrap();
                let mut resource_tracker = self.resource_tracker.borrow_mut();
                let cached_descriptor_set = self.descriptor_cache.memoize(
                    &mut resource_tracker,
                    arg_index as u32,
                    pipeline_state,
                    &layout_data,
                    &mut shader_views,
                );
                if let Some(entry) = cached_descriptor_set {
                    assert_ne!(entry.descriptor_pool, ash::vk::DescriptorPool::null());
                    assert_ne!(entry.descriptor_set, ash::vk::DescriptorSet::null());
                    descriptor_sets.push(entry.descriptor_set);
                }
            }
        }

        if descriptor_sets.len() > 0 {
            unsafe {
                self.device.raw.cmd_bind_descriptor_sets(
                    native,
                    bind_point,
                    pipeline_layout,
                    0,
                    &descriptor_sets,
                    &[],
                );
            }
        }

        Ok(())
    }

    #[inline]
    fn default_state_for_handle(
        &self,
        handle: RenderResourceHandle,
    ) -> Result<RenderResourceStates> {
        let resource_arc = self.storage.get(handle)?;
        let resource_base = resource_arc.read().unwrap();
        match resource_base.get_type() {
            RenderResourceType::Texture => {
                let texture = resource_base.downcast_ref::<RenderTextureVk>().unwrap();
                Ok(texture.default_state)
            }
            RenderResourceType::Buffer => {
                let buffer = resource_base.downcast_ref::<RenderBufferVk>().unwrap();
                Ok(buffer.default_state)
            }
            _ => unimplemented!(),
        }
    }

    #[inline]
    fn name_for_handle(&self, handle: RenderResourceHandle) -> Result<String> {
        let resource_arc = self.storage.get(handle)?;
        let resource_base = resource_arc.read().unwrap();
        Ok(resource_base.get_name().to_string())
    }
}
