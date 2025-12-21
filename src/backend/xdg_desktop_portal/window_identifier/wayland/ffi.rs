#![allow(non_camel_case_types)]

use std::{
    ffi::{c_char, c_int, c_void, CStr},
    fmt,
    rc::Rc,
};

pub enum wl_proxy {}
pub enum wl_display {}
pub enum wl_event_queue {}

#[repr(C)]
pub struct wl_message {
    pub name: *const c_char,
    pub signature: *const c_char,
    pub types: *const *const wl_interface,
}

unsafe impl Send for wl_message {}
unsafe impl Sync for wl_message {}

#[repr(C)]
pub struct wl_interface {
    pub name: *const c_char,
    pub version: c_int,
    pub request_count: c_int,
    pub requests: *const wl_message,
    pub event_count: c_int,
    pub events: *const wl_message,
}

impl fmt::Debug for wl_interface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "wl_interface@{:p}", self)
    }
}

unsafe impl Send for wl_interface {}
unsafe impl Sync for wl_interface {}

#[derive(Debug)]
pub struct WlEventQueueHandle {
    wl_event_queue: *mut wl_event_queue,
    lib: LibWayland,
}

impl WlEventQueueHandle {
    pub fn new(lib: &LibWayland, wl_display: *mut wl_display) -> Self {
        let wl_event_queue = (lib.wl_display_create_queue_with_name)(wl_display, c"rfd".as_ptr());
        assert!(!wl_event_queue.is_null());

        Self {
            wl_event_queue,
            lib: lib.clone(),
        }
    }

    pub fn as_ptr(&self) -> *mut wl_event_queue {
        self.wl_event_queue
    }
}

impl Drop for WlEventQueueHandle {
    fn drop(&mut self) {
        (self.lib.wl_event_queue_destroy)(self.wl_event_queue);
    }
}

#[derive(Debug)]
pub struct WlProxyHandle {
    wl_proxy: *mut wl_proxy,
    lib: LibWayland,
}

impl WlProxyHandle {
    pub fn new(wl_proxy: *mut wl_proxy, lib: LibWayland) -> Self {
        Self { wl_proxy, lib }
    }

    pub fn as_ptr(&self) -> *mut wl_proxy {
        self.wl_proxy
    }
}

impl Drop for WlProxyHandle {
    fn drop(&mut self) {
        (self.lib.wl_proxy_destroy)(self.wl_proxy);
    }
}

struct DlHandle(*mut c_void);

impl Drop for DlHandle {
    fn drop(&mut self) {
        unsafe { libc::dlclose(self.0) };
    }
}

#[derive(Clone)]
pub struct LibWayland {
    pub wl_display_create_queue_with_name:
        extern "C" fn(display: *mut wl_display, name: *const c_char) -> *mut wl_event_queue,
    pub wl_display_roundtrip_queue: extern "C" fn(*mut wl_display, *mut wl_event_queue) -> c_int,

    pub wl_event_queue_destroy: extern "C" fn(*mut wl_event_queue),

    #[allow(unused)]
    pub wl_proxy_marshal_flags: extern "C" fn(
        proxy: *mut wl_proxy,
        opcode: u32,
        interface: *const wl_interface,
        version: u32,
        flags: u32,
        ...
    ) -> *mut wl_proxy,

    pub wl_proxy_marshal_constructor:
        extern "C" fn(*mut wl_proxy, u32, *const wl_interface, ...) -> *mut wl_proxy,
    pub wl_proxy_marshal_constructor_versioned:
        extern "C" fn(*mut wl_proxy, u32, *const wl_interface, u32, ...) -> *mut wl_proxy,

    pub wl_proxy_add_listener:
        extern "C" fn(*mut wl_proxy, *mut extern "C" fn(), *mut c_void) -> c_int,

    pub wl_proxy_set_queue: extern "C" fn(*mut wl_proxy, *mut wl_event_queue),
    pub wl_proxy_set_user_data: extern "C" fn(*mut wl_proxy, *mut c_void) -> (),

    pub wl_proxy_create_wrapper: extern "C" fn(*mut wl_proxy) -> *mut wl_proxy,
    pub wl_proxy_wrapper_destroy: extern "C" fn(*mut wl_proxy),
    pub wl_proxy_destroy: extern "C" fn(*mut wl_proxy),

    _lib: Rc<DlHandle>,
}

impl fmt::Debug for LibWayland {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LibWayland").finish_non_exhaustive()
    }
}

