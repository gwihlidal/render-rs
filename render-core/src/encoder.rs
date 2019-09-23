use crate::allocator::{LinearAllocator, LinearAllocatorMark};
use crate::commands::*;
use crate::error::{Error, Result};
use crate::handles::{RenderResourceHandle, RenderResourceHandleAllocator};
use crate::state::*;
use crate::types::*;
use crate::utilities::*;
use failure::Fail;
use std::borrow::Cow;
use std::sync::{Arc, RwLock};

pub type RenderCommandId = LinearAllocatorMark;

pub struct RenderCommandList<'a> {
    commands: Vec<&'a dyn RenderCommand>,
    allocator: LinearAllocator<'a>,
    handles: Arc<RwLock<RenderResourceHandleAllocator>>,
    queue_type: RenderCommandQueueType,
    draw_state: RenderDrawState,
    draw_state_cache: Option<RenderDrawState>,
    render_pass_active: bool,
}

impl<'a> RenderCommandList<'a> {
    pub fn new(
        handles: Arc<RwLock<RenderResourceHandleAllocator>>,
        size_bytes: usize,
        command_reserve: usize,
    ) -> Result<Self> {
        Ok(RenderCommandList {
            commands: Vec::with_capacity(command_reserve),
            allocator: LinearAllocator::new(size_bytes),
            handles: Arc::clone(&handles),
            queue_type: RenderCommandQueueType::NONE,
            draw_state: Default::default(),
            draw_state_cache: None,
            render_pass_active: false,
        })
    }

