//! ## State
//!
//! Runtime data.
use std::{sync::Arc, time::Duration};

use winit::window::Window;

use crate::{event::event::Event, renderer::renderer::Renderer};

pub struct State {
  pub renderer: Renderer,
}

impl State {
  pub async fn new(viewport: (Arc<Window>, wgpu::Color)) -> Self {
    let renderer = Renderer::new(viewport).await;

    Self {
      renderer,
    }
  }

  pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
    self.renderer.resize(new_size);
  }

  pub fn process_events(&mut self, event: Event) -> bool {
    self.renderer.process_events(event)
  }

  pub fn update(&mut self, dt: Duration) {
    self.renderer.update(dt);
  }

  pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    self.renderer.render()
  }
}
