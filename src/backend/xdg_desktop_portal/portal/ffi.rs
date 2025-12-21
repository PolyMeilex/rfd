#![allow(unused)]

use std::{
    ffi::CStr,
    os::raw::{c_char, c_int, c_uint, c_void},
    ptr::NonNull,
    sync::OnceLock,
};

pub type DBusConnection = c_void;
pub type DBusMessage = c_void;

#[repr(C)]
#[derive(Debug, PartialEq, Copy, Clone)]
/// System or Session bus
pub enum DBusBusType {
    Session = 0,
    System = 1,
    Starter = 2,
}

pub const DBUS_TYPE_ARRAY: c_int = 'a' as c_int;
pub const DBUS_TYPE_VARIANT: c_int = 'v' as c_int;
pub const DBUS_TYPE_BOOLEAN: c_int = 'b' as c_int;
pub const DBUS_TYPE_INVALID: c_int = 0;
pub const DBUS_TYPE_STRING: c_int = 's' as c_int;
pub const DBUS_TYPE_DICT_ENTRY: c_int = 'e' as c_int;
pub const DBUS_TYPE_BYTE: c_int = 'y' as c_int;
pub const DBUS_TYPE_INT16: c_int = 'n' as c_int;
pub const DBUS_TYPE_UINT16: c_int = 'q' as c_int;
pub const DBUS_TYPE_INT32: c_int = 'i' as c_int;
pub const DBUS_TYPE_UINT32: c_int = 'u' as c_int;
pub const DBUS_TYPE_INT64: c_int = 'x' as c_int;
pub const DBUS_TYPE_UINT64: c_int = 't' as c_int;
pub const DBUS_TYPE_DOUBLE: c_int = 'd' as c_int;
pub const DBUS_TYPE_UNIX_FD: c_int = 'h' as c_int;
pub const DBUS_TYPE_STRUCT: c_int = 'r' as c_int;
pub const DBUS_TYPE_OBJECT_PATH: c_int = 'o' as c_int;
pub const DBUS_TYPE_SIGNATURE: c_int = 'g' as c_int;

#[repr(C)]
pub struct DBusError {
    pub name: *const c_char,
    pub message: *const c_char,
    pub dummy: c_uint,
    pub padding1: *const c_void,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DBusMessageIter {
    pub dummy1: *mut c_void,
    pub dummy2: *mut c_void,
    pub dummy3: u32,
    pub dummy4: c_int,
    pub dummy5: c_int,
    pub dummy6: c_int,
    pub dummy7: c_int,
    pub dummy8: c_int,
    pub dummy9: c_int,
    pub dummy10: c_int,
    pub dummy11: c_int,
    pub pad1: c_int,
    pub pad2: c_int,
    // Here would have been padding; because pad3 is a 8 byte aligned pointer (on amd64).
    // Rust reserves the right not to copy the padding when cloning a struct,
    // but the D-Bus library uses this part of the struct too.
    // Hence, add a field as big as the padding to ensure Rust copies all data.
    pub pad2_added_by_rust: c_int,
    pub pad3: *mut c_void,
}

pub struct Libdbus {
    _handle: Liblary,
    // Bus Connections
    pub dbus_bus_get_private:
        unsafe extern "C" fn(DBusBusType, *mut DBusError) -> *mut DBusConnection,
    pub dbus_bus_get_unique_name: unsafe extern "C" fn(*mut DBusConnection) -> *const c_char,

    pub dbus_bus_add_match:
        unsafe extern "C" fn(*mut DBusConnection, *const c_char, *mut DBusError),

    // Connection Management
    pub dbus_connection_close: unsafe extern "C" fn(*mut DBusConnection),
    pub dbus_connection_flush: unsafe extern "C" fn(*mut DBusConnection),
    pub dbus_connection_send_with_reply_and_block: unsafe extern "C" fn(
        *mut DBusConnection,
        *mut DBusMessage,
        c_int,
        *mut DBusError,
    ) -> *mut DBusMessage,
    pub dbus_connection_read_write: unsafe extern "C" fn(*mut DBusConnection, c_int) -> u32,
    pub dbus_connection_pop_message: unsafe extern "C" fn(*mut DBusConnection) -> *mut DBusMessage,

    // Errors
    pub dbus_error_init: unsafe extern "C" fn(*mut DBusError),
    pub dbus_error_free: unsafe extern "C" fn(*mut DBusError),

    // Messages
    pub dbus_message_new_method_call: unsafe extern "C" fn(
        *const c_char,
        *const c_char,
        *const c_char,
        *const c_char,
    ) -> *mut DBusMessage,
    pub dbus_message_unref: unsafe extern "C" fn(*mut DBusMessage),
    pub dbus_message_is_signal:
        unsafe extern "C" fn(*mut DBusMessage, *const c_char, *const c_char) -> u32,
    pub dbus_message_get_path: unsafe extern "C" fn(*mut DBusMessage) -> *const c_char,