    #[inline(always)]
    pub fn get_commands(&self) -> &Vec<&'a dyn RenderCommand> {
        &self.commands
    }

    #[inline(always)]
    pub fn get_queue_type(&self) -> RenderCommandQueueType {
        self.queue_type
    }

    #[inline(always)]
    pub fn get_command_data(&self, mark: LinearAllocatorMark, size: usize) -> Result<&[u8]> {
        Ok(self.allocator.mark_data(mark, size)?)
    }

    pub fn draw(
        &mut self,
        pipeline_state: RenderResourceHandle,
        shader_arguments: &[RenderShaderArgument],
        draw_binding: Option<RenderResourceHandle>,
        draw_state: &RenderDrawState,
        draw_packet: &RenderDrawPacket,
    ) -> Result<RenderCommandId> {
        type CommandType = RenderCommandDraw;
        let mark = self.allocator.allocate_typed::<CommandType>()?;
        let cmd = self.allocator.mark_place::<CommandType>(
            mark,
            CommandType::new(
                pipeline_state,
                shader_arguments,
                Some(*draw_state),
                draw_binding,
                *draw_packet,
            ),
        )?;
        self.commands.push(cmd);
        self.queue_type.insert(cmd.get_queue());
        Ok(mark)
    }

    pub fn draw_indirect(
        &mut self,
        pipeline_state: RenderResourceHandle,
        shader_arguments: &[RenderShaderArgument],
        draw_binding: RenderResourceHandle,
        draw_state: &RenderDrawState,
        primitive: RenderPrimitiveType,
        indirect_buffer: RenderResourceHandle,
        indirect_byte_offset: usize,
        count_buffer: RenderResourceHandle,
        count_byte_offset: usize,
        command_limit: u32,
    ) -> Result<RenderCommandId> {
        type CommandType = RenderCommandDrawIndirect;
        let mark = self.allocator.allocate_typed::<CommandType>()?;
        let cmd = self.allocator.mark_place::<CommandType>(
            mark,
            CommandType::new(
                pipeline_state,
                shader_arguments,
                Some(*draw_state),
                draw_binding,
                primitive,
                indirect_buffer,
                indirect_byte_offset,
                count_buffer,
                count_byte_offset,
                command_limit,
            ),
        )?;
        self.commands.push(cmd);
        self.queue_type.insert(cmd.get_queue());
        Ok(mark)
    }

    pub fn dispatch(
        &mut self,
        pipeline_state: RenderResourceHandle,
        shader_arguments: &[RenderShaderArgument],
        dispatch_x: u32,
        dispatch_y: u32,
        dispatch_z: u32,
    ) -> Result<RenderCommandId> {
        type CommandType = RenderCommandDispatch;
        let mark = self.allocator.allocate_typed::<CommandType>()?;
        let cmd = self.allocator.mark_place::<CommandType>(
            mark,
            CommandType::new(
                pipeline_state,
                shader_arguments,
                dispatch_x,
                dispatch_y,
                dispatch_z,
            ),
        )?;
        self.commands.push(cmd);
        self.queue_type.insert(cmd.get_queue());
        Ok(mark)
    }

    pub fn dispatch_1d(
        &mut self,
        pipeline_state: RenderResourceHandle,
        shader_arguments: &[RenderShaderArgument],
        thread_count_x: u32,
        group_size_x: Option<u32>,
    ) -> Result<RenderCommandId> {
        self.dispatch(
            pipeline_state,
            shader_arguments,
            divide_up_multiple(thread_count_x, group_size_x.unwrap_or(64)),
            1,
            1,
        )
    }

    pub fn dispatch_2d(
        &mut self,
        pipeline_state: RenderResourceHandle,
        shader_arguments: &[RenderShaderArgument],
        thread_count_x: u32,
        thread_count_y: u32,
        group_size_x: Option<u32>,
        group_size_y: Option<u32>,
    ) -> Result<RenderCommandId> {
        self.dispatch(
            pipeline_state,
            shader_arguments,
            divide_up_multiple(thread_count_x, group_size_x.unwrap_or(8)),
            divide_up_multiple(thread_count_y, group_size_y.unwrap_or(8)),
            1,
        )
    }

    pub fn dispatch_3d(
        &mut self,
        pipeline_state: RenderResourceHandle,
        shader_arguments: &[RenderShaderArgument],
        thread_count_x: u32,
        thread_count_y: u32,
        thread_count_z: u32,
        group_size_x: Option<u32>,
        group_size_y: Option<u32>,
        group_size_z: Option<u32>,
    ) -> Result<RenderCommandId> {
        self.dispatch(
            pipeline_state,
            shader_arguments,
            divide_up_multiple(thread_count_x, group_size_x.unwrap_or(4)),
            divide_up_multiple(thread_count_y, group_size_y.unwrap_or(4)),
            divide_up_multiple(thread_count_z, group_size_z.unwrap_or(4)),
        )
    }

    pub fn dispatch_indirect(
        &mut self,
        pipeline_state: RenderResourceHandle,
        shader_arguments: &[RenderShaderArgument],
        indirect_buffer: RenderResourceHandle,
        indirect_byte_offset: usize,
        count_buffer: RenderResourceHandle,
        count_byte_offset: usize,
        command_limit: u32,
    ) -> Result<RenderCommandId> {
        type CommandType = RenderCommandDispatchIndirect;
        let mark = self.allocator.allocate_typed::<CommandType>()?;
        let cmd = self.allocator.mark_place::<CommandType>(
            mark,
            CommandType::new(
                pipeline_state,
                shader_arguments,
                indirect_buffer,
                indirect_byte_offset,
                count_buffer,
                count_byte_offset,
                command_limit,
            ),
        )?;
        self.commands.push(cmd);
        self.queue_type.insert(cmd.get_queue());
        Ok(mark)
    }

    pub fn update_buffer(
        &mut self,
        buffer: RenderResourceHandle,
        offset: usize,
        data: &[u8],
    ) -> Result<RenderCommandId> {
        let aligned_len = align_forward(data.len(), 4); // Most APIs require multiple of 4 sizes
        let data_mark = self
            .allocator
            .allocate_raw(aligned_len, 16 /* for SIMD */, 0)?;
        self.allocator.mark_insert(data_mark, data)?;
        type CommandType = RenderCommandUpdateBuffer;
        let mark = self.allocator.allocate_typed::<CommandType>()?;
        let cmd = self.allocator.mark_place::<CommandType>(
            mark,
            CommandType::new(buffer, offset, aligned_len, data_mark),
        )?;
        self.commands.push(cmd);
        self.queue_type.insert(cmd.get_queue());
        Ok(mark)
    }

    pub fn update_buffer_mark(
        &mut self,
        buffer: RenderResourceHandle,
        offset: usize,
        data_size: usize,
        data_mark: LinearAllocatorMark,
    ) -> Result<RenderCommandId> {
        if is_aligned(data_size, 4) {
            type CommandType = RenderCommandUpdateBuffer;
            let cmd_mark = self.allocator.allocate_typed::<CommandType>()?;
            let cmd = self.allocator.mark_place::<CommandType>(
                cmd_mark,
                CommandType::new(buffer, offset, data_size, data_mark),
            )?;
            self.commands.push(cmd);
            self.queue_type.insert(cmd.get_queue());
            Ok(cmd_mark)
        } else {
            Err(Error::backend(format!(
                "Buffer update sizes must be a multiple of 4 - {} vs {}",
                data_size,
                align_forward(data_size, 4)
            )))
        }
    }

    pub fn update_texture(
        &mut self,
        texture: RenderResourceHandle,
        sub_resource: u16,
        sub_row_pitch: u32,
        sub_slice_pitch: u32,
        sub_data: &[u8],
    ) -> Result<RenderCommandId> {
        let sub_data_size = sub_data.len(); // TODO: Check alignment restrictions similar to update_buffer
        let sub_data_mark = self.allocator.allocate_raw(sub_data_size, 8, 0)?;
        self.allocator.mark_insert(sub_data_mark, sub_data)?;
        type CommandType = RenderCommandUpdateTexture;
        let mark = self.allocator.allocate_typed::<CommandType>()?;
        let cmd = self.allocator.mark_place::<CommandType>(
            mark,
            CommandType::new(
                texture,
                sub_resource,
                sub_row_pitch,
                sub_slice_pitch,
                sub_data_size,
                sub_data_mark,
            ),
        )?;
        self.commands.push(cmd);
        self.queue_type.insert(cmd.get_queue());
        Ok(mark)
    }

    pub fn copy_buffer(
        &mut self,
        src_buffer: RenderResourceHandle,
        src_offset: usize,
        src_size: usize,
        dst_buffer: RenderResourceHandle,
        dst_offset: usize,
    ) -> Result<RenderCommandId> {
        type CommandType = RenderCommandCopyBuffer;
        let mark = self.allocator.allocate_typed::<CommandType>()?;
        let cmd = self.allocator.mark_place::<CommandType>(
            mark,
            CommandType::new(src_buffer, src_offset, src_size, dst_buffer, dst_offset),
        )?;
        self.commands.push(cmd);
        self.queue_type.insert(cmd.get_queue());
        Ok(mark)
    }

    pub fn copy_texture(
        &mut self,
        src_texture: RenderResourceHandle,
        src_sub_resource: u16,
        src_box: RenderBox,
        dst_texture: RenderResourceHandle,
        dst_sub_resource: u16,
        dst_point: RenderPoint,
    ) -> Result<RenderCommandId> {
        type CommandType = RenderCommandCopyTexture;
        let mark = self.allocator.allocate_typed::<CommandType>()?;
        let cmd = self.allocator.mark_place::<CommandType>(
            mark,
            CommandType::new(
                src_texture,
                src_sub_resource,
                src_box,
                dst_texture,
                dst_sub_resource,
                dst_point,
            ),
        )?;
        self.commands.push(cmd);
        self.queue_type.insert(cmd.get_queue());
        Ok(mark)
    }

    pub fn barriers(&mut self, barriers: &[RenderResourceHandle]) -> Result<RenderCommandId> {
        type CommandType = RenderCommandBarriers;
        let mark = self.allocator.allocate_typed::<CommandType>()?;
        let cmd = self
            .allocator
            .mark_place::<CommandType>(mark, CommandType::new(barriers))?;
        self.commands.push(cmd);
        self.queue_type.insert(cmd.get_queue());
        Ok(mark)
    }

    pub fn transitions(
        &mut self,
        transitions: &[RenderTransitionRecord],
    ) -> Result<RenderCommandId> {
        type CommandType = RenderCommandTransitions;
        let mark = self.allocator.allocate_typed::<CommandType>()?;
        let cmd = self
            .allocator
            .mark_place::<CommandType>(mark, CommandType::new(transitions))?;
        self.commands.push(cmd);
        self.queue_type.insert(cmd.get_queue());
        Ok(mark)
    }

    pub fn begin_timing(
        &mut self,
        timing_heap: RenderResourceHandle,
        region: u32,
    ) -> Result<RenderCommandId> {
        type CommandType = RenderCommandBeginTiming;
        let mark = self.allocator.allocate_typed::<CommandType>()?;
        let cmd = self
            .allocator
            .mark_place::<CommandType>(mark, CommandType::new(timing_heap, region))?;
        self.commands.push(cmd);
        self.queue_type.insert(cmd.get_queue());
        Ok(mark)
    }

    pub fn end_timing(
        &mut self,
        timing_heap: RenderResourceHandle,
        region: u32,
    ) -> Result<RenderCommandId> {
        type CommandType = RenderCommandEndTiming;
        let mark = self.allocator.allocate_typed::<CommandType>()?;
        let cmd = self
            .allocator
            .mark_place::<CommandType>(mark, CommandType::new(timing_heap, region))?;
        self.commands.push(cmd);
        self.queue_type.insert(cmd.get_queue());
        Ok(mark)
    }

    pub fn resolve_timings(
        &mut self,
        timing_heap: RenderResourceHandle,
        region_start: u32,
        region_count: u32,
    ) -> Result<RenderCommandId> {
        type CommandType = RenderCommandResolveTimings;
        let mark = self.allocator.allocate_typed::<CommandType>()?;
        let cmd = self.allocator.mark_place::<CommandType>(
            mark,
            CommandType::new(timing_heap, region_start, region_count),
        )?;
        self.commands.push(cmd);
        self.queue_type.insert(cmd.get_queue());
        Ok(mark)
    }

    pub fn begin_event(
        &mut self,
        user_data: u32,
        message: Cow<'static, str>,
    ) -> Result<RenderCommandId> {
        type CommandType = RenderCommandBeginEvent;
        let mark = self.allocator.allocate_typed::<CommandType>()?;
        let cmd = self
            .allocator
            .mark_place::<CommandType>(mark, CommandType::new(user_data, message))?;
        self.commands.push(cmd);
        self.queue_type.insert(cmd.get_queue());
        Ok(mark)
    }

    pub fn end_event(&mut self) -> Result<RenderCommandId> {
        type CommandType = RenderCommandEndEvent;
        let mark = self.allocator.allocate_typed::<CommandType>()?;
        let cmd = self
            .allocator
            .mark_place::<CommandType>(mark, CommandType::new())?;
        self.commands.push(cmd);
        self.queue_type.insert(cmd.get_queue());
        Ok(mark)
    }

    pub fn begin_render_pass(
        &mut self,
        render_pass: RenderResourceHandle,
    ) -> Result<RenderCommandId> {
        type CommandType = RenderCommandBeginRenderPass;
        let mark = self.allocator.allocate_typed::<CommandType>()?;
        let cmd = self
            .allocator
            .mark_place::<CommandType>(mark, CommandType::new(render_pass))?;
        self.commands.push(cmd);
        self.queue_type.insert(cmd.get_queue());
        Ok(mark)
    }

    pub fn end_render_pass(&mut self) -> Result<RenderCommandId> {
        type CommandType = RenderCommandEndRenderPass;
        let mark = self.allocator.allocate_typed::<CommandType>()?;
        let cmd = self
            .allocator
            .mark_place::<CommandType>(mark, CommandType::new())?;
        self.commands.push(cmd);
        self.queue_type.insert(cmd.get_queue());
        Ok(mark)
    }

    pub fn ray_trace(
        &mut self,
        pipeline_state: RenderResourceHandle,
        shader_table: RenderResourceHandle,
        rt_output: RenderResourceHandle, // TODO: Eliminate this once we have a better way to specify explicit transitions
        width: u32,
        height: u32,
        ray_gen_index: u32,
    ) -> Result<RenderCommandId> {
        type CommandType = RenderCommandRayTrace;
        let mark = self.allocator.allocate_typed::<CommandType>()?;
        let cmd = self.allocator.mark_place::<CommandType>(
            mark,
            CommandType::new(
                pipeline_state,
                shader_table,
                rt_output,
                width,
                height,
                ray_gen_index,
            ),
        )?;
        self.commands.push(cmd);
        self.queue_type.insert(cmd.get_queue());
        Ok(mark)
    }

    pub fn update_top_level_acceleration(
        &mut self,
        acceleration: RenderResourceHandle,
        desc: RenderAccelerationTopDesc,
    ) -> Result<RenderCommandId> {
        type CommandType = RenderCommandUpdateTopLevelAcceleration;
        let mark = self.allocator.allocate_typed::<CommandType>()?;
        let cmd = self
            .allocator
            .mark_place::<CommandType>(mark, CommandType::new(acceleration, desc))?;
        self.commands.push(cmd);
        self.queue_type.insert(cmd.get_queue());
        Ok(mark)
    }

    pub fn update_bottom_level_acceleration(
        &mut self,
        acceleration: RenderResourceHandle,
        refit: bool,
    ) -> Result<RenderCommandId> {
        type CommandType = RenderCommandUpdateBottomLevelAcceleration;
        let mark = self.allocator.allocate_typed::<CommandType>()?;
        let cmd = self
            .allocator
            .mark_place::<CommandType>(mark, CommandType::new(acceleration, refit))?;
        self.commands.push(cmd);
        self.queue_type.insert(cmd.get_queue());
        Ok(mark)
    }

    pub fn update_shader_table(
        &mut self,
        shader_table: RenderResourceHandle,
        desc: RenderShaderTableUpdateDesc,
    ) -> Result<RenderCommandId> {
        type CommandType = RenderCommandUpdateShaderTable;
        let mark = self.allocator.allocate_typed::<CommandType>()?;
        let cmd = self
            .allocator
            .mark_place::<CommandType>(mark, CommandType::new(shader_table, desc))?;
        self.commands.push(cmd);
        self.queue_type.insert(cmd.get_queue());
        Ok(mark)
    }
}
