////////////////////////////////////////////////////////////////////////////////
//
//                ██╗   ██╗███╗   ██╗███████╗███╗   ██╗
//                ██║   ██║████╗  ██║██╔════╝████╗  ██║
//                ██║   ██║██╔██╗ ██║█████╗  ██╔██╗ ██║
//                ██║   ██║██║╚██╗██║██╔══╝  ██║╚██╗██║
//                ╚██████╔╝██║ ╚████║███████╗██║ ╚████║
//                 ╚═════╝ ╚═╝  ╚═══╝╚══════╝╚═╝  ╚═══╝ LIB
//
////////////////////////////////////////////////////////////////////////////////
// ? This file contains the main Engine struct, its where the abstract methods
// ? are called and the general flow of execution is defined.
use super::state::State;

use crate::event::event::Event;

use env_logger::Env;
use winit::{
    event::Event as WinitEvent,
    event::WindowEvent as WinitWindowEvent,
    event_loop::EventLoop,
    window::WindowBuilder,
    keyboard::{PhysicalKey::{Code, self}, KeyCode}
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
    // This method is the only one that should be called from the application
    pub fn start(
        &mut self,
        start_f: impl FnOnce(&mut Engine),
        update_f: impl FnMut(&mut Engine),
        render_f: impl FnMut(&mut Engine),
        event_f: impl FnMut(&mut Engine, Event)) {
            // Initializes the logger
            let env = Env::default()
                .filter_or("MY_LOG_LEVEL", "info")
                .write_style_or("MY_LOG_STYLE", "always");
            env_logger::init_from_env(env);

            start_f(self);

            self.running = true;
            let result = tokio::runtime::Runtime::new().unwrap().block_on(self.run(update_f, render_f, event_f));
    }

    // Stops the engine
    // Should be called for a graceful shutdown of the engine
    pub fn stop(&mut self) {
        // Running async so we must be sure it gets called only once
        if self.running {
            log::info!("Graceful shutdown complete, see you again :D");
            self.running = false;
        }
    }

    // Starts running the engine
    async fn run(
        &mut self,
        mut update_f: impl FnMut(&mut Engine),
        mut render_f: impl FnMut(&mut Engine),
        mut event_f: impl FnMut(&mut Engine, Event)) {
            let event_loop = EventLoop::new().unwrap();
            let window = WindowBuilder::new().build(&event_loop).unwrap();
            window.set_title(&self.title);

            let mut state = State::new(window).await;

            let my_window_id = state.window().id();

            event_loop.run(move |event, elwt| {
                match event {
                    WinitEvent::WindowEvent {
                        ref event,
                        window_id,
                    } if window_id == my_window_id => {
                        match &event {
                            // Close event
                            WinitWindowEvent::CloseRequested => elwt.exit(),
                            // Resize event
                            WinitWindowEvent::Resized(physical_size) => {
                                state.resize(*physical_size);
                            },
                            // Scale changed event
                            WinitWindowEvent::ScaleFactorChanged { .. } => {
                                state.resize(state.window().inner_size());
                            },
                            // Input event
                            WinitWindowEvent::KeyboardInput { event, .. } => {
                                // Handle the inner inputs, if correctly handled, request a new frame
                                if state.input(event) {
                                    state.window().request_redraw();
                                    return;
                                }

                                match event.physical_key {
                                    PhysicalKey::Code(key_code) => {
                                        event_f(self, Event::Keyboard {
                                            key: key_code,
                                            is_pressed: event.state.is_pressed()
                                        });
                                    },
                                    PhysicalKey::Unidentified(_) => {}
                                }
                            },
                            // New frame requested
                            // Just executes if the window id is the same as the one handled by the engine
                            //
                            // This is where the main loop runs on
                            WinitWindowEvent::RedrawRequested if window_id == state.window().id() => {
                                // Should we stop?
                                if !self.running {
                                    elwt.exit();
                                }

                                // Update the state
                                state.update();

                                // Update the upper application
                                update_f(self);

                                // Render the state
                                // Also handles errors
                                match state.render() {
                                    Ok(_) => {}
                                    // Reconfigure the surface if lost
                                    Err(wgpu::SurfaceError::Lost) => state.resize(*state.size()),
                                    // The system is out of memory, we should probably quit
                                    Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                                    Err(e) => eprintln!("{:?}", e),
                                }

                                // Render the upper application
                                render_f(self);
                            },
                            _ => {}
                        }
                    },
                    // Sometimes this will be called, and when it gets called it should just request a new frame
                    WinitEvent::AboutToWait => state.window().request_redraw(),
                    _ => {}
                }
            }).unwrap();
    }
}
