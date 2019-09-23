extern crate render_core;
use render_core::commands::*;
use render_core::encoder::RenderCommandList;
use render_core::handles::{RenderResourceHandle, RenderResourceHandleAllocator};
use render_core::state::*;
use render_core::types::*;
use render_core::utilities::typed_to_bytes;
use std::sync::{Arc, RwLock};

#[test]
fn record_commands() {
    let handles = Arc::new(RwLock::new(RenderResourceHandleAllocator::new()));
    let mut handles_write = handles.write().unwrap();
    let mut command_list = RenderCommandList::new(Arc::clone(&handles), 8 * 1024, 16).unwrap();

    // Dummy render resources
    let graphics_state_handle = handles_write.allocate(RenderResourceType::GraphicsPipelineState);
    let compute_state_handle = handles_write.allocate(RenderResourceType::ComputePipelineState);
    let draw_binding_handle = handles_write.allocate(RenderResourceType::DrawBindingSet);
    let buffer1_handle = handles_write.allocate(RenderResourceType::Buffer);
    let buffer2_handle = handles_write.allocate(RenderResourceType::Buffer);
    let texture1_handle = handles_write.allocate(RenderResourceType::Texture);
    let texture2_handle = handles_write.allocate(RenderResourceType::Texture);
    let render_pass_handle = handles_write.allocate(RenderResourceType::RenderPass);
    let indirect_buffer_handle = handles_write.allocate(RenderResourceType::Buffer);
    let count_buffer_handle = handles_write.allocate(RenderResourceType::Buffer);

    // Draws
    {
        let draw_state = RenderDrawState::default();

        assert!(command_list.begin_render_pass(render_pass_handle).is_ok());

        assert!(command_list
            .draw(
                graphics_state_handle,
                &[],
                Some(draw_binding_handle),
                &draw_state,
                &RenderDrawPacket {
                    index_offset: 1,
                    vertex_offset: 2,
                    vertex_count: 3,
                    first_instance: 4,
                    instance_count: 5,
                }
            )
            .is_ok());

        assert!(command_list
            .draw_indirect(
                graphics_state_handle,
                &[],
                draw_binding_handle,
                &draw_state,
                RenderPrimitiveType::TriangleList,
                indirect_buffer_handle,
                12,
                count_buffer_handle,
                34,
                56
            )
            .is_ok());

        assert!(command_list.end_render_pass().is_ok());
    }

    // Dispatches
    {
        assert!(command_list
            .dispatch(
                compute_state_handle,
                &[],
                20, // dispatch_x
                30, // dispatch_y
                40
            ) // dispatch_z
            .is_ok());

        assert!(command_list
            .dispatch_indirect(
                compute_state_handle,
                &[],
                indirect_buffer_handle,
                12,
                count_buffer_handle,
                34,
                56
            )
            .is_ok());
    }

    // Updates
    {
        let offset = 32 * 1024;
        let data: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

        assert!(command_list
            .update_buffer(buffer1_handle, offset, &data)
            .is_ok());

        let dword_values: [u32; 4] = [1, 2, 3, 4];
        let sub_data = &typed_to_bytes(&dword_values);
        let sub_resource = 4;
        let sub_row_pitch = 4;
        let sub_slice_pitch = 16;

        assert!(command_list
            .update_texture(
                texture1_handle,
                sub_resource,
                sub_row_pitch,
                sub_slice_pitch,
                sub_data
            )
            .is_ok());
    }

    // Copies
    {
        assert!(command_list
            .copy_buffer(buffer1_handle, 12, 34, buffer2_handle, 56)
            .is_ok());

        let dst_point = RenderPoint { x: 1, y: 2, z: 3 };

        let src_box = RenderBox {
            x: 1,
            y: 2,
            z: 3,
            w: 4,
            h: 5,
            d: 6,
        };

        // Copy between two different resources, same sub-resource
        assert!(command_list
            .copy_texture(texture1_handle, 0, src_box, texture2_handle, 0, dst_point)
            .is_ok());

        // Copy within the same resource, different sub-resource
        assert!(command_list
            .copy_texture(texture1_handle, 1, src_box, texture1_handle, 0, dst_point)
            .is_ok());
    }

    // Validate command list
    for command in command_list.get_commands() {
        match command.get_type() {
            RenderCommandType::Draw => {}
            RenderCommandType::DrawIndirect => {}
            RenderCommandType::Dispatch => {}
            RenderCommandType::DispatchIndirect => {}
            RenderCommandType::UpdateBuffer => {}
            RenderCommandType::UpdateTexture => {}
            RenderCommandType::CopyBuffer => {}
            RenderCommandType::CopyTexture => {}
            RenderCommandType::Barriers => {}
            RenderCommandType::Transitions => {}
            RenderCommandType::BeginEvent => {}
            RenderCommandType::EndEvent => {}
            RenderCommandType::BeginTiming => {}
            RenderCommandType::EndTiming => {}
            RenderCommandType::ResolveTimings => {}
            RenderCommandType::BeginRenderPass => {}
            RenderCommandType::EndRenderPass => {}
            _ => {
                panic!("invalid render command in command list");
            }
        }
    }
}

