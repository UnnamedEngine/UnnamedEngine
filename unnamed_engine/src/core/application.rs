use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::{ControlFlow, EventLoop}, window::Window};

use super::engine::Engine;

pub struct Application {
    window: Option<Window>,
    engine: Engine,
    title: String,
}

impl Application {
    pub fn new(title: String) -> Self {
        let engine = Engine::default();

        let window = None;

        Self {
            window,
            engine,
            title,
        }
    }

    pub fn run(&mut self) {
        // Start the engine
        self.engine.run();

        // Creates the event loop and sets it to `ControlFlow::Poll`, that way
        // we continously run the event loop
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);

        match event_loop.run_app(self) {
            Ok(_) => {},
            Err(err) => {
                log::error!("Failed to run event_loop: {}", err.to_string());
            },
        }
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        // Sets initial attributes for our window
        let mut window_attributes = Window::default_attributes();
        window_attributes.title = self.title.clone();

        self.window = Some(event_loop.create_window(window_attributes).unwrap());
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                self.engine.shutdown();
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                // Redraw the application
                //
                // It's preferable for applications that do not render
                // continously to render in this event rather than in
                // AboutToWait, since rendering in here allows the program to
                // gracefully handle redraws requested by the OS.

                // Draw.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need
                // to redraw in applications which do not always need to.
                // Applications that redraw continously can render here instead.
                self.window.as_ref().unwrap().request_redraw();
            },
            _ => (),
        }
    }
}
