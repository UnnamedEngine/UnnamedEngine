use super::state::State;

use winit::{
    event::*,
    event_loop::EventLoop,
    window::WindowBuilder,
    keyboard::KeyCode,
    keyboard::PhysicalKey::Code
};

pub struct Engine {
    running: bool,
    title: String,
}

impl Engine {
    pub fn new(title: String) -> Self {
        Engine {
            running: false,
            title: title,
        }
    }

    // Starts the engine
    // This method is the only that should be called from the application
    pub fn start(&mut self, start_f: impl FnOnce(), update_f: impl FnMut(), render_f: impl FnMut()) {
        env_logger::init();

        start_f();

        self.running = true;
        let result = tokio::runtime::Runtime::new().unwrap().block_on(self.run(update_f, render_f));
    }

    // // Stops the engine
    // // Should be called for a graceful shutdown of the engine
    // fn stop(&mut self, stop_f: impl FnOnce()) {
    //     stop_f();
    //     self.running = false;
    // }

    // Starts running the engine
    async fn run(&self, mut update_f: impl FnMut(), mut render_f: impl FnMut()) {
        let event_loop = EventLoop::new().unwrap();
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        let mut state = State::new(window).await;

        let my_window_id = state.window().id();

        event_loop.run(move |event, elwt| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == my_window_id => {
                    match &event {
                        WindowEvent::CloseRequested => elwt.exit(),
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        },
                        WindowEvent::ScaleFactorChanged { .. } => {
                            state.resize(state.window().inner_size());
                        },
                        WindowEvent::KeyboardInput { event, .. } =>{
                            if state.input(event) {
                                state.window().request_redraw();
                                return;
                            }
                            if event.state.is_pressed() {
                                match event.physical_key {
                                    Code(KeyCode::Escape) => {
                                        elwt.exit();
                                    },
                                    _ => {}
                                }
                            }
                        },
                        WindowEvent::RedrawRequested if window_id == state.window().id() => {
                            state.update();
                            match state.render() {
                                Ok(_) => {}
                                // Reconfigure the surface if lost
                                Err(wgpu::SurfaceError::Lost) => state.resize(*state.size()),
                                // The system is out of memory, we should probably quit
                                Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                                // All other errors (Outdated, Timeout) should be resolved by the next frame
                                Err(e) => eprintln!("{:?}", e),
                            }

                        },
                        _ => {}
                    }
                },
                Event::AboutToWait => state.window().request_redraw(),
                _ => {}
            }
        }).unwrap();
    }

    // Logical update that runs at each iteration of the engine
    fn update(&self, mut update_f: impl FnMut()) {
        update_f();
    }

    // Rendering that runs at each iteration of the engine
    fn render(&self, mut render_f: impl FnMut()) {
        render_f();
    }
}
