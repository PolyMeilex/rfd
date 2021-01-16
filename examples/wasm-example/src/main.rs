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
    let executor = Executor::new();

    event_loop.run(move |event, _, control_flow| match event {
        event::Event::UserEvent(name) => {
            #[cfg(target_arch = "wasm32")]
            alert(&name);
            #[cfg(not(target_arch = "wasm32"))]
            println!("{}", name);
        }
        event::Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested { .. } => *control_flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput {
                input:
                    event::KeyboardInput {
                        state: event::ElementState::Pressed,
                        virtual_keycode: Some(winit::event::VirtualKeyCode::D),
                        ..
                    },
                ..
            } => {
                let mut dialog = rfd::AsyncFileDialog::new();

                let event_loop_proxy = event_loop_proxy.clone();
                executor.execut(async move {
                    let files = dialog.pick_files().await;

                    // let names: Vec<String> = files.into_iter().map(|f| f.file_name()).collect();

                    let names = files;

                    event_loop_proxy.send_event(format!("{:#?}", names)).ok();
                });
            }
            _ => {}
        },
        _ => {}
    });
}

use std::future::Future;

struct Executor {
    #[cfg(not(target_arch = "wasm32"))]
    pool: futures::executor::ThreadPool,
}

impl Executor {
    fn new() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            pool: futures::executor::ThreadPool::new().unwrap(),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn execut<F: Future<Output = ()> + Send + 'static>(&self, f: F) {
        self.pool.spawn_ok(f);
    }
    #[cfg(target_arch = "wasm32")]
    fn execut<F: Future<Output = ()> + 'static>(&self, f: F) {
        wasm_bindgen_futures::spawn_local(f);
    }
}
