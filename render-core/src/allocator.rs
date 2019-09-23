use crate::error::{Error, Result};
use crate::utilities::{align_forward, align_ptr_forward};
use failure::Fail;
use std::cmp;
use std::marker::PhantomData;
use std::mem;
use std::slice;

pub type LinearAllocatorMark = usize;

/// A basic linear allocator that wraps an externally owned contiguous region of memory.
pub struct LinearAllocator<'a> {
    pos: usize,
    size: usize,
    data: Vec<u8>,
    phantom: PhantomData<&'a Vec<u8>>,
}

impl<'a> LinearAllocator<'a> {
    pub fn new(size: usize) -> Self {
        LinearAllocator {
            pos: 0,
            size,
            data: vec![0u8; size],
            phantom: PhantomData,
        }
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.pos == 0
    }

    #[inline(always)]
    pub fn size(&self) -> usize {
        self.pos
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.size
    }

    #[inline(always)]
    pub fn mark(&self) -> LinearAllocatorMark {
        self.pos
    }

    #[inline(always)]
    pub fn available(&self) -> usize {
        self.size() - self.capacity()
    }

    /// Allocate `size` bytes with minimum `alignment` at `offset` bytes into the allocation.
    #[inline(always)]
    pub fn allocate_raw(
        &mut self,
        size: usize,
        alignment: usize,
        offset: usize,
    ) -> Result<LinearAllocatorMark> {
        let pos = cmp::min(
            align_forward(self.mark() + offset, alignment) - offset + size,
            self.capacity() + 1,
        );
        if (self.pos + size) > self.capacity() {
            error!(
                "ran out of space in linear allocator! requested: {}, position: {}, capacity: {}",
                self.pos,
                size,
                self.capacity()
            );
            panic!("TODO: Error here");
        } else {
            self.pos = pos;
            Ok(self.pos - size)
        }
    }

    #[inline(always)]
    pub fn allocate_typed<T>(&mut self) -> Result<LinearAllocatorMark> {
        let (type_size, type_align) = (mem::size_of::<T>(), mem::align_of::<T>());
        let mark = self.allocate_raw(type_size, type_align, 0)?;
        Ok(mark)
    }

    #[inline(always)]
    pub fn mark_ref<T>(&self, mark: LinearAllocatorMark) -> Result<&'a T> {
        unsafe {
            let base_ptr = self.data.as_ptr();
            let type_ptr = base_ptr.offset(mark as isize) as *const T;
            Ok(&*type_ptr)
        }
    }

    #[inline(always)]
    pub fn mark_mut<T>(&mut self, mark: LinearAllocatorMark) -> Result<&'a mut T> {
        unsafe {
            let base_ptr = self.data.as_mut_ptr();
            let type_ptr = base_ptr.offset(mark as isize) as *mut T;
            Ok(&mut *type_ptr)
        }
    }

    #[inline(always)]
    pub fn mark_place<T>(&mut self, mark: LinearAllocatorMark, entry: T) -> Result<&'a T> {
        unsafe {
            let base_ptr = self.data.as_mut_ptr();
            let type_ptr = base_ptr.offset(mark as isize) as *mut T;
            *type_ptr = entry;
            Ok(&*type_ptr)
        }
    }

    #[inline(always)]
    pub fn mark_insert(&mut self, mark: LinearAllocatorMark, data: &[u8]) -> Result<()> {
        // TODO: Error handling - make sure ranges and lengths are correct
        let end = mark + data.len();
        self.data[mark..end].copy_from_slice(data);
        Ok(())
    }

    #[inline(always)]
    pub fn mark_data(&'a self, mark: LinearAllocatorMark, size: usize) -> Result<&'a [u8]> {
        let end = mark + size;
        Ok(&self.data[mark..end])
    }

    #[inline(always)]
    pub fn rewind(&mut self, mark: LinearAllocatorMark) -> Result<()> {
        self.pos = mark;
        Ok(())
    }
}