#[test]
fn record_draw() {
    let handles = Arc::new(RwLock::new(RenderResourceHandleAllocator::new()));
    let mut handles_write = handles.write().unwrap();
    let mut command_list = RenderCommandList::new(Arc::clone(&handles), 8 * 1024, 16).unwrap();

    // Dummy render resources
    let pipeline_state_handle = handles_write.allocate(RenderResourceType::GraphicsPipelineState);
    let draw_binding_handle = handles_write.allocate(RenderResourceType::DrawBindingSet);
    let render_pass_handle = handles_write.allocate(RenderResourceType::RenderPass);

    let draw_state = RenderDrawState::default();

    assert!(command_list.begin_render_pass(render_pass_handle).is_ok());
    assert!(command_list
        .draw(
            pipeline_state_handle,
            &[],
            Some(draw_binding_handle),
            &draw_state,
            &RenderDrawPacket {
                index_offset: 1,
                vertex_offset: 2,
                vertex_count: 3,
                first_instance: 4,
                instance_count: 5,
            }
        )
        .is_ok());
    assert!(command_list.end_render_pass().is_ok());

    let commands = command_list.get_commands();
    assert_eq!(commands.len(), 3);

    let command0 = &commands[0];
    let command1 = &commands[1];
    let command2 = &commands[2];

    assert_eq!(command0.get_type(), RenderCommandType::BeginRenderPass);
    assert_eq!(command1.get_type(), RenderCommandType::Draw);
    assert_eq!(command2.get_type(), RenderCommandType::EndRenderPass);

    let command0_typed = command0
        .downcast_ref::<RenderCommandBeginRenderPass>()
        .unwrap();
    let command1_typed = command1.downcast_ref::<RenderCommandDraw>().unwrap();
    let command2_typed = command2
        .downcast_ref::<RenderCommandEndRenderPass>()
        .unwrap();

    assert_eq!(command0_typed.render_pass, render_pass_handle);
    assert_eq!(command1_typed.pipeline_state, pipeline_state_handle);
    assert_eq!(command1_typed.draw_binding, Some(draw_binding_handle));
    assert_eq!(command1_typed.draw_state, Some(draw_state));
    assert_eq!(command1_typed.draw_packet.index_offset, 1);
    assert_eq!(command1_typed.draw_packet.vertex_offset, 2);
    assert_eq!(command1_typed.draw_packet.vertex_count, 3);
    assert_eq!(command1_typed.draw_packet.first_instance, 4);
    assert_eq!(command1_typed.draw_packet.instance_count, 5);

    // Does the command list now contain the queue type needed to run this command?
    assert!(command_list.get_queue_type().contains(
        command0_typed.get_queue() | command1_typed.get_queue() | command2_typed.get_queue()
    ));
}

