extern crate render_core;
use render_core::handles::{RenderResourceHandle, RenderResourceHandleAllocator};
use render_core::types::RenderResourceType;

#[test]
fn bogus_handle() {
    let alloc = RenderResourceHandleAllocator::new();
    let bogus = RenderResourceHandle::new(0, RenderResourceType::Buffer, 0);
    assert!(!alloc.is_valid(&bogus));
}

#[test]
fn set_id() {
    let mut handle = RenderResourceHandle::new(0, RenderResourceType::Buffer, 0);
    assert_ne!(handle.get_id(), 123);
    handle.set_id(123);
    assert_eq!(handle.get_id(), 123);
}

#[test]
fn set_type() {
    let mut handle = RenderResourceHandle::new(0, RenderResourceType::Buffer, 0);
    assert_ne!(handle.get_type(), RenderResourceType::GraphicsPipelineState);
    handle.set_type(RenderResourceType::GraphicsPipelineState);
    assert_eq!(handle.get_type(), RenderResourceType::GraphicsPipelineState);
}

#[test]
fn set_cookie() {
    let mut handle = RenderResourceHandle::new(0, RenderResourceType::Buffer, 0);
    assert_ne!(handle.get_cookie(), 456);
    handle.set_cookie(456);
    assert_eq!(handle.get_cookie(), 456);
}

#[test]
fn alloc_swap_chain() {
    let mut alloc = RenderResourceHandleAllocator::new();
    let handle = alloc.allocate(RenderResourceType::SwapChain);
    assert!(handle.get_type() == RenderResourceType::SwapChain);
    assert!(alloc.is_valid(&handle));
    alloc.release(handle);
    assert!(!alloc.is_valid(&handle));
}

#[test]
fn alloc_buffer() {
    let mut alloc = RenderResourceHandleAllocator::new();
    let handle = alloc.allocate(RenderResourceType::Buffer);
    assert!(handle.get_type() == RenderResourceType::Buffer);
    assert!(alloc.is_valid(&handle));
    alloc.release(handle);
    assert!(!alloc.is_valid(&handle));
}

#[test]
fn alloc_texture() {
    let mut alloc = RenderResourceHandleAllocator::new();
    let handle = alloc.allocate(RenderResourceType::Texture);
    assert!(handle.get_type() == RenderResourceType::Texture);
    assert!(alloc.is_valid(&handle));
    alloc.release(handle);
    assert!(!alloc.is_valid(&handle));
}

#[test]
fn alloc_sampler_state() {
    let mut alloc = RenderResourceHandleAllocator::new();
    let handle = alloc.allocate(RenderResourceType::SamplerState);
    assert!(handle.get_type() == RenderResourceType::SamplerState);
    assert!(alloc.is_valid(&handle));
    alloc.release(handle);
    assert!(!alloc.is_valid(&handle));
}

#[test]
fn alloc_shader() {
    let mut alloc = RenderResourceHandleAllocator::new();
    let handle = alloc.allocate(RenderResourceType::Shader);
    assert!(handle.get_type() == RenderResourceType::Shader);
    assert!(alloc.is_valid(&handle));
    alloc.release(handle);
    assert!(!alloc.is_valid(&handle));
}

#[test]
fn alloc_shader_views() {
    let mut alloc = RenderResourceHandleAllocator::new();
    let handle = alloc.allocate(RenderResourceType::ShaderViews);
    assert!(handle.get_type() == RenderResourceType::ShaderViews);
    assert!(alloc.is_valid(&handle));
    alloc.release(handle);
    assert!(!alloc.is_valid(&handle));
}

#[test]
fn alloc_graphics_pipeline_state() {
    let mut alloc = RenderResourceHandleAllocator::new();
    let handle = alloc.allocate(RenderResourceType::GraphicsPipelineState);
    assert!(handle.get_type() == RenderResourceType::GraphicsPipelineState);
    assert!(alloc.is_valid(&handle));
    alloc.release(handle);
    assert!(!alloc.is_valid(&handle));
}

#[test]
fn alloc_compute_pipeline_state() {
    let mut alloc = RenderResourceHandleAllocator::new();
    let handle = alloc.allocate(RenderResourceType::ComputePipelineState);
    assert!(handle.get_type() == RenderResourceType::ComputePipelineState);
    assert!(alloc.is_valid(&handle));
    alloc.release(handle);
    assert!(!alloc.is_valid(&handle));
}

