#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

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

#[cfg(feature = "KernelPrinting")]
pub struct DebugOutHandle;

#[cfg(feature = "KernelPrinting")]
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

#[cfg(all(target_arch = "arm", target_os = "sel4", target_env = "fel4"))]
/// Number of bits in a `seL4_Word`.
///
/// # Remarks
///
/// Normally this is defined as the following macro:
/// ```
/// #define seL4_WordBits (sizeof(seL4_Word) * 8)
/// ```
///
/// For our `arm-sel4-fel4` target see file:
/// `libsel4/sel4_arch_include/aarch32/sel4/sel4_arch/constants.h`
///
/// However due to bindgen not being able to expand functional
/// macros, the type gets ignored.
///
/// For the time being, we just provide the constant here.
///
/// See following issues for more information:
/// - `rust-bindgen/issues/753`
/// - `feL4-dependencies/issues/18`
pub const seL4_WordBits: u32 = 32;

#[cfg(all(target_arch = "aarch64", target_os = "sel4", target_env = "fel4"))]
/// Number of bits in a `seL4_Word`.
///
/// # Remarks
///
/// Normally this is defined as the following macro:
/// ```
/// #define seL4_WordBits (sizeof(seL4_Word) * 8)
/// ```
///
/// For our `aarch64-sel4-fel4` target see file:
/// `libsel4/sel4_arch_include/aarch64/sel4/sel4_arch/constants.h`
///
/// However due to bindgen not being able to expand functional
/// macros, the type gets ignored.
///
/// For the time being, we just provide the constant here.
///
/// See following issues for more information:
/// - `rust-bindgen/issues/753`
/// - `feL4-dependencies/issues/18`
pub const seL4_WordBits: u32 = 64;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