#[test]
fn record_draw_indirect() {
    let handles = Arc::new(RwLock::new(RenderResourceHandleAllocator::new()));
    let mut handles_write = handles.write().unwrap();
    let mut command_list = RenderCommandList::new(Arc::clone(&handles), 8 * 1024, 16).unwrap();

    // Dummy render resources
    let pipeline_state_handle = handles_write.allocate(RenderResourceType::GraphicsPipelineState);
    let draw_binding_handle = handles_write.allocate(RenderResourceType::DrawBindingSet);
    let indirect_buffer_handle = handles_write.allocate(RenderResourceType::Buffer);
    let count_buffer_handle = handles_write.allocate(RenderResourceType::Buffer);
    let render_pass_handle = handles_write.allocate(RenderResourceType::RenderPass);

    let draw_state = RenderDrawState::default();

    assert!(command_list.begin_render_pass(render_pass_handle).is_ok());
    assert!(command_list
        .draw_indirect(
            pipeline_state_handle,
            &[],
            draw_binding_handle,
            &draw_state,
            RenderPrimitiveType::TriangleList,
            indirect_buffer_handle,
            12,
            count_buffer_handle,
            34,
            56
        )
        .is_ok());
    assert!(command_list.end_render_pass().is_ok());

    let commands = command_list.get_commands();
    assert_eq!(commands.len(), 3);

    let command0 = &commands[0];
    let command1 = &commands[1];
    let command2 = &commands[2];

    assert_eq!(command0.get_type(), RenderCommandType::BeginRenderPass);
    assert_eq!(command1.get_type(), RenderCommandType::DrawIndirect);
    assert_eq!(command2.get_type(), RenderCommandType::EndRenderPass);

    let command0_typed = command0
        .downcast_ref::<RenderCommandBeginRenderPass>()
        .unwrap();
    let command1_typed = command1
        .downcast_ref::<RenderCommandDrawIndirect>()
        .unwrap();
    let command2_typed = command2
        .downcast_ref::<RenderCommandEndRenderPass>()
        .unwrap();

    assert_eq!(command0_typed.render_pass, render_pass_handle);
    assert_eq!(command1_typed.pipeline_state, pipeline_state_handle);
    assert_eq!(command1_typed.draw_binding, draw_binding_handle);
    assert_eq!(command1_typed.draw_state, Some(draw_state));
    assert_eq!(command1_typed.indirect_buffer, indirect_buffer_handle);
    assert_eq!(command1_typed.indirect_byte_offset, 12);
    assert_eq!(command1_typed.count_buffer, count_buffer_handle);
    assert_eq!(command1_typed.count_byte_offset, 34);
    assert_eq!(command1_typed.command_limit, 56);

    // Does the command list now contain the queue type needed to run this command?
    assert!(command_list.get_queue_type().contains(
        command0_typed.get_queue() | command1_typed.get_queue() | command2_typed.get_queue()
    ));
}

#[test]
fn record_dispatch() {
    let handles = Arc::new(RwLock::new(RenderResourceHandleAllocator::new()));
    let mut handles_write = handles.write().unwrap();
    let mut command_list = RenderCommandList::new(Arc::clone(&handles), 8 * 1024, 16).unwrap();

    let compute_state_handle = handles_write.allocate(RenderResourceType::ComputePipelineState);

    assert!(command_list
        .dispatch(
            compute_state_handle,
            &[],
            20, // dispatch_x
            30, // dispatch_y
            40
        ) // dispatch_z
        .is_ok());

    let commands = command_list.get_commands();
    assert_eq!(commands.len(), 1);

    let command = &commands[0];
    assert_eq!(command.get_type(), RenderCommandType::Dispatch);

    let command_typed = command.downcast_ref::<RenderCommandDispatch>().unwrap();
    assert_eq!(command_typed.pipeline_state, compute_state_handle);
    assert_eq!(command_typed.dispatch_x, 20);
    assert_eq!(command_typed.dispatch_y, 30);
    assert_eq!(command_typed.dispatch_z, 40);

    // Does the command list now contain the queue type needed to run this command?
    assert!(command_list
        .get_queue_type()
        .contains(command_typed.get_queue()));
}

#[test]
fn record_dispatch_indirect() {
    let handles = Arc::new(RwLock::new(RenderResourceHandleAllocator::new()));
    let mut handles_write = handles.write().unwrap();
    let mut command_list = RenderCommandList::new(Arc::clone(&handles), 8 * 1024, 16).unwrap();

    let compute_state_handle = handles_write.allocate(RenderResourceType::ComputePipelineState);
    let indirect_buffer_handle = handles_write.allocate(RenderResourceType::Buffer);
    let count_buffer_handle = handles_write.allocate(RenderResourceType::Buffer);

    assert!(command_list
        .dispatch_indirect(
            compute_state_handle,
            &[],
            indirect_buffer_handle,
            12,
            count_buffer_handle,
            34,
            56
        )
        .is_ok());

    let commands = command_list.get_commands();
    assert_eq!(commands.len(), 1);

    let command = &commands[0];
    assert_eq!(command.get_type(), RenderCommandType::DispatchIndirect);

    let command_typed = command
        .downcast_ref::<RenderCommandDispatchIndirect>()
        .unwrap();
    assert_eq!(command_typed.pipeline_state, compute_state_handle);
    assert_eq!(command_typed.indirect_buffer, indirect_buffer_handle);
    assert_eq!(command_typed.indirect_byte_offset, 12);
    assert_eq!(command_typed.count_buffer, count_buffer_handle);
    assert_eq!(command_typed.count_byte_offset, 34);
    assert_eq!(command_typed.command_limit, 56);

    // Does the command list now contain the queue type needed to run this command?
    assert!(command_list
        .get_queue_type()
        .contains(command_typed.get_queue()));
}