#[test]
fn alloc_draw_binding_set() {
    let mut alloc = RenderResourceHandleAllocator::new();
    let handle = alloc.allocate(RenderResourceType::DrawBindingSet);
    assert!(handle.get_type() == RenderResourceType::DrawBindingSet);
    assert!(alloc.is_valid(&handle));
    alloc.release(handle);
    assert!(!alloc.is_valid(&handle));
}

#[test]
fn alloc_frame_binding_set() {
    let mut alloc = RenderResourceHandleAllocator::new();
    let handle = alloc.allocate(RenderResourceType::FrameBindingSet);
    assert!(handle.get_type() == RenderResourceType::FrameBindingSet);
    assert!(alloc.is_valid(&handle));
    alloc.release(handle);
    assert!(!alloc.is_valid(&handle));
}

#[test]
fn alloc_render_pass() {
    let mut alloc = RenderResourceHandleAllocator::new();
    let handle = alloc.allocate(RenderResourceType::RenderPass);
    assert!(handle.get_type() == RenderResourceType::RenderPass);
    assert!(alloc.is_valid(&handle));
    alloc.release(handle);
    assert!(!alloc.is_valid(&handle));
}

#[test]
fn alloc_command_list() {
    let mut alloc = RenderResourceHandleAllocator::new();
    let handle = alloc.allocate(RenderResourceType::CommandList);
    assert!(handle.get_type() == RenderResourceType::CommandList);
    assert!(alloc.is_valid(&handle));
    alloc.release(handle);
    assert!(!alloc.is_valid(&handle));
}

#[test]
fn alloc_fence() {
    let mut alloc = RenderResourceHandleAllocator::new();
    let handle = alloc.allocate(RenderResourceType::Fence);
    assert!(handle.get_type() == RenderResourceType::Fence);
    assert!(alloc.is_valid(&handle));
    alloc.release(handle);
    assert!(!alloc.is_valid(&handle));
}

#[test]
fn alloc_timing_heap() {
    let mut alloc = RenderResourceHandleAllocator::new();
    let handle = alloc.allocate(RenderResourceType::TimingHeap);
    assert!(handle.get_type() == RenderResourceType::TimingHeap);
    assert!(alloc.is_valid(&handle));
    alloc.release(handle);
    assert!(!alloc.is_valid(&handle));
}

