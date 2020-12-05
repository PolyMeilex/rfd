use winit::*;

fn main() {
    let el = event_loop::EventLoop::new();
    let win = window::WindowBuilder::new().build(&el).unwrap();

    el.run(move |event, _, control_flow| {
        //*control_flow = event_loop::ControlFlow::Wait;
        match event {
            event::Event::WindowEvent {
                event: event::WindowEvent::MouseInput { .. },
                ..
            } => {
                println!("event");

                rfd::open();
            }
            _ => {}
        }
    })
}
