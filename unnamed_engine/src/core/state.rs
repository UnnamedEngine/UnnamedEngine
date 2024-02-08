////////////////////////////////////////////////////////////////////////////////
//                ██╗   ██╗███╗   ██╗███████╗███╗   ██╗                       //
//                ██║   ██║████╗  ██║██╔════╝████╗  ██║                       //
//                ██║   ██║██╔██╗ ██║█████╗  ██╔██╗ ██║                       //
//                ██║   ██║██║╚██╗██║██╔══╝  ██║╚██╗██║                       //
//                ╚██████╔╝██║ ╚████║███████╗██║ ╚████║                       //
//                 ╚═════╝ ╚═╝  ╚═══╝╚══════╝╚═╝  ╚═══╝ LIB                   //
////////////////////////////////////////////////////////////////////////////////
// ? Where the data that will be used during execution is defined and handled.

use std::sync::Arc;

use wgpu::InstanceFlags;
use winit::window::Window;

use crate::{event::event::Event, renderer::{camera::{CameraController, CameraDescriptor}, middleware_renderer::MiddlewareRenderer, viewport::{Viewport, ViewportDesc}}};

pub struct State {
  viewport: Viewport,
  device: wgpu::Device,
  queue: wgpu::Queue,
  camera_controller: CameraController,
  middleware_renderer: MiddlewareRenderer,
}

impl State {
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
    .request_adapter(
      &wgpu::RequestAdapterOptions {
        // Request an adapter which can render to our surface
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
          // WebGL doesn't support all the wgpu's features, so if
          // we're building for the web we'll have to disable some
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

    let camera_controller = CameraController::new(&device, &CameraDescriptor {
      speed: 0.2,
      aspect: viewport.config.width as f32 / viewport.config.height as f32,
      fovy: 45.0,
      near: 0.1,
      far: 100.0
    });

    let middleware_renderer = MiddlewareRenderer::new(
      &device,
      &queue,
      &camera_controller,
      viewport.format,
    );

    Self {
      viewport,
      device,
      queue,
      camera_controller,
      middleware_renderer,
    }
  }

  pub fn viewport(&self) -> &Viewport {
    &self.viewport
  }

  pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
    if new_size.width > 0 && new_size.height > 0 {
      self.viewport.resize(&self.device, new_size)
    }
  }

  pub fn input(&mut self, event: &Event) -> bool {
    self.camera_controller.process_events(event)
  }

  pub fn update(&mut self) {
    self.camera_controller.update();
    self.queue.write_buffer(&self.camera_controller.buffer, 0, bytemuck::cast_slice(&[self.camera_controller.uniform]));
  }

  pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    self.middleware_renderer.render(
      &mut self.viewport,
      &self.device,
      &self.queue,
      &self.camera_controller,
    )
  }
}