#[test]
fn release_randomly() {
    let mut alloc = RenderResourceHandleAllocator::new();

    let handle1 = alloc.allocate(RenderResourceType::Buffer);
    let handle2 = alloc.allocate(RenderResourceType::Texture);
    let handle3 = alloc.allocate(RenderResourceType::SwapChain);
    let handle4 = alloc.allocate(RenderResourceType::GraphicsPipelineState);
    let handle5 = alloc.allocate(RenderResourceType::ComputePipelineState);
    let handle6 = alloc.allocate(RenderResourceType::Buffer);
    let handle7 = alloc.allocate(RenderResourceType::DrawBindingSet);
    let handle8 = alloc.allocate(RenderResourceType::RenderPass);
    let handle9 = alloc.allocate(RenderResourceType::FrameBindingSet);

    assert!(alloc.is_valid(&handle1));
    assert!(alloc.is_valid(&handle2));
    assert!(alloc.is_valid(&handle3));
    assert!(alloc.is_valid(&handle4));
    assert!(alloc.is_valid(&handle5));
    assert!(alloc.is_valid(&handle6));
    assert!(alloc.is_valid(&handle7));
    assert!(alloc.is_valid(&handle8));
    assert!(alloc.is_valid(&handle9));

    alloc.release(handle1);
    assert!(!alloc.is_valid(&handle1));
    assert!(alloc.is_valid(&handle2));
    assert!(alloc.is_valid(&handle3));
    assert!(alloc.is_valid(&handle4));
    assert!(alloc.is_valid(&handle5));
    assert!(alloc.is_valid(&handle6));
    assert!(alloc.is_valid(&handle7));
    assert!(alloc.is_valid(&handle8));
    assert!(alloc.is_valid(&handle9));

    alloc.release(handle3);
    assert!(!alloc.is_valid(&handle1));
    assert!(alloc.is_valid(&handle2));
    assert!(!alloc.is_valid(&handle3));
    assert!(alloc.is_valid(&handle4));
    assert!(alloc.is_valid(&handle5));
    assert!(alloc.is_valid(&handle6));
    assert!(alloc.is_valid(&handle7));
    assert!(alloc.is_valid(&handle8));
    assert!(alloc.is_valid(&handle9));

    alloc.release(handle2);
    assert!(!alloc.is_valid(&handle1));
    assert!(!alloc.is_valid(&handle2));
    assert!(!alloc.is_valid(&handle3));
    assert!(alloc.is_valid(&handle4));
    assert!(alloc.is_valid(&handle5));
    assert!(alloc.is_valid(&handle6));
    assert!(alloc.is_valid(&handle7));
    assert!(alloc.is_valid(&handle8));
    assert!(alloc.is_valid(&handle9));

    alloc.release(handle9);
    assert!(!alloc.is_valid(&handle1));
    assert!(!alloc.is_valid(&handle2));
    assert!(!alloc.is_valid(&handle3));
    assert!(alloc.is_valid(&handle4));
    assert!(alloc.is_valid(&handle5));
    assert!(alloc.is_valid(&handle6));
    assert!(alloc.is_valid(&handle7));
    assert!(alloc.is_valid(&handle8));
    assert!(!alloc.is_valid(&handle9));

    alloc.release(handle7);
    assert!(!alloc.is_valid(&handle1));
    assert!(!alloc.is_valid(&handle2));
    assert!(!alloc.is_valid(&handle3));
    assert!(alloc.is_valid(&handle4));
    assert!(alloc.is_valid(&handle5));
    assert!(alloc.is_valid(&handle6));
    assert!(!alloc.is_valid(&handle7));
    assert!(alloc.is_valid(&handle8));
    assert!(!alloc.is_valid(&handle9));

    alloc.release(handle5);
    assert!(!alloc.is_valid(&handle1));
    assert!(!alloc.is_valid(&handle2));
    assert!(!alloc.is_valid(&handle3));
    assert!(alloc.is_valid(&handle4));
    assert!(!alloc.is_valid(&handle5));
    assert!(alloc.is_valid(&handle6));
    assert!(!alloc.is_valid(&handle7));
    assert!(alloc.is_valid(&handle8));
    assert!(!alloc.is_valid(&handle9));

    alloc.release(handle4);
    assert!(!alloc.is_valid(&handle1));
    assert!(!alloc.is_valid(&handle2));
    assert!(!alloc.is_valid(&handle3));
    assert!(!alloc.is_valid(&handle4));
    assert!(!alloc.is_valid(&handle5));
    assert!(alloc.is_valid(&handle6));
    assert!(!alloc.is_valid(&handle7));
    assert!(alloc.is_valid(&handle8));
    assert!(!alloc.is_valid(&handle9));

    alloc.release(handle6);
    assert!(!alloc.is_valid(&handle1));
    assert!(!alloc.is_valid(&handle2));
    assert!(!alloc.is_valid(&handle3));
    assert!(!alloc.is_valid(&handle4));
    assert!(!alloc.is_valid(&handle5));
    assert!(!alloc.is_valid(&handle6));
    assert!(!alloc.is_valid(&handle7));
    assert!(alloc.is_valid(&handle8));
    assert!(!alloc.is_valid(&handle9));

    alloc.release(handle8);
    assert!(!alloc.is_valid(&handle1));
    assert!(!alloc.is_valid(&handle2));
    assert!(!alloc.is_valid(&handle3));
    assert!(!alloc.is_valid(&handle4));
    assert!(!alloc.is_valid(&handle5));
    assert!(!alloc.is_valid(&handle6));
    assert!(!alloc.is_valid(&handle7));
    assert!(!alloc.is_valid(&handle8));
    assert!(!alloc.is_valid(&handle9));
}

#[test]
fn multi_type() {
    let mut alloc = RenderResourceHandleAllocator::new();

    let handle1 = alloc.allocate(RenderResourceType::Buffer);
    let handle2 = alloc.allocate(RenderResourceType::Buffer);
    let handle3 = alloc.allocate(RenderResourceType::Buffer);
    let handle4 = alloc.allocate(RenderResourceType::Buffer);

    assert!(alloc.is_valid(&handle1));
    assert!(alloc.is_valid(&handle2));
    assert!(alloc.is_valid(&handle3));
    assert!(alloc.is_valid(&handle4));

    alloc.release(handle2);
    alloc.release(handle4);

    assert!(alloc.is_valid(&handle1));
    assert!(!alloc.is_valid(&handle2));
    assert!(alloc.is_valid(&handle3));
    assert!(!alloc.is_valid(&handle4));

    alloc.release(handle1);
    alloc.release(handle3);

    assert!(!alloc.is_valid(&handle1));
    assert!(!alloc.is_valid(&handle2));
    assert!(!alloc.is_valid(&handle3));
    assert!(!alloc.is_valid(&handle4));
}