    // Message Iterators
    pub dbus_message_iter_append_basic:
        unsafe extern "C" fn(*mut DBusMessageIter, c_int, *const c_void) -> u32,
    pub dbus_message_iter_init: unsafe extern "C" fn(*mut DBusMessage, *mut DBusMessageIter) -> u32,
    pub dbus_message_iter_init_append: unsafe extern "C" fn(*mut DBusMessage, *mut DBusMessageIter),
    pub dbus_message_iter_get_arg_type: unsafe extern "C" fn(*mut DBusMessageIter) -> c_int,
    pub dbus_message_iter_get_basic: unsafe extern "C" fn(*mut DBusMessageIter, *mut c_void),
    pub dbus_message_iter_next: unsafe extern "C" fn(*mut DBusMessageIter) -> u32,
    pub dbus_message_iter_recurse: unsafe extern "C" fn(*mut DBusMessageIter, *mut DBusMessageIter),
    pub dbus_message_iter_open_container: unsafe extern "C" fn(
        *mut DBusMessageIter,
        c_int,
        *const c_char,
        *mut DBusMessageIter,
    ) -> u32,
    pub dbus_message_iter_close_container:
        unsafe extern "C" fn(*mut DBusMessageIter, *mut DBusMessageIter) -> u32,
}

unsafe impl Send for Libdbus {}
unsafe impl Sync for Libdbus {}

macro_rules! load_symbols {
    ($handle:expr, [ $($name:ident),* $(,)? ]) => {
        #[allow(clippy::missing_transmute_annotations)]
        Self {
            $(
                $name: unsafe {
                    // 1. Stringify the identifier name
                    // 2. Append null terminator
                    // 3. Convert to CStr (equivalent to c"name")
                    // 4. Load symbol and transmute
                    std::mem::transmute($handle.symbol(
                        std::ffi::CStr::from_bytes_with_nul_unchecked(
                            concat!(stringify!($name), "\0").as_bytes()
                        )
                    )?)
                },
            )*
            _handle: $handle,
        }
    };
}

static LIB: OnceLock<Libdbus> = OnceLock::new();

impl Libdbus {
    fn new(mut handle: Liblary) -> Option<Self> {
        Some(load_symbols!(
            handle,
            [
                // Bus Connections
                dbus_bus_get_private,
                dbus_bus_get_unique_name,
                dbus_bus_add_match,
                // Connection Management
                dbus_connection_close,
                dbus_connection_flush,
                dbus_connection_send_with_reply_and_block,
                dbus_connection_read_write,
                dbus_connection_pop_message,
                // Errors
                dbus_error_init,
                dbus_error_free,
                // Messages
                dbus_message_new_method_call,
                dbus_message_unref,
                dbus_message_is_signal,
                dbus_message_get_path,
                // Message Iterators
                dbus_message_iter_append_basic,
                dbus_message_iter_init,
                dbus_message_iter_init_append,
                dbus_message_iter_get_arg_type,
                dbus_message_iter_get_basic,
                dbus_message_iter_next,
                dbus_message_iter_recurse,
                dbus_message_iter_open_container,
                dbus_message_iter_close_container,
            ]
        ))
    }

    pub fn open_libdbus() -> Option<&'static Self> {
        if let Some(lib) = LIB.get() {
            return Some(lib);
        }

        let lib = Liblary::open(c"libdbus-1.so.3").or_else(|| Liblary::open(c"libdbus-1.so"))?;

        let lib = Self::new(lib)?;
        LIB.set(lib).ok();
        LIB.get()
    }

    pub fn get() -> &'static Self {
        LIB.get().unwrap()
    }
}

struct Liblary(pub NonNull<c_void>);
unsafe impl Send for Liblary {}
unsafe impl Sync for Liblary {}

impl Liblary {
    pub fn open(lib: &CStr) -> Option<Self> {
        unsafe {
            NonNull::new(libc::dlopen(
                lib.as_ptr(),
                libc::RTLD_LAZY | libc::RTLD_LOCAL,
            ))
            .map(Self)
        }
    }

    pub fn symbol(&mut self, symbol: &CStr) -> Option<NonNull<c_void>> {
        unsafe { NonNull::new(libc::dlsym(self.0.as_ptr(), symbol.as_ptr())) }
    }
}

impl Drop for Liblary {
    fn drop(&mut self) {
        unsafe { libc::dlclose(self.0.as_ptr()) };
    }
}
