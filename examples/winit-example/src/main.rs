use winit::{
    event::{self, WindowEvent},
    event_loop::EventLoopBuilder,
    keyboard::{KeyCode, PhysicalKey},
};

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

fn main() {
    let event_loop = EventLoopBuilder::<String>::with_user_event()
        .build()
        .expect("Failed");
    let builder = winit::window::WindowBuilder::new();

    let window = builder.build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;

        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                if let Some(canvas) = window.canvas() {
                    body.append_child(&canvas.into())
                        .ok()
                } else {
                    None
                }
            })
            .expect("couldn't append canvas to document body");
    }
    let event_loop_proxy = event_loop.create_proxy();
    let executor = Executor::new();

    event_loop
        .run(move |event, target| match event {
            event::Event::UserEvent(name) => {
                #[cfg(target_arch = "wasm32")]
                alert(&name);
                #[cfg(not(target_arch = "wasm32"))]
                println!("{}", name);
            }
            event::Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested { .. } => target.exit(),
                WindowEvent::KeyboardInput {
                    event:
                        event::KeyEvent {
                            state: event::ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::KeyS),
                            ..
                        },
                    ..
                } => {
                    let dialog = rfd::AsyncFileDialog::new()
                        .add_filter("midi", &["mid", "midi"])
                        .add_filter("rust", &["rs", "toml"])
                        .set_parent(&window)
                        .save_file();

                    let event_loop_proxy = event_loop_proxy.clone();
                    executor.execut(async move {
                        let file = dialog.await;

                        let file = if let Some(file) = file {
                            file.write(b"Hi! This is a test file").await.unwrap();
                            Some(file)
                        } else {
                            None
                        };

                        event_loop_proxy
                            .send_event(format!("saved file name: {:#?}", file))
                            .ok();
                    });
                }
                WindowEvent::KeyboardInput {
                    event:
                        event::KeyEvent {
                            state: event::ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::KeyF),
                            ..
                        },
                    ..
                } => {
                    let dialog = rfd::AsyncFileDialog::new()
                        .add_filter("midi", &["mid", "midi"])
                        .add_filter("rust", &["rs", "toml"])
                        .set_parent(&window)
                        .pick_file();

                    let event_loop_proxy = event_loop_proxy.clone();
                    executor.execut(async move {
                        let files = dialog.await;

                        // let names: Vec<String> = files.into_iter().map(|f| f.file_name()).collect();
                        let names = files;

                        event_loop_proxy.send_event(format!("{:#?}", names)).ok();
                    });
                }
                WindowEvent::DroppedFile(file_path) => {
                    let dialog = rfd::AsyncMessageDialog::new()
                        .set_title("File dropped")
                        .set_description(format!("file path was: {:#?}", file_path))
                        .set_buttons(rfd::MessageButtons::YesNo)
                        .set_parent(&window)
                        .show();

                    let event_loop_proxy = event_loop_proxy.clone();
                    executor.execut(async move {
                        let val = dialog.await;
                        event_loop_proxy.send_event(format!("Msg: {}", val)).ok();
                    });
                }
                WindowEvent::KeyboardInput {
                    event:
                        event::KeyEvent {
                            state: event::ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::KeyM),
                            ..
                        },
                    ..
                } => {
                    let dialog = rfd::AsyncMessageDialog::new()
                        .set_title("Msg!")
                        .set_description("Description!")
                        .set_buttons(rfd::MessageButtons::YesNo)
                        .set_parent(&window)
                        .show();

                    let event_loop_proxy = event_loop_proxy.clone();
                    executor.execut(async move {
                        let val = dialog.await;
                        event_loop_proxy.send_event(format!("Msg: {}", val)).ok();
                    });
                }
                _ => {}
            },
            _ => {}
        })
        .unwrap();
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
