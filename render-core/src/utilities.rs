use std::mem;
use std::slice;

#[inline(always)]
pub fn divide_up_multiple(val: u32, align: u32) -> u32 {
    (val + (align - 1)) / align
}

// TODO: Make generic
#[inline(always)]
pub fn divide_up_multiple_usize(val: usize, align: usize) -> usize {
    (val + (align - 1)) / align
}

#[inline(always)]
pub fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe { slice::from_raw_parts((p as *const T) as *const u8, mem::size_of::<T>()) }
}

#[inline(always)]
pub fn typed_to_bytes<T>(typed: &[T]) -> &[u8] {
    unsafe {
        slice::from_raw_parts(
            typed.as_ptr() as *const u8,
            typed.len() * mem::size_of::<T>(),
        )
    }
}

#[inline(always)]
pub fn bytes_to_typed<T>(bytes: &mut [u8]) -> &mut [T] {
    unsafe {
        slice::from_raw_parts_mut(
            bytes.as_mut_ptr() as *mut T,
            bytes.len() / mem::size_of::<T>(),
        )
    }
}

/// Aligns a pointer forward to the next value aligned with `align`.
#[inline(always)]
pub fn align_ptr_forward(ptr: *mut u8, align: usize) -> *mut u8 {
    ((ptr as usize + align - 1) & !(align - 1)) as *mut u8
}

/// Aligns an offset forward to the next value aligned with `align`.
#[inline(always)]
pub fn align_forward(offset: usize, align: usize) -> usize {
    ((offset + align - 1) & !(align - 1))
}

/// Checks if an offset is aligned with 'align'.
#[inline(always)]
pub fn is_aligned(offset: usize, align: usize) -> bool {
    align_forward(offset, align) == offset
}
