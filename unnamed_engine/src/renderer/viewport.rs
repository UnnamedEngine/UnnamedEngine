////////////////////////////////////////////////////////////////////////////////
//                ██╗   ██╗███╗   ██╗███████╗███╗   ██╗                       //
//                ██║   ██║████╗  ██║██╔════╝████╗  ██║                       //
//                ██║   ██║██╔██╗ ██║█████╗  ██╔██╗ ██║                       //
//                ██║   ██║██║╚██╗██║██╔══╝  ██║╚██╗██║                       //
//                ╚██████╔╝██║ ╚████║███████╗██║ ╚████║                       //
//                 ╚═════╝ ╚═╝  ╚═══╝╚══════╝╚═╝  ╚═══╝ LIB                   //
////////////////////////////////////////////////////////////////////////////////
// ? Defines a viewport

use std::sync::Arc;

use winit::window::Window;

pub struct ViewportDesc {
  pub window: Arc<Window>,
  pub background: wgpu::Color,
  pub surface: wgpu::Surface<'static>,
}

pub struct Viewport {
  pub desc: ViewportDesc,
  pub config: wgpu::SurfaceConfiguration,
  pub format: wgpu::TextureFormat,
}

impl ViewportDesc {
  pub fn new(window: Arc<Window>, background: wgpu::Color, instance: &wgpu::Instance) -> Self {
    let surface = instance.create_surface(window.clone()).unwrap();
    Self {
        window,
        background,
        surface,
    }
  }

  pub fn build(self, adapter: &wgpu::Adapter, device: &wgpu::Device) -> Viewport {
    let size = self.window.inner_size();

    let caps = self.surface.get_capabilities(&adapter);
    let format = caps.formats.iter()
      .copied()
      .find(|f| { f.is_srgb() })
      .unwrap_or(caps.formats[0]);

    let config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format,
      width: size.width,
      height: size.height,
      present_mode: caps.present_modes[0],
      desired_maximum_frame_latency: 2,
      alpha_mode: caps.alpha_modes[0],
      view_formats: vec![],
    };

    self.surface.configure(device, &config);

    Viewport {
      desc: self,
      config,
      format
    }
  }
}

impl Viewport {
  pub fn resize(&mut self, device: &wgpu::Device, size: winit::dpi::PhysicalSize<u32>) {
    self.config.width = size.width;
    self.config.height = size.height;
    self.desc.surface.configure(device, &self.config);
  }

  pub fn get_current_texture(&mut self) -> wgpu::SurfaceTexture {
    self.desc
      .surface
      .get_current_texture()
      .expect("Failed to acquire next swap chain texture")
  }

  pub fn set_title(&mut self, title: &str) {
    self.desc.window.set_title(title);
  }
}

