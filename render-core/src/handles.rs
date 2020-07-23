#![allow(dead_code)]

use crate::types::RenderResourceType;
use enum_primitive::FromPrimitive;
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use std::ops::{Shl, Shr};
use strum::EnumCount;

type RenderResourceId = u16;
type RenderResourceCookie = u16;

// TODO: Add bit-wise limits

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq)]
pub struct RenderResourceHandle {
    index: RenderResourceId,
    kind: RenderResourceType,
    cookie: RenderResourceCookie,
}

impl Default for RenderResourceHandle {
    #[inline(always)]
    fn default() -> RenderResourceHandle {
        RenderResourceHandle {
            index: 0u16,
            kind: RenderResourceType::Buffer,
            cookie: 0u16,
        }
    }
}

impl Hash for RenderResourceHandle {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.index.hash(state);
        self.kind.hash(state);
    }
}

impl RenderResourceHandle {
    #[inline(always)]
    pub fn new(
        index: RenderResourceId,
        kind: RenderResourceType,
        cookie: RenderResourceCookie,
    ) -> RenderResourceHandle {
        RenderResourceHandle {
            index,
            kind,
            cookie,
        }
    }

    #[inline(always)]
    pub fn set_id(&mut self, id: RenderResourceId) {
        self.index = id;
    }

    #[inline(always)]
    pub fn set_type(&mut self, kind: RenderResourceType) {
        self.kind = kind;
    }

    #[inline(always)]
    pub fn set_cookie(&mut self, cookie: RenderResourceCookie) {
        self.cookie = cookie;
    }

    #[inline(always)]
    pub fn get_id(&self) -> RenderResourceId {
        self.index
    }

    #[inline(always)]
    pub fn get_type(&self) -> RenderResourceType {
        self.kind
    }

    #[inline(always)]
    pub fn get_cookie(&self) -> RenderResourceCookie {
        self.cookie
    }

    #[inline(always)]
    pub fn get_packed(&self) -> u32 {
        let type_val = self.get_type() as u32;
        type_val.shl(16) | self.get_id() as u32
    }

    #[inline(always)]
    pub fn from_packed(packed: u32, cookie: u16) -> RenderResourceHandle {
        let type_val = (packed & 0xffff0000).shr(16) as u16;
        let id_val = (packed & 0x0000ffff) as u16;
        match RenderResourceType::from_u16(type_val) {
            Some(kind) => RenderResourceHandle::new(id_val, kind, cookie),
            None => {
                assert!(
                    false,
                    "failed to create render resource handle from packed value: {}",
                    packed
                );
                RenderResourceHandle::new(0, RenderResourceType::Buffer, 0)
            }
        }
    }

    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        if self.index == 0u16 && self.kind == RenderResourceType::Buffer && self.cookie == 0u16 {
            false
        } else {
            true
        }
    }
}

#[derive(Default, Debug)]
pub struct RenderResourceHandleAllocator {
    cookies: Vec<u16>,
    unallocated: Vec<VecDeque<u16>>,
    allocated: Vec<VecDeque<u8>>,
}

impl RenderResourceHandleAllocator {
    #[inline(always)]
    pub fn new() -> RenderResourceHandleAllocator {
        let cookie_size = RenderResourceType::count() * RenderResourceCookie::max_value() as usize;
        RenderResourceHandleAllocator {
            cookies: vec![1u16; cookie_size],
            unallocated: vec![VecDeque::new(); RenderResourceType::count()],
            allocated: vec![VecDeque::new(); RenderResourceType::count()],
        }
    }

    #[inline(always)]
    pub fn allocate(&mut self, kind: RenderResourceType) -> RenderResourceHandle {
        let tracking = kind as usize;
        let mut handle = RenderResourceHandle::new(0, kind, 0u16);
        let mut valid_handle = false;
        if self.unallocated[tracking].len() > 0 {
            handle.set_id(self.unallocated[tracking].pop_back().unwrap());
            valid_handle = true;
        } else {
            if self.allocated[tracking].len() < RenderResourceId::max_value() as usize {
                self.allocated[tracking].push_back(0);
                handle.set_id(self.allocated[tracking].len() as u16 - 1);
                valid_handle = true;
            }
        }

        // TODO: Error handling around valid_handle
        if valid_handle {
            let handle_id = handle.get_id() as usize;
            if handle_id as usize >= self.allocated[tracking].len() {
                self.allocated[tracking].resize(handle_id + 1, 0);
            }
            assert!(self.allocated[tracking][handle_id] == 0);
            self.allocated[tracking][handle_id] = 1;
            let cookie = self.get_cookie(handle.get_id(), handle.get_type());
            assert!(cookie != 0);
            handle.set_cookie(cookie);
        }

        handle
    }

    #[inline(always)]
    pub fn release(&mut self, handle: RenderResourceHandle) {
        assert!(self.is_valid(&handle), "Attempting to free invalid handle.");
        let tracking = handle.get_type() as usize;
        assert!(tracking < self.unallocated.len() && tracking < self.allocated.len());
        let _cookie = self.inc_cookie(handle.get_id(), handle.get_type());
        self.allocated[tracking][handle.get_id() as usize] = 0;
        self.unallocated[tracking].push_back(handle.get_id());
    }

    #[inline(always)]
    pub fn is_allocated(&self, index: RenderResourceId, kind: RenderResourceType) -> bool {
        let tracking = kind as usize;
        let index = index as usize;
        if tracking < self.unallocated.len() && tracking < self.allocated.len() {
            let deque = &self.allocated[tracking];
            if index < deque.len() {
                deque[index] == 1
            } else {
                false
            }
        } else {
            false
        }
    }

    #[inline(always)]
    pub fn is_valid(&self, handle: &RenderResourceHandle) -> bool {
        let cookie = self.get_cookie(handle.get_id(), handle.get_type());
        cookie == handle.get_cookie()
    }

    #[inline(always)]
    pub fn get_count(&self, kind: RenderResourceType) -> usize {
        let tracking = kind as usize;
        assert!(tracking < self.unallocated.len() && tracking < self.allocated.len());
        let mut handle_count = 0;
        let allocated = &self.allocated[tracking];
        for entry in allocated {
            if *entry > 0 {
                handle_count += 1;
            }
        }
        handle_count
    }

    #[inline(always)]
    pub fn get_max(&self, kind: RenderResourceType) -> usize {
        let tracking = kind as usize;
        assert!(tracking < self.unallocated.len() && tracking < self.allocated.len());
        self.allocated[tracking].len()
    }

    #[inline(always)]
    pub(crate) fn get_cookie(
        &self,
        index: RenderResourceId,
        kind: RenderResourceType,
    ) -> RenderResourceCookie {
        let offset = kind as usize * RenderResourceCookie::max_value() as usize;
        self.cookies[offset + index as usize]
    }

    #[inline(always)]
    pub(crate) fn inc_cookie(
        &mut self,
        index: RenderResourceId,
        kind: RenderResourceType,
    ) -> RenderResourceCookie {
        let offset = kind as usize * RenderResourceCookie::max_value() as usize;
        let mut cookie = self.cookies[offset + index as usize];

        if (cookie as usize) + 1 > RenderResourceCookie::max_value() as usize {
            cookie = 1;
        } else {
            cookie += 1;
        }

        self.cookies[offset + index as usize] = cookie;
        cookie
    }
}
