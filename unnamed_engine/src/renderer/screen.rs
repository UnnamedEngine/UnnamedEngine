use super::{texture::{self, Texture}, viewport::Viewport};

pub struct Screen {
  pub diffuse_texture: Texture,
  pub depth_texture: Texture,
  pub shader: wgpu::ShaderModule,
}

impl Screen {
  pub fn new(
    device: &wgpu::Device,
    viewport: &Viewport,
    bind_group_layout: &wgpu::BindGroupLayout,
  ) -> Self {
    let mut diffuse_texture = texture::Texture::create_diffuse_texture(device, &viewport.config, "diffuse_texture");
    diffuse_texture.set_bind_group(device, bind_group_layout);

    let mut depth_texture = texture::Texture::create_depth_texture(device, &viewport.config, "depth_texture");
    depth_texture.set_bind_group(device, bind_group_layout);

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: None,
      source: wgpu::ShaderSource::Wgsl(include_str!("../screen.wgsl").into()),
    });

    Self {
      diffuse_texture,
      depth_texture,
      shader,
    }
  }
}
