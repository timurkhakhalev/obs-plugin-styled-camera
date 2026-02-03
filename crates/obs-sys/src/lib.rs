#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub use libc::{c_char, c_double, c_float, c_int, c_long, c_longlong, c_short, c_uchar, c_uint, c_ulong, c_ulonglong, c_ushort, c_void};

pub type size_t = libc::size_t;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