#[test]
fn count_type() {
    let mut alloc = RenderResourceHandleAllocator::new();
    assert_eq!(alloc.get_count(RenderResourceType::Buffer), 0);
    assert_eq!(alloc.get_max(RenderResourceType::Buffer), 0);

    let handle1 = alloc.allocate(RenderResourceType::Buffer);
    assert_eq!(alloc.get_count(RenderResourceType::Buffer), 1);
    assert_eq!(alloc.get_max(RenderResourceType::Buffer), 1);

    let handle2 = alloc.allocate(RenderResourceType::Buffer);
    assert_eq!(alloc.get_count(RenderResourceType::Buffer), 2);
    assert_eq!(alloc.get_max(RenderResourceType::Buffer), 2);

    let handle3 = alloc.allocate(RenderResourceType::Buffer);
    assert_eq!(alloc.get_count(RenderResourceType::Buffer), 3);
    assert_eq!(alloc.get_max(RenderResourceType::Buffer), 3);

    let handle4 = alloc.allocate(RenderResourceType::Buffer);
    assert_eq!(alloc.get_count(RenderResourceType::Buffer), 4);
    assert_eq!(alloc.get_max(RenderResourceType::Buffer), 4);

    alloc.release(handle1);
    assert_eq!(alloc.get_count(RenderResourceType::Buffer), 3);
    assert_eq!(alloc.get_max(RenderResourceType::Buffer), 4);

    alloc.release(handle2);
    assert_eq!(alloc.get_count(RenderResourceType::Buffer), 2);
    assert_eq!(alloc.get_max(RenderResourceType::Buffer), 4);

    alloc.release(handle3);
    assert_eq!(alloc.get_count(RenderResourceType::Buffer), 1);
    assert_eq!(alloc.get_max(RenderResourceType::Buffer), 4);

    alloc.release(handle4);
    assert_eq!(alloc.get_count(RenderResourceType::Buffer), 0);
    assert_eq!(alloc.get_max(RenderResourceType::Buffer), 4);
}

#[test]
fn allocated_not_valid() {
    let mut alloc = RenderResourceHandleAllocator::new();

    let handle1 = alloc.allocate(RenderResourceType::Buffer);
    assert!(alloc.is_valid(&handle1));
    assert!(alloc.is_allocated(handle1.get_id(), handle1.get_type()));

    alloc.release(handle1);
    let handle2 = alloc.allocate(RenderResourceType::Buffer);

    assert!(!alloc.is_valid(&handle1));
    assert!(alloc.is_allocated(handle1.get_id(), handle1.get_type()));

    assert!(alloc.is_valid(&handle2));
    assert!(alloc.is_allocated(handle2.get_id(), handle2.get_type()));
}

#[test]
fn default_not_valid() {
    let alloc = RenderResourceHandleAllocator::new();
    let handle = RenderResourceHandle::default();
    assert!(!alloc.is_valid(&handle));
    assert!(!alloc.is_allocated(handle.get_id(), handle.get_type()));
}

#[test]
fn handle_compare() {
    let handle1 = RenderResourceHandle::new(123, RenderResourceType::Texture, 12345);
    let handle2 = RenderResourceHandle::new(123, RenderResourceType::Texture, 12345);
    let handle3 = RenderResourceHandle::new(1234, RenderResourceType::Texture, 12345);
    let handle4 = RenderResourceHandle::new(123, RenderResourceType::Buffer, 12345);
    let handle5 = RenderResourceHandle::new(123, RenderResourceType::Texture, 1234);

    assert_eq!(handle1, handle1);
    assert_eq!(handle1, handle2);
    assert_ne!(handle1, handle3);
    assert_ne!(handle1, handle4);
    assert_ne!(handle1, handle5);
}

#[test]
fn handle_clone() {
    let handle1 = RenderResourceHandle::new(123, RenderResourceType::Texture, 12345);
    let handle2 = handle1.clone();
    assert_eq!(handle1, handle2);
}

#[test]
fn handle_packing() {
    let handle1 = RenderResourceHandle::new(123, RenderResourceType::GraphicsPipelineState, 12345);
    let packed = handle1.get_packed();
    assert_ne!(packed, 0);
    assert_ne!(packed, u32::max_value());
    let handle2 = RenderResourceHandle::from_packed(packed, handle1.get_cookie());
    assert_eq!(handle1, handle2);
}
