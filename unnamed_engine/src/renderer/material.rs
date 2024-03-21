use std::{error::Error, fs, path::PathBuf};

pub struct Material {
  shader: wgpu::ShaderModule,
  path: Option<PathBuf>,
}

impl Material {
  pub fn from_path(
    device: &wgpu::Device,
    path: PathBuf,
  ) -> Result<Self, Box<dyn Error>> {
    let shader_source = fs::read_to_string(&path)?;
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("material_shader"),
      source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    Ok(Self {
      shader,
      path: Some(path),
    })
  }

  pub fn from_string(
    device: &wgpu::Device,
    source: String,
  ) -> Self {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("material_shader"),
      source: wgpu::ShaderSource::Wgsl(source.into()),
    });

    Self {
      shader,
      path: None,
    }
  }

  pub fn shader(&self) -> &wgpu::ShaderModule {
    &self.shader
  }

  pub fn path(&self) -> &Option<PathBuf> {
    &self.path
  }
}
