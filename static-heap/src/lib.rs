#![no_std]
#![feature(lang_items, global_allocator, allocator_api, alloc, core_intrinsics)]

extern crate alloc;
use core::alloc::{GlobalAlloc, Layout, Opaque};

pub static mut ALLOCATE: extern "C" fn(Layout) -> *mut Opaque = unset_allocate;
pub static mut DEALLOCATE: extern "C" fn(*mut Opaque, Layout) =
    unset_deallocate;
pub static mut REALLOCATE: extern "C" fn(*mut Opaque, Layout, usize)
    -> *mut Opaque = unset_reallocate;

pub struct StaticAlloc;

unsafe impl GlobalAlloc for StaticAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut Opaque {
        ALLOCATE(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut Opaque, layout: Layout) {
        DEALLOCATE(ptr, layout);
    }

    unsafe fn realloc(
        &self,
        ptr: *mut Opaque,
        old_layout: Layout,
        new_size: usize,
    ) -> *mut Opaque {
        REALLOCATE(ptr, old_layout, new_size)
    }
}

#[allow(unused_variables)]
extern "C" fn unset_allocate(layout: Layout) -> *mut Opaque {
    0 as _
}

#[allow(unused_variables)]
extern "C" fn unset_deallocate(ptr: *mut Opaque, layout: Layout) {
    unimplemented!();
}

#[allow(unused_variables)]
extern "C" fn unset_reallocate(
    ptr: *mut Opaque,
    old_layout: Layout,
    new_size: usize,
) -> *mut Opaque {
    0 as _
}

const SCRATCH_LEN_BYTES: usize = 1024 * 1024 * 16;
static mut SCRATCH_HEAP: [u8; SCRATCH_LEN_BYTES] = [0; SCRATCH_LEN_BYTES];
static mut SCRATCH_PTR: usize = 0;

#[allow(unused_variables)]
extern "C" fn scratch_allocate(layout: Layout) -> *mut Opaque {
    unsafe {
        SCRATCH_PTR += SCRATCH_PTR % layout.align();
        let res = &mut SCRATCH_HEAP[SCRATCH_PTR];
        SCRATCH_PTR += layout.size();
        if SCRATCH_PTR <= SCRATCH_LEN_BYTES {
            *res as *mut Opaque
        } else {
            0 as *mut Opaque
        }
    }
}

#[allow(unused_variables)]
extern "C" fn scratch_deallocate(ptr: *mut Opaque, layout: Layout) {
    unsafe {
        if SCRATCH_PTR - layout.size() == ptr as usize {
            SCRATCH_PTR -= layout.size();
        }
    }
}

#[allow(unused_variables)]
extern "C" fn scratch_reallocate(
    ptr: *mut Opaque,
    old_layout: Layout,
    new_size: usize,
) -> *mut Opaque {
    scratch_deallocate(ptr, old_layout.clone());
    scratch_allocate(old_layout)
}

pub unsafe fn switch_to_static_heap() {
    ALLOCATE = scratch_allocate;
    DEALLOCATE = scratch_deallocate;
    REALLOCATE = scratch_reallocate;
}
