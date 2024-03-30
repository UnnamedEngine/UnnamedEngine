//! ## Middleware Renderer
//!
//! Contains the low level boilerplate for the renderer, the data here should be
//! passed around inside the renderer.

use std::sync::Arc;

use wgpu::InstanceFlags;
use winit::window::Window;

use super::viewport::{Viewport, ViewportDesc};

/// Contains the low low level data that is used inside the renderer
pub struct MiddlewareRenderer {
  pub device: wgpu::Device,
  pub queue: wgpu::Queue,
  pub viewport: Viewport,
}

impl MiddlewareRenderer {
  pub async fn new(
    viewport: (Arc<Window>, wgpu::Color),
  ) -> Self {
    // The instance is a handle to our GPU
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
      backends: wgpu::Backends::all(), // Vulkan + DX12 + Browser WebGPU
      dx12_shader_compiler: Default::default(),
      flags: InstanceFlags::default(),
      gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
    });

    let viewport = ViewportDesc::new(viewport.0, viewport.1, &instance);

    let adapter = instance
      .request_adapter(&wgpu::RequestAdapterOptions {
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
          // WebGL doesn't support all the wgpu's features, so if we're building
          // for the web we'll have to disable some
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

    Self {
      device,
      queue,
      viewport,
    }
  }
}
