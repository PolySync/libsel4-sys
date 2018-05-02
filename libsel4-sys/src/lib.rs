#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/sel4_config.rs"));

extern crate rlibc;

mod c_types {
    pub type c_uint = u32;
    pub type c_int = i32;

    pub type c_ulong = u64;
    pub type c_long = u32;

    pub type c_uchar = u8;
    pub type c_char = i8;
    pub type c_schar = i8;

    pub type c_ushort = u16;
    pub type c_short = i16;

    pub type c_ulonglong = u64;
    pub type c_longlong = i64;
}

#[cfg(feature = "KERNEL_PRINTING")]
pub struct DebugOutHandle;

#[cfg(feature = "KERNEL_PRINTING")]
impl ::core::fmt::Write for DebugOutHandle {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        for &b in s.as_bytes() {
            unsafe { self::seL4_DebugPutChar(b as i8) };
        }
        Ok(())
    }
}

#[no_mangle]
pub unsafe extern "C" fn __assert_fail(
    mstr: *const c_types::c_char,
    file: *const c_types::c_char,
    line: c_types::c_int,
    function: *const c_types::c_char,
) {
    panic!("assertion failed");
}

#[no_mangle]
pub unsafe extern "C" fn stpcpy(
    dest: *mut c_types::c_schar,
    source: *const c_types::c_schar,
) -> *mut c_types::c_schar {
    for i in 0.. {
        *dest.offset(i) = *source.offset(i);
        if *dest.offset(i) == 0 {
            break;
        }
    }

    dest
}

#[no_mangle]
pub unsafe extern "C" fn strcpy(
    dest: *mut c_types::c_schar,
    source: *const c_types::c_schar,
) -> *mut c_types::c_schar {
    stpcpy(dest, source);
    dest
}

#[cfg(target = "arm-sel4-helios")]
pub const seL4_WordBits: u32 = 32;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
