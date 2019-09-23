extern crate render_core;
use render_core::allocator::{LinearAllocator, LinearAllocatorMark};
use render_core::utilities::*;

#[test]
fn allocator_is_empty() {
    let allocator = LinearAllocator::new(64);
    assert_eq!(allocator.is_empty(), true);
    assert_eq!(allocator.size(), 0);
    assert_eq!(allocator.capacity(), 64);
    assert_eq!(allocator.mark(), 0 as LinearAllocatorMark);
}

#[test]
fn allocator_mark_insert() {
    let mut allocator = LinearAllocator::new(64);
    let mut test_data: Vec<f32> = Vec::with_capacity(12);
    for i in 0..12 {
        test_data.push(((i as f32) * 1234f32) + ((i as f32) * 1000f32));
    }

    let byte_length = std::mem::size_of::<f32>() * 12;

    // Test slices going out of scope
    let data_mark = {
        let test_bytes = typed_to_bytes(&test_data);
        assert_eq!(test_bytes.len(), byte_length);

        let data_mark = allocator.allocate_raw(byte_length, 8, 0).unwrap();
        allocator.mark_insert(data_mark, &test_bytes).unwrap();

        assert_eq!(byte_length, allocator.size());
        assert_eq!(allocator.mark(), byte_length);
        data_mark
    };

    let test_bytes2 = typed_to_bytes(&test_data);
    assert_eq!(test_bytes2.len(), byte_length);

    let placed_data = allocator.mark_data(data_mark, byte_length).unwrap();
    assert_eq!(test_bytes2, placed_data);
    assert_eq!(allocator.capacity(), 64);
}
