////////////////////////////////////////////////////////////////////////////////
//                ██╗   ██╗███╗   ██╗███████╗███╗   ██╗                       //
//                ██║   ██║████╗  ██║██╔════╝████╗  ██║                       //
//                ██║   ██║██╔██╗ ██║█████╗  ██╔██╗ ██║                       //
//                ██║   ██║██║╚██╗██║██╔══╝  ██║╚██╗██║                       //
//                ╚██████╔╝██║ ╚████║███████╗██║ ╚████║                       //
//                 ╚═════╝ ╚═╝  ╚═══╝╚══════╝╚═╝  ╚═══╝ LIB                   //
////////////////////////////////////////////////////////////////////////////////
// ? Where the data that will be used during execution is defined and handled.

use wgpu::InstanceFlags;
use winit::window::Window;

use crate::{event::event::Event, renderer::{camera::{CameraController, CameraDescriptor}, middleware_renderer::MiddlewareRenderer}};

pub struct State {
  surface: wgpu::Surface,
  device: wgpu::Device,
  queue: wgpu::Queue,
  config: wgpu::SurfaceConfiguration,
  size: winit::dpi::PhysicalSize<u32>,
  window: Window,
  camera_controller: CameraController,
  middleware_renderer: MiddlewareRenderer,
}

impl State {
  pub async fn new(window: Window) -> Self {
    let size = window.inner_size();

    // The instance is a handle to our GPU
    // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
      backends: wgpu::Backends::all(),
      dx12_shader_compiler: Default::default(),
      flags: InstanceFlags::default(),
      gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
    });

    let surface = unsafe { instance.create_surface(&window) }.unwrap();

    let adapter = instance.request_adapter(
      &wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
      },
    ).await.unwrap();

    let (device, queue) = adapter.request_device(
      &wgpu::DeviceDescriptor {
        features: wgpu::Features::empty(),
        // WebGL doesn't support all the wgpu's features, so if
        // we're building for the web we'll have to disable some
        limits: if cfg!(target_arch = "wasm32") {
          wgpu::Limits::downlevel_webgl2_defaults()
        } else {
          wgpu::Limits::default()
        },
        label: None,
      },
      None,
    ).await.unwrap();

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps.formats.iter()
      .copied()
      .find(|f| { f.is_srgb() })
      .unwrap_or(surface_caps.formats[0]);
    let config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: surface_format,
      width: size.width,
      height: size.height,
      present_mode: surface_caps.present_modes[0],
      alpha_mode: surface_caps.alpha_modes[0],
      view_formats: vec![],
    };
    surface.configure(&device, &config);

    let camera_controller = CameraController::new(&device, &CameraDescriptor {
      speed: 0.2,
      aspect: config.width as f32 / config.height as f32,
      fovy: 45.0,
      near: 0.1,
      far: 100.0
    });

    let middleware_renderer = MiddlewareRenderer::new(
      &device,
      &queue,
      &camera_controller,
      surface_format,
    );

    Self {
      window,
      surface,
      device,
      queue,
      config,
      size,
      camera_controller,
      middleware_renderer,
    }
  }

  pub fn window(&self) -> &Window {
    &self.window
  }

  pub fn size(&self) -> &winit::dpi::PhysicalSize<u32> {
    &self.size
  }

  pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
    if new_size.width > 0 && new_size.height > 0 {
      self.size = new_size;
      self.config.width = new_size.width;
      self.config.height = new_size.height;
      self.surface.configure(&self.device, &self.config);
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
      &self.surface,
      &self.device,
      &self.queue,
      &self.camera_controller,
    )
  }
}