#[test]
fn record_update_buffer() {
    let handles = Arc::new(RwLock::new(RenderResourceHandleAllocator::new()));
    let mut handles_write = handles.write().unwrap();
    let mut command_list = RenderCommandList::new(Arc::clone(&handles), 8 * 1024, 16).unwrap();

    // Dummy render resources
    let buffer_handle = handles_write.allocate(RenderResourceType::Buffer);

    let offset = 32 * 1024;
    let data: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

    assert!(command_list
        .update_buffer(buffer_handle, offset, &data)
        .is_ok());

    let commands = command_list.get_commands();
    assert_eq!(commands.len(), 1);

    let command = &commands[0];
    assert_eq!(command.get_type(), RenderCommandType::UpdateBuffer);

    let command_typed = command.downcast_ref::<RenderCommandUpdateBuffer>().unwrap();

    assert_eq!(command_typed.buffer, buffer_handle);
    assert_eq!(command_typed.offset, offset);
    assert_eq!(command_typed.size, data.len());
    let command_data = command_list
        .get_command_data(command_typed.data, command_typed.size)
        .unwrap();
    assert_eq!(command_data.len(), data.len());
    for index in 0..16 {
        assert_eq!(data[index], command_data[index]);
    }

    // Does the command list now contain the queue type needed to run this command?
    assert!(command_list
        .get_queue_type()
        .contains(command_typed.get_queue()));
}

#[test]
fn record_update_texture() {
    let handles = Arc::new(RwLock::new(RenderResourceHandleAllocator::new()));
    let mut handles_write = handles.write().unwrap();
    let mut command_list = RenderCommandList::new(Arc::clone(&handles), 8 * 1024, 16).unwrap();

    // Dummy render resources
    let texture_handle = handles_write.allocate(RenderResourceType::Texture);

    let dword_values: [u32; 4] = [1, 2, 3, 4];
    let sub_data = &typed_to_bytes(&dword_values);
    let sub_resource = 4;
    let sub_row_pitch = 4;
    let sub_slice_pitch = 16;

    assert!(command_list
        .update_texture(
            texture_handle,
            sub_resource,
            sub_row_pitch,
            sub_slice_pitch,
            sub_data
        )
        .is_ok());

    let commands = command_list.get_commands();
    assert_eq!(commands.len(), 1);

    let command = &commands[0];
    assert_eq!(command.get_type(), RenderCommandType::UpdateTexture);

    let command_typed = command
        .downcast_ref::<RenderCommandUpdateTexture>()
        .unwrap();

    assert_eq!(command_typed.texture, texture_handle);
    assert_eq!(command_typed.sub_resource, sub_resource);
    assert_eq!(command_typed.sub_row_pitch, sub_row_pitch);
    assert_eq!(command_typed.sub_slice_pitch, sub_slice_pitch);
    assert_eq!(command_typed.sub_data_size, sub_data.len());
    let command_data = command_list
        .get_command_data(command_typed.sub_data_mark, command_typed.sub_data_size)
        .unwrap();
    assert_eq!(command_data.len(), sub_data.len());
    for index in 0..command_data.len() {
        assert_eq!(command_data[index], sub_data[index]);
    }

    // Does the command list now contain the queue type needed to run this command?
    assert!(command_list
        .get_queue_type()
        .contains(command_typed.get_queue()));
}

