use winit::{
    event::{self, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

fn main() {
    let event_loop = EventLoop::new();
    let mut builder = winit::window::WindowBuilder::new();

    let window = builder.build(&event_loop).unwrap();

    use winit::platform::web::WindowExtWebSys;

    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| doc.body())
        .and_then(|body| {
            body.append_child(&web_sys::Element::from(window.canvas()))
                .ok()
        })
        .expect("couldn't append canvas to document body");

    let window = web_sys::window().expect("Window not found");
    let document = window.document().expect("Document not found");
    let body = document.body().expect("document should have a body");

    event_loop.run(move |event, _, control_flow| match event {
        event::Event::WindowEvent { event, .. } => match event {
            WindowEvent::KeyboardInput {
                input:
                    event::KeyboardInput {
                        state: event::ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                let body = body.clone();
                let mut dialog = rfd::wasm::Dialog::new(&document);
                wasm_bindgen_futures::spawn_local(async move {
                    let files = dialog.open(&body).await;

                    for file in files {
                        let file = file.read().await;

                        alert(&format!("{:?}", file));
                    }
                });
            }
            _ => {}
        },
        _ => {}
    });
}
