//! ## Engine
//!
//! Defines the engine struct.
use super::state::State;

use crate::{event::event::Event, input::{input_manager::InputManager, keyboard::Keyboard}};

use env_logger::Env;
use winit::{
  dpi::PhysicalSize, event::{Event as WinitEvent, WindowEvent as WinitWindowEvent}, event_loop::EventLoop, keyboard::PhysicalKey, window::WindowBuilder
};

/// ## Engine
///
/// Main struct of UnnamedEngine, its the core wrapper that contains the upper abstract methods and
/// defines the general flow of execution.
pub struct Engine {
  running: bool,
  title: String,
  pub input_manager: InputManager,
}

impl Engine {
  pub fn new(title: String) -> Self {
    let input_manager = InputManager::new();

    Engine {
      running: false,
      title,
      input_manager,
    }
  }

  /// Exposes the keyboard
  pub fn keyboard(&mut self) -> &Keyboard {
    &self.input_manager.keyboard
  }

  /// Starts the engine
  pub fn start(
    &mut self,
    start_f: impl FnOnce(&mut Engine),
    update_f: impl FnMut(&mut Engine),
    render_f: impl FnMut(&mut Engine),
    event_f: impl FnMut(&mut Engine, &Event),
    ) {
      // Initializes the logger
      let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "info")
        .write_style_or("MY_LOG_STYLE", "always");
      env_logger::init_from_env(env);

      start_f(self);

      self.running = true;
      tokio::runtime::Runtime::new().unwrap().block_on(self.run(update_f, render_f, event_f));
    }

    /// Stops the engine
    ///
    /// Should be called for a graceful shutdown of the engine
    pub fn stop(&mut self) {
      // Running async so we must be sure it gets called only once
      if self.running {
        log::info!("Graceful shutdown complete, see you again :D");
        self.running = false;
      }
    }

  /// Starts the engine
  async fn run(
    &mut self,
    mut update_f: impl FnMut(&mut Engine),
    mut render_f: impl FnMut(&mut Engine),
    mut event_f: impl FnMut(&mut Engine, &Event))
    {
      let event_loop = EventLoop::new()
        .expect("Could not create event loop");
      let window = WindowBuilder::new().build(&event_loop)
        .expect("Could not create window");
      window.set_title(&self.title);

      let mut engine_state = State::new((window.into(), wgpu::Color::BLACK)).await;

      let my_window_id = engine_state.renderer.viewport.desc.window.id();

      event_loop.run(move |event, elwt| {
        match event {
          WinitEvent::WindowEvent {
            ref event,
            window_id,
          } if window_id == my_window_id => {
            match &event {
              // Close event
              WinitWindowEvent::CloseRequested => {
                // Create and dispatch shutdown event
                self.on_event(&mut engine_state, &Event::Shutdown, &mut event_f);
              },
              // Resize event
              WinitWindowEvent::Resized(physical_size) => {
                // Create and dispatch resize event
                // TODO make it work as an event
                self.on_event(
                  &mut engine_state,
                  &Event::Resize {
                    width: physical_size.width,
                    height: physical_size.height,
                  },
                  &mut event_f,
                );
              },
              // Scale changed event
              WinitWindowEvent::ScaleFactorChanged { .. } => {
                // Create and dispatch resize event
                let size = engine_state.renderer.viewport.desc.window.inner_size();
                self.on_event(
                  &mut engine_state,
                  &Event::Resize {
                    width: size.width,
                    height: size.height,
                  },
                  &mut event_f,
                );
              },
              // Keyboard input event
              WinitWindowEvent::KeyboardInput { event, .. } => {
                match event.physical_key {
                  PhysicalKey::Code(key_code) => {
                    // Send a keyboard input event
                    self.on_event(
                      &mut engine_state,
                      &Event::KeyboardInput {
                        key: key_code,
                        is_pressed: event.state.is_pressed()
                      },
                      &mut event_f,
                    );
                  },
                  PhysicalKey::Unidentified(_) => {}
                }
              },
              // Mouse moved
              WinitWindowEvent::CursorMoved {
                device_id: _,
                position } => {
                  // Send a mouse moved event
                  self.on_event(
                    &mut engine_state,
                    &Event::MouseMoved {
                      x: position.x as u32,
                      y: position.y as u32,
                    },
                    &mut event_f,
                  );
              },
              // Mouse clicked
              WinitWindowEvent::MouseInput {
                device_id: _,
                state,
                button } => {
                // Send a mouse input event
                self.on_event(
                  &mut engine_state,
                  &Event::MouseInput {
                    button: *button,
                    is_pressed: state.is_pressed(),
                  },
                  &mut event_f,
                );
              }
              // New frame requested
              // Only executes if the window id is the same as the one handled by the engine
              //
              // This is where the main loop runs on
              WinitWindowEvent::MouseWheel {
                delta,
                ..
              } => {
                match delta {
                  winit::event::MouseScrollDelta::LineDelta(x, y) => {
                    self.on_event(
                      &mut engine_state,
                      &Event::MouseScroll {
                        delta: (*x, *y)
                      },
                      &mut event_f,
                    );
                  },
                  _ => {},
                }
              },
              WinitWindowEvent::RedrawRequested if window_id == engine_state.renderer.viewport.desc.window.id() => {
                // Should we stop?
                if !self.running {
                  elwt.exit();
                }

                // Update the state
                engine_state.update();

                // Update the upper application
                update_f(self);

                // Render the state
                engine_state.render().expect("Failed to render");

                // Render the upper application
                render_f(self);
              },
              _ => {}
            }
            engine_state.renderer.egui.handle_input(&mut engine_state.renderer.viewport.desc.window, &event);
          },
          // Sometimes this will be called, and when it gets called it should just request a new frame
          WinitEvent::AboutToWait => engine_state.renderer.request_redraw(),
          _ => {}
        }
      }).expect("Failed to run a loop");
  }

  /// Gets called from the event loop and replicate the events to the applications
  fn on_event(
    &mut self,
    engine_state: &mut State,
    event: &Event,
    event_f: &mut impl FnMut(&mut Engine, &Event)) {
    // First try to handle it internally, if correctly handled request a new frame
    if engine_state.process_events(event) {
      engine_state.renderer.request_redraw();
      return;
    }

    // Handle events for the input manager
    self.input_manager.process_events(event);

    match event {
      // Shutdown event, gets called when the engine should stop
      Event::Shutdown => {
        self.stop();
        return
      },
      // Resize event, gets called when size or scale of the window change
      Event::Resize {
        width,
        height
      } => {
        engine_state.resize(PhysicalSize {
          width: *width,
          height: *height
        });
      },
      _ => {}
    }

    // Should be the last one since events are replicated from the engine
    // to the application, thus the engine should try to handle events first
    event_f(self, event);
  }
}
