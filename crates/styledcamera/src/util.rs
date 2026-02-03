use std::os::raw::c_char;

pub(crate) fn cstr(bytes: &'static [u8]) -> *const c_char {
    debug_assert!(
        bytes.last() == Some(&0),
        "C string must be NUL-terminated"
    );
    bytes.as_ptr().cast()
}
