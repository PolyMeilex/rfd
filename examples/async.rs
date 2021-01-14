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
                // Spawn dialog on main thread
                let task = rfd::macos::pick_file_async(&Default::default());
                // Await somewhere else
                std::thread::spawn(move || {
                    futures::executor::block_on(async {
                        let files = task.await;
                        println!("Hell yeah it's async done!!");
                    });
                });
            }
            _ => {}
        },
        _ => {}
    });
}
