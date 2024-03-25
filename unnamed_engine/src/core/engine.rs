//! ## Engine
//!
//! Defines the engine struct.
use std::error::Error;

use super::state::State;

use crate::{
  event::event::Event,
  input::{input_manager::InputManager, keyboard::Keyboard},
  networking::common::{make_server_endpoint, run_client, run_server},
};

use env_logger::Env;
use tokio::runtime::Runtime;
use winit::{
  event::{Event as WinitEvent, WindowEvent as WinitWindowEvent},
  event_loop::EventLoop,
  keyboard::PhysicalKey,
  window::WindowBuilder,
};

/// ## Engine
///
/// Main struct of UnnamedEngine, its the core wrapper that contains the upper abstract methods and
/// defines the general flow of execution.
pub struct Engine {
  running: bool,
  title: String,
  pub input_manager: InputManager,
  pub state: Option<State>,
}

impl Engine {
  pub fn new(title: String) -> Self {
    let input_manager = InputManager::new();
    let state = None;

    Engine {
      running: false,
      title,
      input_manager,
      state,
    }
  }

  pub fn get_state(&mut self) -> Result<&mut State, Box<dyn Error>> {
    match self.state {
      Some(ref mut state) => Ok(state),
      None => Err("No state found".into()),
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
    event_f: impl FnMut(&mut Engine, Event),
  ) {
    // Initializes the logger
    let env = Env::default()
      .filter_or("MY_LOG_LEVEL", "info")
      .write_style_or("MY_LOG_STYLE", "always");
    env_logger::init_from_env(env);

    start_f(self);

    let runtime = Runtime::new().unwrap();

    runtime.block_on(async move {
      let server_addr = "127.0.0.1:32732".parse().unwrap();
      let (server_endpoint, server_cert) = make_server_endpoint(server_addr).unwrap();

      tokio::spawn(run_server(server_endpoint));
      tokio::spawn(run_client(server_addr, server_cert));
    });

    self.running = true;
    tokio::runtime::Runtime::new()
      .unwrap()
      .block_on(self.run(update_f, render_f, event_f));
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
    mut event_f: impl FnMut(&mut Engine, Event),
  ) {
    let event_loop = EventLoop::new().expect("Could not create event loop");
    let window = WindowBuilder::new()
      .build(&event_loop)
      .expect("Could not create window");
    window.set_title(&self.title);

    let mut last_render_time = instant::Instant::now();
    let my_window_id = window.id();
    self.state = Some(State::new((window.into(), wgpu::Color::BLACK)).await);

    event_loop
      .run(move |event, elwt| {
        elwt.set_control_flow(winit::event_loop::ControlFlow::Poll);
        match event {
          WinitEvent::WindowEvent {
            ref event,
            window_id,
          } if window_id == my_window_id => {
            match &event {
              // Close event
              WinitWindowEvent::CloseRequested => {
                // Create and dispatch shutdown event
                self.on_event(Event::Shutdown, &mut event_f);
              }
              // Resize event
              WinitWindowEvent::Resized(physical_size) => {
                // Create and dispatch resize event
                // TODO make it work as an event
                self.on_event(
                  Event::Resize {
                    width: physical_size.width,
                    height: physical_size.height,
                  },
                  &mut event_f,
                );
              }
              // Scale changed event
              WinitWindowEvent::ScaleFactorChanged { .. } => {
                // Create and dispatch resize event
                let size = self
                  .get_state()
                  .unwrap()
                  .find_viewport()
                  .unwrap()
                  .desc
                  .window
                  .inner_size();
                self.on_event(
                  Event::Resize {
                    width: size.width,
                    height: size.height,
                  },
                  &mut event_f,
                );
              }
              // Keyboard input event
              WinitWindowEvent::KeyboardInput { event, .. } => {
                match event.physical_key {
                  PhysicalKey::Code(key_code) => {
                    // Send a keyboard input event
                    self.on_event(
                      Event::KeyboardInput {
                        key: key_code,
                        is_pressed: event.state.is_pressed(),
                      },
                      &mut event_f,
                    );
                  }
                  PhysicalKey::Unidentified(_) => {}
                }
              }
              // Mouse moved
              WinitWindowEvent::CursorMoved {
                device_id: _,
                position,
              } => {
                // Send a mouse moved event
                self.on_event(
                  Event::MousePosition {
                    x: position.x as u32,
                    y: position.y as u32,
                  },
                  &mut event_f,
                );
              }
              // Mouse clicked
              WinitWindowEvent::MouseInput {
                device_id: _,
                state,
                button,
              } => {
                // Send a mouse input event
                self.on_event(
                  Event::MouseInput {
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
              WinitWindowEvent::MouseWheel { delta, .. } => match delta {
                winit::event::MouseScrollDelta::LineDelta(x, y) => {
                  self.on_event(Event::MouseScroll { delta: (*x, *y) }, &mut event_f);
                }
                _ => {}
              },
              WinitWindowEvent::RedrawRequested
                if window_id
                  == self
                    .get_state()
                    .unwrap()
                    .find_viewport()
                    .unwrap()
                    .desc
                    .window
                    .id() =>
              {
                // Should we stop?
                if !self.running {
                  elwt.exit();
                }

                let now = instant::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;

                // Update the state
                self.get_state().unwrap().update(dt);

                // Update the upper application
                update_f(self);

                // Render the state
                self
                  .get_state()
                  .unwrap()
                  .render()
                  .expect("Failed to render");

                // Render the upper application
                render_f(self);
              }
              _ => {}
            }
            self.get_state().unwrap().forward_window_event(event);
          }
          WinitEvent::DeviceEvent {
            device_id: _,
            event,
          } => match event {
            winit::event::DeviceEvent::MouseMotion { delta } => {
              self.on_event(
                Event::MouseMotion {
                  x: delta.0 as f32,
                  y: delta.1 as f32,
                },
                &mut event_f,
              );
            }
            _ => {}
          },
          // Sometimes this will be called, and when it gets called it should just request a new frame
          WinitEvent::AboutToWait => {
            self.on_event(Event::Redraw, &mut event_f);
          }
          _ => {}
        }
      })
      .expect("Failed to run a loop");
  }

  /// Gets called from the event loop and replicate the events to the applications
  fn on_event(&mut self, event: Event, event_f: &mut impl FnMut(&mut Engine, Event)) {
    // First try to handle it internally, if correctly handled request a new frame
    match self.get_state().unwrap().process_events(event) {
      Ok(_) => {}
      Err(_e) => {}
    }

    // Handle events for the input manager
    self.input_manager.process_events(event);

    match event {
      // Shutdown event, gets called when the engine should stop
      Event::Shutdown => {
        self.stop();
        return;
      }
      _ => {}
    }

    // Should be the last one since events are replicated from the engine
    // to the application, thus the engine should try to handle events first
    event_f(self, event);
  }
}
