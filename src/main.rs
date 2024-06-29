use winit::{
    event::*, event_loop::EventLoop, window::WindowBuilder, keyboard::{KeyCode, PhysicalKey},
};

use::ultimate_voxel_engine::*;


async fn run() {
    // env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = state::State::new(&window).await;

    let _ = event_loop.run(move |event, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                ..
                            },
                        ..
                    } => control_flow.exit(),
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }

                    WindowEvent::RedrawRequested => {
                        state.update();
                        match state.render() {
                            Ok(_) => {},
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }

                    _ => {}
                }
            }

            Event::AboutToWait => {
                state.window().request_redraw();
            }

            _ => {}
        }
    });

}

fn main() {
    env_logger::init();
    pollster::block_on(run());
}