impl LibWayland {
    pub fn new() -> Option<Self> {
        unsafe {
            let lib = Rc::new(DlHandle(libc::dlopen(
                c"libwayland-client.so.0".as_ptr(),
                libc::RTLD_LAZY | libc::RTLD_LOCAL,
            )));

            macro_rules! load_symbols {
                ($handle:expr, [ $($name:ident),* $(,)? ]) => {
                    Self {
                        $(
                            $name: {
                                let name = concat!(stringify!($name), "\0");
                                let sym = libc::dlsym(
                                    $handle.0,
                                    CStr::from_bytes_with_nul_unchecked(name.as_bytes()).as_ptr()
                                );

                                if sym.is_null() {
                                    log::error!("Symbol '{name}' not found");
                                    return None;
                                }

                                #[allow(clippy::missing_transmute_annotations)]
                                std::mem::transmute(sym)
                            },
                        )*
                        _lib: $handle,
                    }
                };
            }

            let lib = load_symbols!(
                lib,
                [
                    wl_display_create_queue_with_name,
                    wl_display_roundtrip_queue,
                    wl_event_queue_destroy,
                    wl_proxy_marshal_flags,
                    wl_proxy_marshal_constructor,
                    wl_proxy_marshal_constructor_versioned,
                    wl_proxy_add_listener,
                    wl_proxy_set_queue,
                    wl_proxy_set_user_data,
                    wl_proxy_create_wrapper,
                    wl_proxy_wrapper_destroy,
                    wl_proxy_destroy,
                ]
            );

            Some(lib)
        }
    }
}

pub struct SyncWrapper<T>(pub T);
unsafe impl<T> Sync for SyncWrapper<T> {}

pub mod display {
    #[allow(unused)]
    pub const WL_DISPLAY_SYNC: u32 = 0;
    pub const WL_DISPLAY_GET_REGISTRY: u32 = 1;
}

pub mod registry {
    use super::{wl_interface, wl_message};

    pub const WL_REGISTRY_BIND: u32 = 0;

    pub static INTERFACE: wl_interface = wl_interface {
        name: c"wl_registry".as_ptr(),
        version: 1,
        request_count: 1,
        requests: {
            static MESSAGES: [wl_message; 1] = [wl_message {
                name: c"bind".as_ptr(),
                signature: c"usun".as_ptr(),
                types: {
                    static TYPES: [Option<&'static wl_interface>; 4] = [None, None, None, None];
                    TYPES.as_ptr().cast()
                },
            }];
            MESSAGES.as_ptr()
        },
        event_count: 2,
        events: {
            static MESSAGES: [wl_message; 2] = [
                wl_message {
                    name: c"global".as_ptr(),
                    signature: c"usu".as_ptr(),
                    types: {
                        static TYPES: [Option<&'static wl_interface>; 3] = [None, None, None];
                        TYPES.as_ptr().cast()
                    },
                },
                wl_message {
                    name: c"global_remove".as_ptr(),
                    signature: c"u".as_ptr(),
                    types: {
                        static TYPES: [Option<&'static wl_interface>; 1] = [None];
                        TYPES.as_ptr().cast()
                    },
                },
            ];
            MESSAGES.as_ptr()
        },
    };
}

pub mod xdg_exporter {
    use std::ptr::{self, null};

    use super::{wl_interface, wl_message};

    #[allow(unused)]
    pub const ZXDG_EXPORTER_V2_DESTROY: u32 = 0;
    pub const ZXDG_EXPORTER_V2_EXPORT_TOPLEVEL: u32 = 1;

    pub static INTERFACE: wl_interface = wl_interface {
        name: c"zxdg_exporter_v2".as_ptr(),
        version: 1,
        request_count: 2,
        requests: {
            static MESSAGES: [wl_message; 2] = [
                wl_message {
                    name: c"destroy".as_ptr(),
                    signature: c"".as_ptr(),
                    types: {
                        static TYPES: [Option<&'static wl_interface>; 1] = [None];
                        TYPES.as_ptr().cast()
                    },
                },
                wl_message {
                    name: c"export_toplevel".as_ptr(),
                    signature: c"no".as_ptr(),
                    types: {
                        static WL_SURFACE_INTERFACE: wl_interface = wl_interface {
                            name: c"wl_surface".as_ptr(),
                            version: 6,
                            request_count: 0,
                            requests: ptr::null(),
                            event_count: 0,
                            events: ptr::null(),
                        };
                        static MESSAGES: [Option<&'static wl_interface>; 2] = [
                            Some(&super::xdg_exported::INTERFACE),
                            Some(&WL_SURFACE_INTERFACE),
                        ];
                        MESSAGES.as_ptr().cast()
                    },
                },
            ];
            MESSAGES.as_ptr()
        },
        event_count: 0,
        events: null::<wl_message>(),
    };
}

pub mod xdg_exported {

    use super::{wl_interface, wl_message};

    pub static INTERFACE: wl_interface = wl_interface {
        name: c"zxdg_exported_v2".as_ptr(),
        version: 1,
        request_count: 1,
        requests: {
            static MESSAGES: [wl_message; 1] = [wl_message {
                name: c"destroy".as_ptr(),
                signature: c"".as_ptr(),
                types: {
                    static TYPES: [Option<&'static wl_interface>; 1] = [None];
                    TYPES.as_ptr().cast()
                },
            }];
            MESSAGES.as_ptr()
        },
        event_count: 1,
        events: {
            static MESSAGES: [wl_message; 1] = [wl_message {
                name: c"handle".as_ptr(),
                signature: c"s".as_ptr(),
                types: {
                    static TYPES: [Option<&'static wl_interface>; 1] = [None];
                    TYPES.as_ptr().cast()
                },
            }];
            MESSAGES.as_ptr()
        },
    };
}
