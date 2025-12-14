use std::{
    cell::UnsafeCell,
    ffi::{c_char, c_void, CStr, CString},
    fmt, ptr,
};

use super::super::super::window_identifier::WindowIdentifierType;
use super::ffi;

use ffi::{wl_display, wl_proxy, LibWayland, SyncWrapper, WlEventQueueHandle, WlProxyHandle};

impl LibWayland {
    fn wl_display_get_registry(
        &self,
        display: *mut wl_display,
        queue: &WlEventQueueHandle,
    ) -> WlProxyHandle {
        let wrapped_display = (self.wl_proxy_create_wrapper)(display.cast());
        assert!(!wrapped_display.is_null());

        (self.wl_proxy_set_queue)(wrapped_display, queue.as_ptr());

        let wl_registry = (self.wl_proxy_marshal_constructor)(
            wrapped_display,
            ffi::display::WL_DISPLAY_GET_REGISTRY,
            &ffi::registry::INTERFACE,
        );
        assert!(!wl_registry.is_null());

        (self.wl_proxy_wrapper_destroy)(wrapped_display);

        WlProxyHandle::new(wl_registry, self.clone())
    }

    fn wl_registry_bind_exporter(
        &self,
        registry: WlProxyHandle,
        xdg_exporter_name: u32,
    ) -> WlProxyHandle {
        let registry = registry.as_ptr();
        let xdg_exporter = (self.wl_proxy_marshal_constructor)(
            registry,
            ffi::registry::WL_REGISTRY_BIND,
            &ffi::xdg_exporter::INTERFACE,
            xdg_exporter_name,
            ffi::xdg_exporter::INTERFACE.name,
            1,
        );
        assert!(!xdg_exporter.is_null());

        WlProxyHandle::new(xdg_exporter, self.clone())
    }

    fn xdg_exporter_export_toplevel(
        &self,
        xdg_exporter: WlProxyHandle,
        wl_surface: *mut wl_proxy,
    ) -> WlProxyHandle {
        let xdg_exporter = xdg_exporter.as_ptr();
        let xdg_exported = (self.wl_proxy_marshal_constructor_versioned)(
            xdg_exporter,
            ffi::xdg_exporter::ZXDG_EXPORTER_V2_EXPORT_TOPLEVEL,
            &ffi::xdg_exported::INTERFACE,
            1,
            0,
            wl_surface,
        );
        assert!(!xdg_exported.is_null());
        WlProxyHandle::new(xdg_exported, self.clone())
    }

    fn roundtrip_with_listener<const N: usize, U>(
        &self,
        display: *mut wl_display,
        queue: &WlEventQueueHandle,
        proxy: &WlProxyHandle,
        cb: &SyncWrapper<UnsafeCell<[*const c_void; N]>>,
        user_data: &mut U,
    ) {
        (self.wl_proxy_add_listener)(
            proxy.as_ptr(),
            cb.0.get().cast(),
            user_data as *mut _ as *mut _,
        );
        (self.wl_display_roundtrip_queue)(display, queue.as_ptr());
        (self.wl_proxy_set_user_data)(proxy.as_ptr(), ptr::null_mut::<c_void>());
    }

    fn fetch_xdg_exporter_from_registry(
        &self,
        display: *mut wl_display,
        queue: &WlEventQueueHandle,
    ) -> Option<WlProxyHandle> {
        let registry = self.wl_display_get_registry(display.cast(), queue);

        let xdg_exporter_name = {
            #[derive(Default)]
            struct UserData(Option<u32>);

            extern "C" fn registry_global(
                data: *mut c_void,
                _registry: *mut wl_proxy,
                name: u32,
                interface: *const c_char,
                _version: u32,
            ) {
                unsafe {
                    let data: *mut UserData = data.cast();
                    if data.is_null() {
                        return;
                    }

                    if CStr::from_ptr(interface) == c"zxdg_exporter_v2" {
                        (*data).0 = Some(name);
                    }
                }
            }

            extern "C" fn registry_global_remove(
                _data: *mut c_void,
                _registry: *mut wl_proxy,
                _name: u32,
            ) {
            }

            static REGISTRY_IMPL: SyncWrapper<UnsafeCell<[*const c_void; 2]>> =
                SyncWrapper(UnsafeCell::new([
                    registry_global as *const c_void,
                    registry_global_remove as *const c_void,
                ]));

            let mut xdg_exported = UserData::default();

            self.roundtrip_with_listener(
                display,
                queue,
                &registry,
                &REGISTRY_IMPL,
                &mut xdg_exported,
            );

            xdg_exported.0
        };

        Some(self.wl_registry_bind_exporter(registry, xdg_exporter_name?))
    }

    fn fetch_xdg_exported_handle(
        &self,
        xdg_exported: &WlProxyHandle,
        display: *mut wl_display,
        queue: &WlEventQueueHandle,
    ) -> Option<CString> {
        #[derive(Default)]
        struct UserData(Option<CString>);

        extern "C" fn xdg_exported_handle(
            data: *mut c_void,
            _exported: *mut c_void,
            handle: *const c_char,
        ) {
            unsafe {
                let data: *mut UserData = data.cast();
                if data.is_null() {
                    return;
                }
                let handle = CStr::from_ptr(handle).to_owned();
                (*data).0 = Some(handle);
            }
        }

        static XDG_EXPORTED_IMPL: SyncWrapper<UnsafeCell<[*const c_void; 1]>> =
            SyncWrapper(UnsafeCell::new([xdg_exported_handle as *const c_void]));

        let mut user_data = UserData::default();

        self.roundtrip_with_listener(
            display,
            queue,
            xdg_exported,
            &XDG_EXPORTED_IMPL,
            &mut user_data,
        );

        user_data.0
    }
}

#[derive(Debug)]
pub struct XdgForeignHandle {
    pub token: WindowIdentifierType,
    _xdg_exported: WlProxyHandle,
    _queue: WlEventQueueHandle,
    _lib: LibWayland,
}

impl fmt::Display for XdgForeignHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.token.fmt(f)
    }
}

impl XdgForeignHandle {}

pub fn run(display_ptr: *mut c_void, surface_ptr: *mut c_void) -> Option<XdgForeignHandle> {
    let display = display_ptr as *mut wl_display;
    let wl_surface = surface_ptr as *mut wl_proxy;

    assert!(!display.is_null());
    assert!(!wl_surface.is_null());

    let lib = LibWayland::new()?;

    let queue = WlEventQueueHandle::new(&lib, display);

    let xdg_exporter = lib.fetch_xdg_exporter_from_registry(display, &queue)?;
    let xdg_exported = lib.xdg_exporter_export_toplevel(xdg_exporter, wl_surface);

    let token = lib.fetch_xdg_exported_handle(&xdg_exported, display, &queue)?;
    let token = token.to_str().ok()?.to_string();

    Some(XdgForeignHandle {
        token: WindowIdentifierType::Wayland(token),
        _xdg_exported: xdg_exported,
        _queue: queue,
        _lib: lib,
    })
}
