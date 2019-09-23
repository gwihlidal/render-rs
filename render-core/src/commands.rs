use crate::allocator::LinearAllocatorMark;
use crate::constants::*;
use crate::handles::{RenderResourceHandle, RenderResourceHandleAllocator};
use crate::state::*;
use crate::types::*;
use downcast_rs::Downcast;
use std::borrow::Cow;
use std::fmt;

bitflags! {
    pub struct RenderCommandQueueType: u8
    {
        const NONE		= 0x00;
        const COPY		= 0x01;
        const COMPUTE	= 0x02;
        const GRAPHICS	= 0x04;
        const ALL		= 0x01 | 0x02 | 0x04;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RenderCommandType {
    Draw = 0,
    DrawIndirect = 1,
    Dispatch = 2,
    DispatchIndirect = 3,
    UpdateBuffer = 4,
    UpdateTexture = 5,
    CopyBuffer = 6,
    CopyTexture = 7,
    Barriers = 8,
    Transitions = 9,
    BeginTiming = 10,
    EndTiming = 11,
    ResolveTimings = 12,
    BeginEvent = 13,
    EndEvent = 14,
    BeginRenderPass = 15,
    EndRenderPass = 16,
    RayTrace = 17,
    UpdateTopLevelAcceleration = 18,
    UpdateBottomLevelAcceleration = 19,
    UpdateShaderTable = 20,
}

pub trait RenderCommand: Downcast + fmt::Debug {
    fn get_type(&self) -> RenderCommandType;
    fn get_queue(&self) -> RenderCommandQueueType;
}

impl_downcast!(RenderCommand);

pub fn command_as_trait<T: RenderCommand>(t: T) -> T {
    t
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderCommandDraw {
    pub pipeline_state: RenderResourceHandle,
    pub shader_arguments: Vec<RenderShaderArgument>,
    pub draw_state: Option<RenderDrawState>,
    pub draw_binding: Option<RenderResourceHandle>,
    pub draw_packet: RenderDrawPacket,
}

impl RenderCommandDraw {
    pub fn new(
        pipeline_state: RenderResourceHandle,
        shader_arguments: &[RenderShaderArgument],
        draw_state: Option<RenderDrawState>,
        draw_binding: Option<RenderResourceHandle>,
        draw_packet: RenderDrawPacket,
    ) -> RenderCommandDraw {
        RenderCommandDraw {
            pipeline_state,
            shader_arguments: shader_arguments.to_vec(),
            draw_state,
            draw_binding,
            draw_packet,
        }
    }
}

impl RenderCommand for RenderCommandDraw {
    #[inline]
    fn get_type(&self) -> RenderCommandType {
        RenderCommandType::Draw
    }

    #[inline]
    fn get_queue(&self) -> RenderCommandQueueType {
        RenderCommandQueueType::GRAPHICS
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderCommandDrawIndirect {
    pub pipeline_state: RenderResourceHandle,
    pub shader_arguments: Vec<RenderShaderArgument>,
    pub draw_state: Option<RenderDrawState>,
    pub draw_binding: RenderResourceHandle,
    pub primitive: RenderPrimitiveType,
    pub indirect_buffer: RenderResourceHandle,
    pub indirect_byte_offset: usize,
    pub count_buffer: RenderResourceHandle,
    pub count_byte_offset: usize,
    pub command_limit: u32,
}

impl RenderCommandDrawIndirect {
    pub fn new(
        pipeline_state: RenderResourceHandle,
        shader_arguments: &[RenderShaderArgument],
        draw_state: Option<RenderDrawState>,
        draw_binding: RenderResourceHandle,
        primitive: RenderPrimitiveType,
        indirect_buffer: RenderResourceHandle,
        indirect_byte_offset: usize,
        count_buffer: RenderResourceHandle,
        count_byte_offset: usize,
        command_limit: u32,
    ) -> RenderCommandDrawIndirect {
        RenderCommandDrawIndirect {
            pipeline_state,
            shader_arguments: shader_arguments.to_vec(),
            draw_state,
            draw_binding,
            primitive,
            indirect_buffer,
            indirect_byte_offset,
            count_buffer,
            count_byte_offset,
            command_limit,
        }
    }
}

impl RenderCommand for RenderCommandDrawIndirect {
    #[inline]
    fn get_type(&self) -> RenderCommandType {
        RenderCommandType::DrawIndirect
    }

    #[inline]
    fn get_queue(&self) -> RenderCommandQueueType {
        RenderCommandQueueType::GRAPHICS
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderCommandDispatch {
    pub pipeline_state: RenderResourceHandle,
    pub shader_arguments: Vec<RenderShaderArgument>,
    pub dispatch_x: u32,
    pub dispatch_y: u32,
    pub dispatch_z: u32,
}

impl RenderCommandDispatch {
    pub fn new(
        pipeline_state: RenderResourceHandle,
        shader_arguments: &[RenderShaderArgument],
        dispatch_x: u32,
        dispatch_y: u32,
        dispatch_z: u32,
    ) -> RenderCommandDispatch {
        RenderCommandDispatch {
            pipeline_state,
            shader_arguments: shader_arguments.to_vec(),
            dispatch_x,
            dispatch_y,
            dispatch_z,
        }
    }
}

impl RenderCommand for RenderCommandDispatch {
    #[inline]
    fn get_type(&self) -> RenderCommandType {
        RenderCommandType::Dispatch
    }

    #[inline]
    fn get_queue(&self) -> RenderCommandQueueType {
        RenderCommandQueueType::COMPUTE
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderCommandDispatchIndirect {
    pub pipeline_state: RenderResourceHandle,
    pub shader_arguments: Vec<RenderShaderArgument>,
    pub indirect_buffer: RenderResourceHandle,
    pub indirect_byte_offset: usize,
    pub count_buffer: RenderResourceHandle,
    pub count_byte_offset: usize,
    pub command_limit: u32,
}

impl RenderCommandDispatchIndirect {
    pub fn new(
        pipeline_state: RenderResourceHandle,
        shader_arguments: &[RenderShaderArgument],
        indirect_buffer: RenderResourceHandle,
        indirect_byte_offset: usize,
        count_buffer: RenderResourceHandle,
        count_byte_offset: usize,
        command_limit: u32,
    ) -> RenderCommandDispatchIndirect {
        RenderCommandDispatchIndirect {
            pipeline_state,
            shader_arguments: shader_arguments.to_vec(),
            indirect_buffer,
            indirect_byte_offset,
            count_buffer,
            count_byte_offset,
            command_limit,
        }
    }
}

impl RenderCommand for RenderCommandDispatchIndirect {
    #[inline]
    fn get_type(&self) -> RenderCommandType {
        RenderCommandType::DispatchIndirect
    }

    #[inline]
    fn get_queue(&self) -> RenderCommandQueueType {
        RenderCommandQueueType::COMPUTE
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderCommandUpdateBuffer {
    pub buffer: RenderResourceHandle,
    pub offset: usize,
    pub size: usize,
    pub data: LinearAllocatorMark,
}

impl RenderCommandUpdateBuffer {
    pub fn new(
        buffer: RenderResourceHandle,
        offset: usize,
        size: usize,
        data: LinearAllocatorMark,
    ) -> RenderCommandUpdateBuffer {
        RenderCommandUpdateBuffer {
            buffer,
            offset,
            size,
            data,
        }
    }
}

impl RenderCommand for RenderCommandUpdateBuffer {
    #[inline]
    fn get_type(&self) -> RenderCommandType {
        RenderCommandType::UpdateBuffer
    }

    #[inline]
    fn get_queue(&self) -> RenderCommandQueueType {
        RenderCommandQueueType::COPY
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderCommandUpdateTexture {
    pub texture: RenderResourceHandle,
    pub sub_resource: u16,
    pub sub_row_pitch: u32,
    pub sub_slice_pitch: u32,
    pub sub_data_size: usize,
    pub sub_data_mark: LinearAllocatorMark,
}

impl RenderCommandUpdateTexture {
    pub fn new(
        texture: RenderResourceHandle,
        sub_resource: u16,
        sub_row_pitch: u32,
        sub_slice_pitch: u32,
        sub_data_size: usize,
        sub_data_mark: LinearAllocatorMark,
    ) -> RenderCommandUpdateTexture {
        RenderCommandUpdateTexture {
            texture,
            sub_resource,
            sub_row_pitch,
            sub_slice_pitch,
            sub_data_size,
            sub_data_mark,
        }
    }
}

impl RenderCommand for RenderCommandUpdateTexture {
    #[inline]
    fn get_type(&self) -> RenderCommandType {
        RenderCommandType::UpdateTexture
    }

    #[inline]
    fn get_queue(&self) -> RenderCommandQueueType {
        RenderCommandQueueType::COPY
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderCommandCopyBuffer {
    pub src_buffer: RenderResourceHandle,
    pub src_offset: usize,
    pub src_size: usize,
    pub dst_buffer: RenderResourceHandle,
    pub dst_offset: usize,
}

impl RenderCommandCopyBuffer {
    pub fn new(
        src_buffer: RenderResourceHandle,
        src_offset: usize,
        src_size: usize,
        dst_buffer: RenderResourceHandle,
        dst_offset: usize,
    ) -> RenderCommandCopyBuffer {
        RenderCommandCopyBuffer {
            src_buffer,
            src_offset,
            src_size,
            dst_buffer,
            dst_offset,
        }
    }
}

impl RenderCommand for RenderCommandCopyBuffer {
    #[inline]
    fn get_type(&self) -> RenderCommandType {
        RenderCommandType::CopyBuffer
    }

    #[inline]
    fn get_queue(&self) -> RenderCommandQueueType {
        RenderCommandQueueType::COPY
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderCommandCopyTexture {
    pub src_texture: RenderResourceHandle,
    pub src_sub_resource: u16,
    pub src_box: RenderBox,
    pub dst_texture: RenderResourceHandle,
    pub dst_sub_resource: u16,
    pub dst_point: RenderPoint,
}

impl RenderCommandCopyTexture {
    pub fn new(
        src_texture: RenderResourceHandle,
        src_sub_resource: u16,
        src_box: RenderBox,
        dst_texture: RenderResourceHandle,
        dst_sub_resource: u16,
        dst_point: RenderPoint,
    ) -> RenderCommandCopyTexture {
        RenderCommandCopyTexture {
            src_texture,
            src_sub_resource,
            src_box,
            dst_texture,
            dst_sub_resource,
            dst_point,
        }
    }
}

impl RenderCommand for RenderCommandCopyTexture {
    #[inline]
    fn get_type(&self) -> RenderCommandType {
        RenderCommandType::CopyTexture
    }

    #[inline]
    fn get_queue(&self) -> RenderCommandQueueType {
        RenderCommandQueueType::COPY
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderCommandBarriers {
    pub barriers: Vec<RenderResourceHandle>,
}

impl RenderCommandBarriers {
    pub fn new(barriers: &[RenderResourceHandle]) -> RenderCommandBarriers {
        RenderCommandBarriers {
            barriers: barriers.to_vec(),
        }
    }
}

impl RenderCommand for RenderCommandBarriers {
    #[inline]
    fn get_type(&self) -> RenderCommandType {
        RenderCommandType::Barriers
    }

    #[inline]
    fn get_queue(&self) -> RenderCommandQueueType {
        RenderCommandQueueType::NONE
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderCommandTransitions {
    pub transitions: Vec<RenderTransitionRecord>,
}

impl RenderCommandTransitions {
    pub fn new(transitions: &[RenderTransitionRecord]) -> RenderCommandTransitions {
        RenderCommandTransitions {
            transitions: transitions.to_vec(),
        }
    }
}

impl RenderCommand for RenderCommandTransitions {
    #[inline]
    fn get_type(&self) -> RenderCommandType {
        RenderCommandType::Transitions
    }

    #[inline]
    fn get_queue(&self) -> RenderCommandQueueType {
        RenderCommandQueueType::NONE
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderCommandBeginTiming {
    pub timing_heap: RenderResourceHandle,
    pub region: u32,
}

impl RenderCommandBeginTiming {
    pub fn new(timing_heap: RenderResourceHandle, region: u32) -> RenderCommandBeginTiming {
        RenderCommandBeginTiming {
            timing_heap,
            region,
        }
    }
}

impl RenderCommand for RenderCommandBeginTiming {
    #[inline]
    fn get_type(&self) -> RenderCommandType {
        RenderCommandType::BeginTiming
    }

    #[inline]
    fn get_queue(&self) -> RenderCommandQueueType {
        RenderCommandQueueType::NONE
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderCommandEndTiming {
    pub timing_heap: RenderResourceHandle,
    pub region: u32,
}

impl RenderCommandEndTiming {
    pub fn new(timing_heap: RenderResourceHandle, region: u32) -> RenderCommandEndTiming {
        RenderCommandEndTiming {
            timing_heap,
            region,
        }
    }
}

impl RenderCommand for RenderCommandEndTiming {
    #[inline]
    fn get_type(&self) -> RenderCommandType {
        RenderCommandType::EndTiming
    }

    #[inline]
    fn get_queue(&self) -> RenderCommandQueueType {
        RenderCommandQueueType::NONE
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderCommandResolveTimings {
    pub timing_heap: RenderResourceHandle,
    pub region_start: u32,
    pub region_count: u32,
}

impl RenderCommandResolveTimings {
    pub fn new(
        timing_heap: RenderResourceHandle,
        region_start: u32,
        region_count: u32,
    ) -> RenderCommandResolveTimings {
        RenderCommandResolveTimings {
            timing_heap,
            region_start,
            region_count,
        }
    }
}

impl RenderCommand for RenderCommandResolveTimings {
    #[inline]
    fn get_type(&self) -> RenderCommandType {
        RenderCommandType::ResolveTimings
    }

    #[inline]
    fn get_queue(&self) -> RenderCommandQueueType {
        RenderCommandQueueType::NONE
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderCommandBeginEvent {
    pub user_data: u32,
    pub message: Cow<'static, str>,
}

impl RenderCommandBeginEvent {
    pub fn new<S>(user_data: u32, message: S) -> RenderCommandBeginEvent
    where
        S: Into<Cow<'static, str>>,
    {
        RenderCommandBeginEvent {
            user_data,
            message: message.into(),
        }
    }
}

impl RenderCommand for RenderCommandBeginEvent {
    #[inline]
    fn get_type(&self) -> RenderCommandType {
        RenderCommandType::BeginEvent
    }

    #[inline]
    fn get_queue(&self) -> RenderCommandQueueType {
        RenderCommandQueueType::NONE
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderCommandEndEvent {}

impl RenderCommandEndEvent {
    pub fn new() -> RenderCommandEndEvent {
        RenderCommandEndEvent {}
    }
}

impl RenderCommand for RenderCommandEndEvent {
    #[inline]
    fn get_type(&self) -> RenderCommandType {
        RenderCommandType::EndEvent
    }

    #[inline]
    fn get_queue(&self) -> RenderCommandQueueType {
        RenderCommandQueueType::NONE
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderCommandBeginRenderPass {
    pub render_pass: RenderResourceHandle,
}

impl RenderCommandBeginRenderPass {
    pub fn new(render_pass: RenderResourceHandle) -> RenderCommandBeginRenderPass {
        RenderCommandBeginRenderPass { render_pass }
    }
}

impl RenderCommand for RenderCommandBeginRenderPass {
    #[inline]
    fn get_type(&self) -> RenderCommandType {
        RenderCommandType::BeginRenderPass
    }

    #[inline]
    fn get_queue(&self) -> RenderCommandQueueType {
        RenderCommandQueueType::GRAPHICS
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderCommandEndRenderPass {}

impl RenderCommandEndRenderPass {
    pub fn new() -> RenderCommandEndRenderPass {
        RenderCommandEndRenderPass {}
    }
}

impl RenderCommand for RenderCommandEndRenderPass {
    #[inline]
    fn get_type(&self) -> RenderCommandType {
        RenderCommandType::EndRenderPass
    }

    #[inline]
    fn get_queue(&self) -> RenderCommandQueueType {
        RenderCommandQueueType::GRAPHICS
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderCommandRayTrace {
    pub pipeline_state: RenderResourceHandle,
    pub shader_table: RenderResourceHandle,
    pub rt_output: RenderResourceHandle, // TODO: Eliminate this once we have a better way to specify explicit transitions
    pub width: u32,
    pub height: u32,
    pub ray_gen_index: u32,
}

impl RenderCommandRayTrace {
    pub fn new(
        pipeline_state: RenderResourceHandle,
        shader_table: RenderResourceHandle,
        rt_output: RenderResourceHandle, // TODO: Eliminate this once we have a better way to specify explicit transitions
        width: u32,
        height: u32,
        ray_gen_index: u32,
    ) -> RenderCommandRayTrace {
        RenderCommandRayTrace {
            pipeline_state,
            shader_table,
            rt_output,
            width,
            height,
            ray_gen_index,
        }
    }
}

impl RenderCommand for RenderCommandRayTrace {
    #[inline]
    fn get_type(&self) -> RenderCommandType {
        RenderCommandType::RayTrace
    }

    #[inline]
    fn get_queue(&self) -> RenderCommandQueueType {
        RenderCommandQueueType::COMPUTE
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderCommandUpdateTopLevelAcceleration {
    pub acceleration: RenderResourceHandle,
    pub desc: RenderAccelerationTopDesc,
}

impl RenderCommandUpdateTopLevelAcceleration {
    pub fn new(
        acceleration: RenderResourceHandle,
        desc: RenderAccelerationTopDesc,
    ) -> RenderCommandUpdateTopLevelAcceleration {
        RenderCommandUpdateTopLevelAcceleration { acceleration, desc }
    }
}

impl RenderCommand for RenderCommandUpdateTopLevelAcceleration {
    #[inline]
    fn get_type(&self) -> RenderCommandType {
        RenderCommandType::UpdateTopLevelAcceleration
    }

    #[inline]
    fn get_queue(&self) -> RenderCommandQueueType {
        RenderCommandQueueType::COMPUTE
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderCommandUpdateBottomLevelAcceleration {
    pub acceleration: RenderResourceHandle,
    pub refit: bool,
}

impl RenderCommandUpdateBottomLevelAcceleration {
    pub fn new(
        acceleration: RenderResourceHandle,
        refit: bool,
    ) -> RenderCommandUpdateBottomLevelAcceleration {
        RenderCommandUpdateBottomLevelAcceleration {
            acceleration,
            refit,
        }
    }
}

impl RenderCommand for RenderCommandUpdateBottomLevelAcceleration {
    #[inline]
    fn get_type(&self) -> RenderCommandType {
        RenderCommandType::UpdateBottomLevelAcceleration
    }

    #[inline]
    fn get_queue(&self) -> RenderCommandQueueType {
        RenderCommandQueueType::COMPUTE
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderCommandUpdateShaderTable {
    pub shader_table: RenderResourceHandle,
    pub desc: RenderShaderTableUpdateDesc,
}

impl RenderCommandUpdateShaderTable {
    pub fn new(
        shader_table: RenderResourceHandle,
        desc: RenderShaderTableUpdateDesc,
    ) -> RenderCommandUpdateShaderTable {
        RenderCommandUpdateShaderTable { shader_table, desc }
    }
}

impl RenderCommand for RenderCommandUpdateShaderTable {
    #[inline]
    fn get_type(&self) -> RenderCommandType {
        RenderCommandType::UpdateShaderTable
    }

    #[inline]
    fn get_queue(&self) -> RenderCommandQueueType {
        RenderCommandQueueType::COMPUTE
    }
}
