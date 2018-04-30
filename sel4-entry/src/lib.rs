#![no_std]
#![feature(lang_items, core_intrinsics)]

extern crate sel4_sys;

use sel4_sys::*;

pub static mut BOOTINFO: *mut seL4_BootInfo = (0 as *mut seL4_BootInfo);
static mut RUN_ONCE: bool = false;

#[lang = "termination"]
trait Termination {
    fn report(self) -> i32;
}

impl Termination for () {
    fn report(self) -> i32 {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn __sel4_start_init_boot_info(
    bootinfo: *mut seL4_BootInfo,
) {
    if !RUN_ONCE {
        BOOTINFO = bootinfo;
        RUN_ONCE = true;
        seL4_SetUserData((*bootinfo).ipcBuffer as usize as seL4_Word);
    }
}

#[lang = "start"]
fn lang_start<T: Termination + 'static>(
    main: fn() -> T,
    _argc: isize,
    _argv: *const *const u8,
) -> isize {
    main();
    panic!("Root task should never return from main!");
}

#[lang = "panic_fmt"]
extern "C" fn panic_fmt(
    fmt: core::fmt::Arguments,
    file: &'static str,
    line: u32,
) -> ! {
    use core::fmt::Write;
    let _ = write!(
        sel4_sys::DebugOutHandle,
        "panic at {}:{}: ",
        file,
        line
    );
    let _ = sel4_sys::DebugOutHandle.write_fmt(fmt);
    let _ = sel4_sys::DebugOutHandle.write_char('\n');
    let _ = write!(
        sel4_sys::DebugOutHandle,
        "----- aborting from panic -----\n"
    );
    unsafe {
        core::intrinsics::abort();
    }
}

#[lang = "eh_personality"]
fn eh_personality() {
    use core::fmt::Write;
    let _ = write!(
        sel4_sys::DebugOutHandle,
        "----- aborting from eh_personality -----\n"
    );
    unsafe {
        core::intrinsics::abort();
    }
}

#[lang = "oom"]
pub extern "C" fn oom() -> ! {
    use core::fmt::Write;
    let _ = write!(
        sel4_sys::DebugOutHandle,
        "----- aborting from out-of-memory -----\n"
    );
    unsafe {
        core::intrinsics::abort();
    }
}