#[test]
fn record_copy_buffer() {
    let handles = Arc::new(RwLock::new(RenderResourceHandleAllocator::new()));
    let mut handles_write = handles.write().unwrap();
    let mut command_list = RenderCommandList::new(Arc::clone(&handles), 8 * 1024, 16).unwrap();

    // Dummy render resources
    let src_buffer_handle = handles_write.allocate(RenderResourceType::Buffer);
    let dst_buffer_handle = handles_write.allocate(RenderResourceType::Buffer);

    assert!(command_list
        .copy_buffer(src_buffer_handle, 12, 34, dst_buffer_handle, 56)
        .is_ok());

    let commands = command_list.get_commands();
    assert_eq!(commands.len(), 1);

    let command = &commands[0];
    assert_eq!(command.get_type(), RenderCommandType::CopyBuffer);

    let command_typed = command.downcast_ref::<RenderCommandCopyBuffer>().unwrap();
    assert_eq!(command_typed.src_buffer, src_buffer_handle);
    assert_eq!(command_typed.src_offset, 12);
    assert_eq!(command_typed.src_size, 34);
    assert_eq!(command_typed.dst_buffer, dst_buffer_handle);
    assert_eq!(command_typed.dst_offset, 56);

    // Does the command list now contain the queue type needed to run this command?
    assert!(command_list
        .get_queue_type()
        .contains(command_typed.get_queue()));
}

#[test]
fn record_copy_texture() {
    let handles = Arc::new(RwLock::new(RenderResourceHandleAllocator::new()));
    let mut handles_write = handles.write().unwrap();
    let mut command_list = RenderCommandList::new(Arc::clone(&handles), 8 * 1024, 16).unwrap();

    // Dummy render resources
    let src_texture_handle = handles_write.allocate(RenderResourceType::Texture);
    let dst_texture_handle = handles_write.allocate(RenderResourceType::Texture);

    let dst_point = RenderPoint { x: 1, y: 2, z: 3 };

    let src_box = RenderBox {
        x: 1,
        y: 2,
        z: 3,
        w: 4,
        h: 5,
        d: 6,
    };

    assert!(command_list
        .copy_texture(
            src_texture_handle,
            12,
            src_box,
            dst_texture_handle,
            34,
            dst_point
        )
        .is_ok());

    let commands = command_list.get_commands();
    assert_eq!(commands.len(), 1);

    let command = &commands[0];
    assert_eq!(command.get_type(), RenderCommandType::CopyTexture);

    let command_typed = command.downcast_ref::<RenderCommandCopyTexture>().unwrap();
    assert_eq!(command_typed.src_texture, src_texture_handle);
    assert_eq!(command_typed.src_sub_resource, 12);
    assert_eq!(command_typed.src_box, src_box);
    assert_eq!(command_typed.dst_texture, dst_texture_handle);
    assert_eq!(command_typed.dst_sub_resource, 34);
    assert_eq!(command_typed.dst_point, dst_point);

    // Does the command list now contain the queue type needed to run this command?
    assert!(command_list
        .get_queue_type()
        .contains(command_typed.get_queue()));
}

#[test]
fn record_barriers() {
    let handles = Arc::new(RwLock::new(RenderResourceHandleAllocator::new()));
    let mut handles_write = handles.write().unwrap();
    let mut command_list = RenderCommandList::new(Arc::clone(&handles), 8 * 1024, 16).unwrap();

    // Dummy render resources
    let dst_buffer_handle_1 = handles_write.allocate(RenderResourceType::Buffer);
    let dst_buffer_handle_2 = handles_write.allocate(RenderResourceType::Buffer);

    let barriers: Vec<RenderResourceHandle> = vec![dst_buffer_handle_1, dst_buffer_handle_2];

    assert!(command_list.barriers(&barriers).is_ok());

    let commands = command_list.get_commands();
    assert_eq!(commands.len(), 1);

    let command = &commands[0];
    assert_eq!(command.get_type(), RenderCommandType::Barriers);

    let command_typed = command.downcast_ref::<RenderCommandBarriers>().unwrap();
    assert_eq!(command_typed.barriers.len(), 2);
    assert_eq!(command_typed.barriers[0], barriers[0]);
    assert_eq!(command_typed.barriers[1], barriers[1]);

    // Does the command list now contain the queue type needed to run this command?
    assert!(command_list
        .get_queue_type()
        .contains(command_typed.get_queue()));
}

