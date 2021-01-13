use winit::{
    event::{self, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

fn main() {
    let event_loop = EventLoop::new();
    let mut builder = winit::window::WindowBuilder::new();

    let window = builder.build(&event_loop).unwrap();

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
                // let task = rfd::macos::callback_test(|| {
                //     println!("async done");
                // });

                let task = rfd::macos::async_test();
                //
                std::thread::spawn(move || {
                    futures::executor::block_on(async {
                        task.await;
                    });
                });
            }
            _ => {}
        },
        _ => {}
    });
}
