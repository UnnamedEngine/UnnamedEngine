//! ## Renderer
//!
//! Defines and implements the renderer and the related structs.

use std::{error::Error, sync::Arc, time::Duration};

use wgpu::InstanceFlags;
use winit::window::Window;

use crate::{core::module::Module, event::event::Event, gui::egui_renderer::EguiRenderer};

use super::{
  camera::CameraController,
  middleware_renderer::MiddlewareRenderer,
  viewport::{Viewport, ViewportDesc},
};

/// ## Renderer
///
/// A renderer is the most important wrapper around rendering, it contains the
/// required data for rendering and defines the general flow of the rendering.
pub struct Renderer {
  pub viewport: Viewport,
  device: wgpu::Device,
  queue: wgpu::Queue,
  pub camera_controller: CameraController,
  middleware: MiddlewareRenderer,
  pub egui: EguiRenderer,
}

impl Module for Renderer {
  fn process_events(&mut self, event: Event) -> Result<(), Box<dyn Error>> {
    self.camera_controller.process_events(event);
    match event {
      Event::Resize { width, height } => {
        if width > 0 && height > 0 {
          self.viewport.resize(&self.device, width, height);
          self.camera_controller.resize(width, height);
          self.middleware.resize(&self.device, &self.viewport);

          // Request a redraw just in case
          self.viewport.desc.window.request_redraw();
        }
      }
      Event::Redraw => {
        self.viewport.desc.window.request_redraw();
      }
      _ => {}
    }

    Ok(())
  }

  fn update(&mut self, dt: Duration) -> Result<(), Box<dyn Error>> {
    self.camera_controller.update(dt);
    self.queue.write_buffer(
      &self.camera_controller.buffer,
      0,
      bytemuck::cast_slice(&[self.camera_controller.uniform]),
    );

    Ok(())
  }

  fn render(&mut self) -> Result<(), Box<dyn Error>> {
    let _ = self.middleware.render(
      &mut self.viewport,
      &self.device,
      &self.queue,
      &self.camera_controller,
      &mut self.egui,
    );

    Ok(())
  }

  fn viewport(&mut self) -> Option<&mut Viewport> {
    Some(&mut self.viewport)
  }

  fn process_window_events(&mut self, event: &winit::event::WindowEvent) {
    self.egui.handle_input(&self.viewport.desc.window, event);
  }
}

impl Renderer {
  pub async fn new(viewport: (Arc<Window>, wgpu::Color)) -> Self {
    // The instance is a handle to our GPU
    // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
      backends: wgpu::Backends::all(),
      dx12_shader_compiler: Default::default(),
      flags: InstanceFlags::default(),
      gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
    });

    let viewport = ViewportDesc::new(viewport.0, viewport.1, &instance);

    let adapter = instance
      .request_adapter(&wgpu::RequestAdapterOptions {
        // Request and adapter which can render to our surface
        compatible_surface: Some(&viewport.surface),
        ..Default::default()
      })
      .await
      .expect("Failed to find an appropriate adapter");

    // Create the logical device and command queue
    let (device, queue) = adapter
      .request_device(
        &wgpu::DeviceDescriptor {
          label: None,
          required_features: wgpu::Features::empty(),
          // WebGL doesn't support all the wgpu's features, so if we're building for the web we'll
          // have to disable some
          required_limits: if cfg!(target_arch = "wasm32") {
            wgpu::Limits::downlevel_webgl2_defaults()
          } else {
            wgpu::Limits::default()
          },
        },
        None,
      )
      .await
      .expect("Failed to create device");

    let viewport = viewport.build(&adapter, &device);

    let egui = EguiRenderer::new(&device, viewport.format, None, 1, &viewport.desc.window);

    let camera_controller = CameraController::new(
      &device,
      4.0,
      0.4,
      viewport.config.width,
      viewport.config.height,
    );

    let middleware = MiddlewareRenderer::new(&device, &queue, &camera_controller, &viewport);

    Self {
      viewport,
      device,
      queue,
      camera_controller,
      middleware,
      egui,
    }
  }
}
