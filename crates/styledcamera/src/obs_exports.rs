use std::os::raw::c_char;

use obs_sys as obs;

use crate::constants::{MODULE_AUTHOR, MODULE_DESCRIPTION, MODULE_NAME};

#[no_mangle]
pub unsafe extern "C" fn obs_module_ver() -> u32 {
    (obs::LIBOBS_API_MAJOR_VER << 24)
        | (obs::LIBOBS_API_MINOR_VER << 16)
        | obs::LIBOBS_API_PATCH_VER
}

static mut OBS_MODULE_PTR: *mut obs::obs_module_t = std::ptr::null_mut();

#[no_mangle]
pub unsafe extern "C" fn obs_module_set_pointer(module: *mut obs::obs_module_t) {
    OBS_MODULE_PTR = module;
}

#[no_mangle]
pub unsafe extern "C" fn obs_current_module() -> *mut obs::obs_module_t {
    OBS_MODULE_PTR
}

#[no_mangle]
pub unsafe extern "C" fn obs_module_name() -> *const c_char {
    MODULE_NAME.as_ptr().cast()
}

#[no_mangle]
pub unsafe extern "C" fn obs_module_description() -> *const c_char {
    MODULE_DESCRIPTION.as_ptr().cast()
}

#[no_mangle]
pub unsafe extern "C" fn obs_module_author() -> *const c_char {
    MODULE_AUTHOR.as_ptr().cast()
}

#[no_mangle]
pub unsafe extern "C" fn obs_module_load() -> bool {
    crate::filter::register_sources();
    true
}

#[no_mangle]
pub unsafe extern "C" fn obs_module_unload() {}

