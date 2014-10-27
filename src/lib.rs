#![feature(phase)]

#[phase(plugin)] extern crate compile_msg;

extern crate libc;

#[cfg(target_os = "macos")]
compile_error!("breakpad not yet supported on OS X")

pub struct ExceptionHandler {
    eh: *mut libc::c_void
}

#[cfg(target_os = "linux")]
impl ExceptionHandler {
    /// Create a new ExceptionHandler that will write crash dumps into `path`
    pub fn new(path: &Path) -> ExceptionHandler { unsafe {
        use std::ptr::{null, null_mut};
        use std::mem::transmute;
        let s = path.display().to_string();
        let desc = s.with_c_str(|cstr| ffi::rust_breakpad_descriptor_new(cstr));
        let eh = ffi::rust_breakpad_exceptionhandler_new(desc, transmute(null::<()>()),
                                                         transmute(null::<()>()), null_mut(), 1);
        ExceptionHandler {
            eh: eh
        }
    } }
}

#[cfg(target_os = "windows")]
impl ExceptionHandler {
    /// Create a new ExceptionHandler that will write crash dumps into `path`
    pub fn new(path: &Path) -> ExceptionHandler { unsafe {
        use std::ptr::{null, null_mut};
        use std::mem::transmute;
        let mut s = path.display().to_string().to_utf16();
        s.push(0); // NUL terminate
        let eh = ffi::rust_breakpad_exceptionhandler_new(s, transmute(null::<()>()),
                                                         transmute(null::<()>()), null_mut(), 1);
        ExceptionHandler {
            eh: eh
        }
    } }

}

impl ExceptionHandler {
    /// Force writing a crash dump.
    ///
    /// Should *not* be called after a crash, as this uses the heap.
    pub fn write_dump(&self) {
        unsafe {
            ffi::rust_breakpad_exceptionhandler_write_minidump(self.eh);
        }
    }
}

impl Drop for ExceptionHandler {
    fn drop(&mut self) {
        unsafe {
            ffi::rust_breakpad_exceptionhandler_free(self.eh);
        }
    }
}

#[cfg(target_os = "linux")]
pub mod ffi {
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

#[cfg(target_os = "windows")]
pub mod ffi {
    use libc;

    pub type FilterCallback = extern fn(context: *mut libc::c_void) -> libc::c_int;
    pub type MinidumpCallback = extern fn(desc: *mut libc::c_void,
                                          context: *mut libc::c_void,
                                          succeeded: libc::c_int) -> libc::c_int;

    #[link(name = "stdc++")]
    extern { }

    #[link(name = "breakpad-dll")]
    extern {
        pub fn rust_breakpad_exceptionhandler_new(desc: *const libc::char,
                                              fcb: FilterCallback,
                                              mcb: MinidumpCallback,
                                              context: *mut libc::c_void,
                                              install_handler: libc::c_int) -> *mut libc::c_void;
        pub fn rust_breakpad_exceptionhandler_write_minidump(eh: *mut libc::c_void) -> libc::c_int;
        pub fn rust_breakpad_exceptionhandler_free(eh: *mut libc::c_void);
    }
}

/// Catch all task failure with breakpad.
///
/// This [installs](http://doc.rust-lang.org/std/rt/unwind/fn.register.html) a global unwinding
/// callback that will abort the process on task failure. This will trigger a crash dump. Returns
/// false if it could not install the callback.
pub fn catch_task_failure() -> bool {
    fn cb(_: &std::any::Any, _: &'static str, _: uint) {
        let mut wr = std::io::stdio::stdout_raw();
        match wr.write(b"Crash detected! Bailing...\n") {
            Ok(_) => { }
            Err(_) => { /* not much we can do anyway, we're unwinding */ }
        }
        unsafe { std::intrinsics::abort() };
    }
    unsafe { std::rt::unwind::register(cb) }
}