#[test]
fn record_transitions() {
    let handles = Arc::new(RwLock::new(RenderResourceHandleAllocator::new()));
    let mut handles_write = handles.write().unwrap();
    let mut command_list = RenderCommandList::new(Arc::clone(&handles), 8 * 1024, 16).unwrap();

    // Dummy render resources
    let dst_buffer_handle_1 = handles_write.allocate(RenderResourceType::Buffer);
    let dst_buffer_handle_2 = handles_write.allocate(RenderResourceType::Buffer);

    let transitions: Vec<RenderTransitionRecord> = vec![
        (dst_buffer_handle_1, RenderResourceStates::GENERIC_READ),
        (dst_buffer_handle_2, RenderResourceStates::UNORDERED_ACCESS),
    ];

    assert!(command_list.transitions(&transitions).is_ok());

    let commands = command_list.get_commands();
    assert_eq!(commands.len(), 1);

    let command = &commands[0];
    assert_eq!(command.get_type(), RenderCommandType::Transitions);

    let command_typed = command.downcast_ref::<RenderCommandTransitions>().unwrap();
    assert_eq!(command_typed.transitions.len(), 2);
    assert_eq!(command_typed.transitions[0], transitions[0]);
    assert_eq!(command_typed.transitions[1], transitions[1]);

    // Does the command list now contain the queue type needed to run this command?
    assert!(command_list
        .get_queue_type()
        .contains(command_typed.get_queue()));
}

#[test]
fn record_begin_timing() {
    let handles = Arc::new(RwLock::new(RenderResourceHandleAllocator::new()));
    let mut handles_write = handles.write().unwrap();
    let mut command_list = RenderCommandList::new(Arc::clone(&handles), 8 * 1024, 16).unwrap();

    // Dummy render resources
    let timing_heap_handle = handles_write.allocate(RenderResourceType::TimingHeap);

    assert!(command_list.begin_timing(timing_heap_handle, 123).is_ok());

    let commands = command_list.get_commands();
    assert_eq!(commands.len(), 1);

    let command = &commands[0];
    assert_eq!(command.get_type(), RenderCommandType::BeginTiming);

    let command_typed = command.downcast_ref::<RenderCommandBeginTiming>().unwrap();
    assert_eq!(command_typed.timing_heap, timing_heap_handle);
    assert_eq!(command_typed.region, 123);

    // Does the command list now contain the queue type needed to run this command?
    assert!(command_list
        .get_queue_type()
        .contains(command_typed.get_queue()));
}

#[test]
fn record_end_timing() {
    let handles = Arc::new(RwLock::new(RenderResourceHandleAllocator::new()));
    let mut handles_write = handles.write().unwrap();
    let mut command_list = RenderCommandList::new(Arc::clone(&handles), 8 * 1024, 16).unwrap();

    // Dummy render resources
    let timing_heap_handle = handles_write.allocate(RenderResourceType::TimingHeap);

    assert!(command_list.end_timing(timing_heap_handle, 123).is_ok());

    let commands = command_list.get_commands();
    assert_eq!(commands.len(), 1);

    let command = &commands[0];
    assert_eq!(command.get_type(), RenderCommandType::EndTiming);

    let command_typed = command.downcast_ref::<RenderCommandEndTiming>().unwrap();
    assert_eq!(command_typed.timing_heap, timing_heap_handle);
    assert_eq!(command_typed.region, 123);

    // Does the command list now contain the queue type needed to run this command?
    assert!(command_list
        .get_queue_type()
        .contains(command_typed.get_queue()));
}

#[test]
fn record_resolve_timings() {
    let handles = Arc::new(RwLock::new(RenderResourceHandleAllocator::new()));
    let mut handles_write = handles.write().unwrap();
    let mut command_list = RenderCommandList::new(Arc::clone(&handles), 8 * 1024, 16).unwrap();

    // Dummy render resources
    let timing_heap_handle = handles_write.allocate(RenderResourceType::TimingHeap);

    assert!(command_list
        .resolve_timings(timing_heap_handle, 123, 456)
        .is_ok());

    let commands = command_list.get_commands();
    assert_eq!(commands.len(), 1);

    let command = &commands[0];
    assert_eq!(command.get_type(), RenderCommandType::ResolveTimings);

    let command_typed = command
        .downcast_ref::<RenderCommandResolveTimings>()
        .unwrap();

    assert_eq!(command_typed.timing_heap, timing_heap_handle);
    assert_eq!(command_typed.region_start, 123);
    assert_eq!(command_typed.region_count, 456);

    // Does the command list now contain the queue type needed to run this command?
    assert!(command_list
        .get_queue_type()
        .contains(command_typed.get_queue()));
}

