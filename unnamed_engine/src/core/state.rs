//! ## State
//!
//! Runtime data.
use std::{error::Error, sync::Arc};

use winit::{event::WindowEvent, window::Window};

use crate::{
  event::event::Event,
  renderer::{renderer::Renderer, viewport::Viewport},
};

use super::module::Module;

pub struct State {
  pub modules: Vec<Box<dyn Module>>,
}

impl State {
  pub async fn new(viewport: (Arc<Window>, wgpu::Color)) -> Self {
    let mut modules: Vec<Box<dyn Module>> = Vec::new();

    let renderer_module = Box::new(Renderer::new(viewport).await);

    modules.push(renderer_module);

    Self { modules }
  }

  pub fn register_module(&mut self, module: Box<dyn Module>) {
    self.modules.push(module);
  }

  pub fn process_events(&mut self, event: Event) -> Result<(), Box<dyn Error>> {
    for module in &mut self.modules {
      module.process_events(event)?
    }

    Ok(())
  }

  pub fn update(&mut self, dt: instant::Duration) -> Result<(), Box<dyn Error>> {
    for module in &mut self.modules {
      module.update(dt)?
    }

    Ok(())
  }

  pub fn render(&mut self) -> Result<(), Box<dyn Error>> {
    for module in &mut self.modules {
      module.render()?
    }

    Ok(())
  }

  pub fn find_viewport(&mut self) -> Option<&mut Viewport> {
    for module in &mut self.modules {
      match module.viewport() {
        Some(viewport) => return Some(viewport),
        None => return None,
      }
    }

    None
  }

  pub fn forward_window_event(&mut self, event: &WindowEvent) {
    for module in &mut self.modules {
      module.process_window_events(event);
    }
  }
}
