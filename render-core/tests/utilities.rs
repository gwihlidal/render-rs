extern crate render_core;
use render_core::utilities;

#[test]
fn align_forward() {
    let a = 12usize;
    assert_eq!(utilities::align_forward(a, 16), 16);
    assert_eq!(utilities::align_forward(a, 32), 32);
    assert_eq!(utilities::align_forward(a, 64), 64);
    assert_eq!(utilities::align_forward(a, 128), 128);
    assert_eq!(utilities::align_forward(a, 256), 256);
    assert_eq!(utilities::align_forward(a, 512), 512);
    assert_eq!(utilities::align_forward(a, 1024), 1024);

    let b = 3usize;
    assert_eq!(utilities::align_forward(b, 16), 16);
    assert_eq!(utilities::align_forward(b, 32), 32);
    assert_eq!(utilities::align_forward(b, 64), 64);
    assert_eq!(utilities::align_forward(b, 128), 128);
    assert_eq!(utilities::align_forward(b, 256), 256);
    assert_eq!(utilities::align_forward(b, 512), 512);
    assert_eq!(utilities::align_forward(b, 1024), 1024);

    let _c = 0usize;
    // TODO: Make this valid?
    //assert_eq!(utilities::align_forward(c, 16), 16);
    //assert_eq!(utilities::align_forward(c, 32), 32);
    //assert_eq!(utilities::align_forward(c, 64), 64);
    //assert_eq!(utilities::align_forward(c, 128), 128);
    //assert_eq!(utilities::align_forward(c, 256), 256);
    //assert_eq!(utilities::align_forward(c, 512), 512);
    //assert_eq!(utilities::align_forward(c, 1024), 1024);

    let d = 7usize;
    assert_eq!(utilities::align_forward(d, 16), 16);
    assert_eq!(utilities::align_forward(d, 32), 32);
    assert_eq!(utilities::align_forward(d, 64), 64);
    assert_eq!(utilities::align_forward(d, 128), 128);
    assert_eq!(utilities::align_forward(d, 256), 256);
    assert_eq!(utilities::align_forward(d, 512), 512);
    assert_eq!(utilities::align_forward(d, 1024), 1024);

    let e = 1usize;
    assert_eq!(utilities::align_forward(e, 16), 16);
    assert_eq!(utilities::align_forward(e, 32), 32);
    assert_eq!(utilities::align_forward(e, 64), 64);
    assert_eq!(utilities::align_forward(e, 128), 128);
    assert_eq!(utilities::align_forward(e, 256), 256);
    assert_eq!(utilities::align_forward(e, 512), 512);
    assert_eq!(utilities::align_forward(e, 1024), 1024);

    let f = 2usize;
    assert_eq!(utilities::align_forward(f, 2), 2);
    assert_eq!(utilities::align_forward(f, 4), 4);
    assert_eq!(utilities::align_forward(f, 128), 128);

    let g = 64usize;
    assert_eq!(utilities::align_forward(g, 64), 64);
    assert_eq!(utilities::align_forward(g, 32), 64);
    assert_eq!(utilities::align_forward(g, 65), 128);

    let h = 129;
    assert_eq!(utilities::align_forward(h, 256), 256);
    assert_eq!(utilities::align_forward(h, 64), 192);

    let i = 702;
    assert_eq!(utilities::align_forward(i, 4), 704);
}

// TODO: Add is_aligned, align_ptr_forward, bytes_to_typed, typed_to_bytes, any_as_u8_slice, divide_up_multiple_usize, divide_up_multiple