#[test]
fn record_begin_event() {
    let handles = Arc::new(RwLock::new(RenderResourceHandleAllocator::new()));
    let mut command_list = RenderCommandList::new(Arc::clone(&handles), 8 * 1024, 16).unwrap();

    assert!(command_list
        .begin_event(123, "This is the message".into())
        .is_ok());

    let commands = command_list.get_commands();
    assert_eq!(commands.len(), 1);

    let command = &commands[0];
    assert_eq!(command.get_type(), RenderCommandType::BeginEvent);

    let command_typed = command.downcast_ref::<RenderCommandBeginEvent>().unwrap();
    assert_eq!(command_typed.user_data, 123);
    assert_eq!(command_typed.message, "This is the message");

    // Does the command list now contain the queue type needed to run this command?
    assert!(command_list
        .get_queue_type()
        .contains(command_typed.get_queue()));
}

#[test]
fn record_end_event() {
    let handles = Arc::new(RwLock::new(RenderResourceHandleAllocator::new()));
    let mut command_list = RenderCommandList::new(Arc::clone(&handles), 8 * 1024, 16).unwrap();

    assert!(command_list.end_event().is_ok());

    let commands = command_list.get_commands();
    assert_eq!(commands.len(), 1);

    let command = &commands[0];
    assert_eq!(command.get_type(), RenderCommandType::EndEvent);

    let command_typed = command.downcast_ref::<RenderCommandEndEvent>().unwrap();

    // Does the command list now contain the queue type needed to run this command?
    assert!(command_list
        .get_queue_type()
        .contains(command_typed.get_queue()));
}

#[test]
fn record_begin_render_pass() {
    let handles = Arc::new(RwLock::new(RenderResourceHandleAllocator::new()));
    let mut handles_write = handles.write().unwrap();
    let mut command_list = RenderCommandList::new(Arc::clone(&handles), 8 * 1024, 16).unwrap();

    // Dummy render resources
    let render_pass_handle = handles_write.allocate(RenderResourceType::RenderPass);

    assert!(command_list.begin_render_pass(render_pass_handle).is_ok());

    let commands = command_list.get_commands();
    assert_eq!(commands.len(), 1);

    let command = &commands[0];
    assert_eq!(command.get_type(), RenderCommandType::BeginRenderPass);

    let command_typed = command
        .downcast_ref::<RenderCommandBeginRenderPass>()
        .unwrap();

    assert_eq!(command_typed.render_pass, render_pass_handle);

    // Does the command list now contain the queue type needed to run this command?
    assert!(command_list
        .get_queue_type()
        .contains(command_typed.get_queue()));
}

#[test]
fn record_end_render_pass() {
    let handles = Arc::new(RwLock::new(RenderResourceHandleAllocator::new()));
    let mut command_list = RenderCommandList::new(Arc::clone(&handles), 8 * 1024, 16).unwrap();

    assert!(command_list.end_render_pass().is_ok());

    let commands = command_list.get_commands();
    assert_eq!(commands.len(), 1);

    let command = &commands[0];
    assert_eq!(command.get_type(), RenderCommandType::EndRenderPass);

    let command_typed = command
        .downcast_ref::<RenderCommandEndRenderPass>()
        .unwrap();

    // Does the command list now contain the queue type needed to run this command?
    assert!(command_list
        .get_queue_type()
        .contains(command_typed.get_queue()));
}

