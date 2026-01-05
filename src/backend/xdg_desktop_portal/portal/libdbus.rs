use std::{
    ffi::{c_char, c_void, CStr, CString},
    fmt,
    marker::PhantomData,
    ptr::NonNull,
};

use super::ffi;
use libc::c_int;

macro_rules! f {
    ($f: ident) => {
        (ffi::Libdbus::get().$f)
    };
}

pub struct DbusError(ffi::DBusError);

impl DbusError {
    pub fn new() -> Self {
        unsafe {
            let mut err: ffi::DBusError = std::mem::zeroed();
            f!(dbus_error_init)(&mut err);
            Self(err)
        }
    }

    pub fn as_ptr(&mut self) -> &mut ffi::DBusError {
        &mut self.0
    }

    pub fn is_err(&self) -> bool {
        self.name().is_some()
    }

    pub fn name(&self) -> Option<&CStr> {
        if self.0.name.is_null() {
            None
        } else {
            unsafe { Some(CStr::from_ptr(self.0.name)) }
        }
    }

    pub fn message(&self) -> Option<&CStr> {
        if self.0.message.is_null() {
            None
        } else {
            unsafe { Some(CStr::from_ptr(self.0.message)) }
        }
    }
}

impl fmt::Display for DbusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.name();
        let message = self.message();
        write!(f, "{name:?}: {message:?}")
    }
}

impl Drop for DbusError {
    fn drop(&mut self) {
        unsafe {
            f!(dbus_error_free)(self.as_ptr());
        }
    }
}

pub struct Connection {
    conn: NonNull<ffi::DBusConnection>,
    err: DbusError,
}

impl Connection {
    pub fn new() -> Option<Self> {
        if ffi::Libdbus::open_libdbus().is_none() {
            log::error!("Can't connect to a portal: libdbus-1.so not found");
            return None;
        }

        let mut err = DbusError::new();
        unsafe {
            let ptr = f!(dbus_bus_get_private)(ffi::DBusBusType::Session, err.as_ptr());

            if err.is_err() {
                log::error!("Failed to connect to session bus: {err}");
                return None;
            }

            Some(Self {
                conn: NonNull::new(ptr)?,
                err,
            })
        }
    }

    pub fn err(&mut self) -> &mut DbusError {
        &mut self.err
    }

    pub fn err_ptr(&mut self) -> &mut ffi::DBusError {
        self.err.as_ptr()
    }

    pub fn as_ptr(&self) -> *mut ffi::DBusConnection {
        self.conn.as_ptr()
    }

    pub fn get_unique_name(&mut self) -> CString {
        unsafe { CStr::from_ptr(f!(dbus_bus_get_unique_name)(self.as_ptr())).to_owned() }
    }

    pub fn add_match(&mut self, match_rule: &CStr) {
        unsafe {
            f!(dbus_bus_add_match)(self.as_ptr(), match_rule.as_ptr(), self.err_ptr());
        }
    }

    pub fn flush(&self) {
        unsafe {
            f!(dbus_connection_flush)(self.as_ptr());
        }
    }

    pub fn read_write(&self, timeout_milliseconds: c_int) {
        unsafe {
            f!(dbus_connection_read_write)(self.as_ptr(), timeout_milliseconds);
        }
    }

    pub fn pop_message(&self) -> Option<Message> {
        unsafe { Message::new(f!(dbus_connection_pop_message)(self.as_ptr())) }
    }

    pub fn send_and_block(&mut self, msg: &Message) -> Option<Message> {
        unsafe {
            Message::new(f!(dbus_connection_send_with_reply_and_block)(
                self.as_ptr(),
                msg.as_ptr(),
                -1,
                self.err_ptr(),
            ))
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe {
            f!(dbus_connection_close)(self.as_ptr());
        }
    }
}

pub struct Message(NonNull<ffi::DBusMessage>);

impl Message {
    pub fn new(ptr: *mut ffi::DBusMessage) -> Option<Self> {
        NonNull::new(ptr).map(Self)
    }

    pub fn new_method_call(
        destination: &CStr,
        path: &CStr,
        iface: &CStr,
        method: &CStr,
    ) -> Option<Self> {
        unsafe {
            Self::new(f!(dbus_message_new_method_call)(
                destination.as_ptr(),
                path.as_ptr(),
                iface.as_ptr(),
                method.as_ptr(),
            ))
        }
    }

    pub fn as_ptr(&self) -> *mut ffi::DBusMessage {
        self.0.as_ptr()
    }

    pub fn is_signal(&self, iface: &CStr, signal_name: &CStr) -> bool {
        unsafe {
            f!(dbus_message_is_signal)(self.as_ptr(), iface.as_ptr(), signal_name.as_ptr()) == 1
        }
    }

    pub fn get_path(&self) -> Option<&CStr> {
        unsafe {
            let path = f!(dbus_message_get_path)(self.as_ptr());
            if path.is_null() {
                None
            } else {
                Some(CStr::from_ptr(path))
            }
        }
    }
}

impl Drop for Message {
    fn drop(&mut self) {
        unsafe {
            f!(dbus_message_unref)(self.as_ptr());
        }
    }
}

pub struct MessageIter<'a>(ffi::DBusMessageIter, #[allow(unused)] &'a PhantomData<()>);

