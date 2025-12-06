use std::fmt;

use wayland_backend::sys::client::Backend;
use wayland_client::{
    protocol::{wl_registry, wl_surface::WlSurface},
    Proxy, QueueHandle,
};
use wayland_protocols::xdg::foreign::zv2::client::{
    zxdg_exported_v2::{self, ZxdgExportedV2},
    zxdg_exporter_v2::ZxdgExporterV2,
};

use super::WindowIdentifierType;

// Supported versions.
const ZXDG_EXPORTER_V2: u32 = 1;

#[derive(Debug)]
pub struct WaylandWindowIdentifier {
    exported: ZxdgExportedV2,
    type_: WindowIdentifierType,
}

impl WaylandWindowIdentifier {
    pub async unsafe fn from_raw(
        surface_ptr: *mut std::ffi::c_void,
        display_ptr: *mut std::ffi::c_void,
    ) -> Option<Self> {
        if surface_ptr.is_null() || display_ptr.is_null() {
            return None;
        }

        let backend = Backend::from_foreign_display(display_ptr as *mut _);
        let conn = wayland_client::Connection::from_backend(backend);
        let obj_id = wayland_backend::sys::client::ObjectId::from_ptr(
            WlSurface::interface(),
            surface_ptr as *mut _,
        )
        .ok()?;

        let surface = WlSurface::from_id(&conn, obj_id).ok()?;

        Self::new_inner(conn, &surface).await
    }

    async fn new_inner(conn: wayland_client::Connection, surface: &WlSurface) -> Option<Self> {
        let (sender, receiver) = crate::oneshot::channel::<Option<WaylandWindowIdentifier>>();

        let surface = surface.clone();
        std::thread::spawn(move || match wayland_export_handle(conn, &surface) {
            Some(window_handle) => sender.send(Some(window_handle)).unwrap(),
            None => {
                sender.send(None).unwrap();
            }
        });

        receiver.await.unwrap()
    }
}

impl fmt::Display for WaylandWindowIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.type_.fmt(f)
    }
}

impl Drop for WaylandWindowIdentifier {
    fn drop(&mut self) {
        self.exported.destroy();
    }
}

#[derive(Default, Debug)]
struct State {
    handle: String,
    exporter: Option<ZxdgExporterV2>,
}

impl wayland_client::Dispatch<ZxdgExportedV2, ()> for State {
    fn event(
        state: &mut Self,
        _proxy: &ZxdgExportedV2,
        event: <ZxdgExportedV2 as Proxy>::Event,
        _data: &(),
        _connhandle: &wayland_client::Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        if let zxdg_exported_v2::Event::Handle { handle } = event {
            state.handle = handle;
        }
    }
}

impl wayland_client::Dispatch<ZxdgExporterV2, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &ZxdgExporterV2,
        _event: <ZxdgExporterV2 as Proxy>::Event,
        _data: &(),
        _connhandle: &wayland_client::Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl wayland_client::Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &wayland_client::Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            if interface.as_str() == "zxdg_exporter_v2" {
                let exporter = registry.bind::<ZxdgExporterV2, (), State>(
                    name,
                    version.min(ZXDG_EXPORTER_V2),
                    qhandle,
                    (),
                );
                state.exporter = Some(exporter);
            }
        }
    }
}

fn wayland_export_handle(
    conn: wayland_client::Connection,
    surface: &WlSurface,
) -> Option<WaylandWindowIdentifier> {
    let display = conn.display();
    let mut event_queue = conn.new_event_queue();
    let qhandle = event_queue.handle();
    let mut state = State::default();
    display.get_registry(&qhandle, ());
    event_queue.roundtrip(&mut state).ok()?;

    let exported = match state.exporter.take() {
        Some(exporter) => {
            let exp = exporter.export_toplevel(surface, &qhandle, ());
            event_queue.roundtrip(&mut state).ok()?;
            exporter.destroy();

            Some(exp)
        }
        None => None,
    };

    if let Some(exported) = exported {
        Some(WaylandWindowIdentifier {
            exported,
            type_: WindowIdentifierType::Wayland(state.handle),
        })
    } else {
        log::warn!("Wayland compositor did not reposne with xdg foreign token");
        None
    }
}