#[test]
fn record_ray_trace() {
    let handles = Arc::new(RwLock::new(RenderResourceHandleAllocator::new()));
    let mut handles_write = handles.write().unwrap();
    let mut command_list = RenderCommandList::new(Arc::clone(&handles), 8 * 1024, 16).unwrap();

    // Dummy render resources
    let pipeline_state_handle = handles_write.allocate(RenderResourceType::RayTracingPipelineState);
    let shader_table_handle = handles_write.allocate(RenderResourceType::RayTracingShaderTable);
    let rt_output_handle = handles_write.allocate(RenderResourceType::Texture);

    assert!(command_list
        .ray_trace(
            pipeline_state_handle,
            shader_table_handle,
            rt_output_handle,
            123,
            456,
            789
        )
        .is_ok());

    let commands = command_list.get_commands();
    assert_eq!(commands.len(), 1);

    let command = &commands[0];
    assert_eq!(command.get_type(), RenderCommandType::RayTrace);

    let command_typed = command.downcast_ref::<RenderCommandRayTrace>().unwrap();
    assert_eq!(command_typed.pipeline_state, pipeline_state_handle);
    assert_eq!(command_typed.shader_table, shader_table_handle);
    assert_eq!(command_typed.rt_output, rt_output_handle);
    assert_eq!(command_typed.width, 123);
    assert_eq!(command_typed.height, 456);
    assert_eq!(command_typed.ray_gen_index, 789);

    // Does the command list now contain the queue type needed to run this command?
    assert!(command_list
        .get_queue_type()
        .contains(command_typed.get_queue()));
}

#[test]
fn record_update_top_level_acceleration() {
    let handles = Arc::new(RwLock::new(RenderResourceHandleAllocator::new()));
    let mut handles_write = handles.write().unwrap();
    let mut command_list = RenderCommandList::new(Arc::clone(&handles), 8 * 1024, 16).unwrap();

    // Dummy render resources
    let acceleration_handle = handles_write.allocate(RenderResourceType::RayTracingAcceleration);

    let desc = RenderAccelerationTopDesc::default();

    assert!(command_list
        .update_top_level_acceleration(acceleration_handle, desc,)
        .is_ok());

    let commands = command_list.get_commands();
    assert_eq!(commands.len(), 1);

    let command = &commands[0];
    assert_eq!(
        command.get_type(),
        RenderCommandType::UpdateTopLevelAcceleration
    );

    let command_typed = command
        .downcast_ref::<RenderCommandUpdateTopLevelAcceleration>()
        .unwrap();

    assert_eq!(command_typed.acceleration, acceleration_handle);

    // Does the command list now contain the queue type needed to run this command?
    assert!(command_list
        .get_queue_type()
        .contains(command_typed.get_queue()));
}

#[test]
fn record_update_bottom_level_acceleration() {
    let handles = Arc::new(RwLock::new(RenderResourceHandleAllocator::new()));
    let mut handles_write = handles.write().unwrap();
    let mut command_list = RenderCommandList::new(Arc::clone(&handles), 8 * 1024, 16).unwrap();

    // Dummy render resources
    let acceleration_handle = handles_write.allocate(RenderResourceType::RayTracingAcceleration);

    assert!(command_list
        .update_bottom_level_acceleration(acceleration_handle, true)
        .is_ok());

    let commands = command_list.get_commands();
    assert_eq!(commands.len(), 1);

    let command = &commands[0];
    assert_eq!(
        command.get_type(),
        RenderCommandType::UpdateBottomLevelAcceleration
    );

    let command_typed = command
        .downcast_ref::<RenderCommandUpdateBottomLevelAcceleration>()
        .unwrap();

    assert_eq!(command_typed.acceleration, acceleration_handle);
    assert_eq!(command_typed.refit, true);

    // Does the command list now contain the queue type needed to run this command?
    assert!(command_list
        .get_queue_type()
        .contains(command_typed.get_queue()));
}

#[test]
fn record_update_shader_table() {
    let handles = Arc::new(RwLock::new(RenderResourceHandleAllocator::new()));
    let mut handles_write = handles.write().unwrap();
    let mut command_list = RenderCommandList::new(Arc::clone(&handles), 8 * 1024, 16).unwrap();

    // Dummy render resources
    let shader_table_handle = handles_write.allocate(RenderResourceType::RayTracingShaderTable);

    let desc = RenderShaderTableUpdateDesc::default();

    assert!(command_list
        .update_shader_table(shader_table_handle, desc,)
        .is_ok());

    let commands = command_list.get_commands();
    assert_eq!(commands.len(), 1);

    let command = &commands[0];
    assert_eq!(command.get_type(), RenderCommandType::UpdateShaderTable);

    let command_typed = command
        .downcast_ref::<RenderCommandUpdateShaderTable>()
        .unwrap();

    assert_eq!(command_typed.shader_table, shader_table_handle);

    // Does the command list now contain the queue type needed to run this command?
    assert!(command_list
        .get_queue_type()
        .contains(command_typed.get_queue()));
}
