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
    let event_loop = EventLoop::<String>::with_user_event();
    let mut builder = winit::window::WindowBuilder::new();

    let window = builder.build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;

        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
    }
    let event_loop_proxy = event_loop.create_proxy();

    event_loop.run(move |event, _, control_flow| match event {
        event::Event::UserEvent(name) => {
            #[cfg(target_arch = "wasm32")]
            alert(&name);
            #[cfg(not(target_arch = "wasm32"))]
            println!("{}", name);
        }
        event::Event::WindowEvent { event, .. } => match event {
            WindowEvent::KeyboardInput {
                input:
                    event::KeyboardInput {
                        state: event::ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                #[cfg(target_arch = "wasm32")]
                let mut dialog = rfd::wasm::FileDialog::new();
                #[cfg(not(target_arch = "wasm32"))]
                let mut dialog = rfd::AsyncFileDialog::new();

                //
                #[cfg(target_arch = "wasm32")]
                {
                    let event_loop_proxy = event_loop_proxy.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        let files = dialog.pick_files().await;

                        for file in files {
                            let name = file.file_name();

                            // let file = file.read().await;
                            event_loop_proxy.send_event(name).ok();
                        }
                    });
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let files = dialog.pick_files().unwrap();

                    // let files = files.into_iter().map(|f|

                    for file in files {
                        let file = rfd::FileHandle::wrap(file);
                        let name = file.file_name();

                        event_loop_proxy.send_event(name).ok();
                    }
                }
            }
            _ => {}
        },
        _ => {}
    });
}