impl<'a> MessageIter<'a> {
    pub fn new() -> Self {
        Self(unsafe { std::mem::zeroed() }, &PhantomData)
    }

    pub fn from_msg(msg: &'a Message) -> Self {
        let mut iter = Self::new();
        iter.init(msg);
        iter
    }

    pub fn iter_recurse(&mut self) -> Self {
        unsafe {
            let mut new = Self::new();
            f!(dbus_message_iter_recurse)(self.as_ptr(), new.as_ptr());
            new
        }
    }

    pub fn init(&mut self, msg: &'a Message) {
        unsafe {
            assert_eq!(f!(dbus_message_iter_init)(msg.as_ptr(), self.as_ptr()), 1);
        }
    }

    pub fn as_ptr(&mut self) -> &mut ffi::DBusMessageIter {
        &mut self.0
    }

    pub fn next(&mut self) -> bool {
        unsafe { f!(dbus_message_iter_next)(self.as_ptr()) != 0 }
    }

    pub fn get_arg_type(&mut self) -> c_int {
        unsafe { f!(dbus_message_iter_get_arg_type)(self.as_ptr()) }
    }

    pub unsafe fn get_basic(&mut self, out: *mut c_void) {
        unsafe {
            f!(dbus_message_iter_get_basic)(self.as_ptr(), out);
        }
    }

    pub fn get_u32(&mut self) -> Option<u32> {
        if self.get_arg_type() != ffi::DBUS_TYPE_UINT32 {
            return None;
        }
        let mut out = u32::MAX;
        unsafe {
            self.get_basic(&mut out as *mut u32 as *mut _);
        }
        Some(out)
    }

    pub fn get_object_path(&mut self) -> Option<CString> {
        if self.get_arg_type() != ffi::DBUS_TYPE_OBJECT_PATH {
            return None;
        }
        unsafe { self.get_basic_str() }
    }

    pub fn get_string(&mut self) -> Option<CString> {
        if self.get_arg_type() != ffi::DBUS_TYPE_STRING {
            return None;
        }
        unsafe { self.get_basic_str() }
    }

    pub fn get_string_array(&mut self) -> Vec<CString> {
        let mut out = Vec::new();

        if self.get_arg_type() == ffi::DBUS_TYPE_ARRAY {
            let mut array_iter = self.iter_recurse();
            while array_iter.get_arg_type() == ffi::DBUS_TYPE_STRING {
                if let Some(item) = array_iter.get_string() {
                    out.push(item);
                } else {
                    log::error!("Wrong type in a string array")
                }
                array_iter.next();
            }
        }

        out
    }

    unsafe fn get_basic_str(&mut self) -> Option<CString> {
        unsafe {
            let mut out: *const c_char = std::ptr::null_mut();
            self.get_basic(&mut out as *mut *const c_char as *mut _);

            if out.is_null() {
                None
            } else {
                Some(CStr::from_ptr(out).to_owned())
            }
        }
    }

    pub fn init_append(msg: &'a mut Message) -> Self {
        let mut new = Self::new();
        unsafe { f!(dbus_message_iter_init_append)(msg.as_ptr(), new.as_ptr()) };
        new
    }

    pub fn append_basic(&mut self, ty: c_int, value: *const c_void) {
        unsafe {
            assert_eq!(
                f!(dbus_message_iter_append_basic)(self.as_ptr(), ty, value,),
                1
            );
        }
    }

    pub fn append_string(&mut self, value: &CStr) {
        self.append_basic(
            ffi::DBUS_TYPE_STRING,
            &value.as_ptr() as *const _ as *const _,
        );
    }

    pub fn append_byte(&mut self, value: u8) {
        self.append_basic(ffi::DBUS_TYPE_BYTE, &value as *const u8 as *const _);
    }

    pub fn append_u32(&mut self, value: u32) {
        self.append_basic(ffi::DBUS_TYPE_UINT32, &value as *const _ as *const _);
    }

    pub fn append_bool(&mut self, value: bool) {
        let value: u32 = if value { 1 } else { 0 };
        self.append_basic(ffi::DBUS_TYPE_BOOLEAN, &value as *const _ as *const _);
    }

    pub fn open_container(&mut self, ty: c_int, signature: Option<&CStr>) -> Self {
        unsafe {
            let mut out = Self::new();
            assert_eq!(
                f!(dbus_message_iter_open_container)(
                    self.as_ptr(),
                    ty,
                    signature.map(|s| s.as_ptr()).unwrap_or_default(),
                    out.as_ptr(),
                ),
                1
            );
            out
        }
    }

    pub fn close_container(&mut self, mut sub: Self) {
        unsafe {
            assert_eq!(
                f!(dbus_message_iter_close_container)(self.as_ptr(), sub.as_ptr()),
                1
            );
        }
    }

    pub fn with_container<T>(
        &mut self,
        ty: c_int,
        signature: Option<&CStr>,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let mut container = self.open_container(ty, signature);
        let res = f(&mut container);
        self.close_container(container);
        res
    }

    pub fn with_dict_entry<T>(
        &mut self,
        key: &CStr,
        variant_signature: &CStr,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        self.with_container(ffi::DBUS_TYPE_DICT_ENTRY, None, |entry| {
            entry.append_string(key);
            entry.with_container(ffi::DBUS_TYPE_VARIANT, Some(variant_signature), |variant| {
                f(variant)
            })
        })
    }
}
