#![feature(phase)]

#[phase(plugin)] extern crate compile_msg;

extern crate libc;

#[cfg(target_os = "macos")]
compile_fatal!("breakpad not yet supported on OS X")

#[cfg(windows)]
compile_fatal!("breakpad not yet support on Windows")

#[cfg(target_os = "linux")]
mod ffi {
    use libc;

    pub type FilterCallback = extern fn(context: *mut libc::c_void) -> libc::c_int;
    pub type MinidumpCallback = extern fn(desc: *mut libc::c_void,
                                          context: *mut libc::c_void,
                                          succeeded: libc::c_int) -> libc::c_int;

    #[link(name = "stdc++")]
    extern { }

    #[link(name = "rust_breakpad_client", kind = "static")]
    extern {
        pub fn rust_breakpad_descriptor_new(path: *const libc::c_char) -> *mut libc::c_void;
        pub fn rust_breakpad_descriptor_path(desc: *const libc::c_void) -> *mut libc::c_char;
        pub fn rust_breakpad_descriptor_free(desc: *mut libc::c_void);

        pub fn rust_breakpad_exceptionhandler_new(desc: *mut libc::c_void,
                                              fcb: FilterCallback,
                                              mcb: MinidumpCallback,
                                              context: *mut libc::c_void,
                                              install_handler: libc::c_int) -> *mut libc::c_void;
        pub fn rust_breakpad_exceptionhandler_write_minidump(eh: *mut libc::c_void) -> libc::c_int;
        pub fn rust_breakpad_exceptionhandler_free(eh: *mut libc::c_void);
    }
}

pub unsafe fn handle() {
    use std::ptr::{null, mut_null};
    use std::mem::transmute;

    let desc = "/tmp".with_c_str(|cstr| ffi::rust_breakpad_descriptor_new(cstr));
    ffi::rust_breakpad_exceptionhandler_new(
        desc, transmute(null::<()>()), transmute(null::<()>()), mut_null(), 1
    );
}
