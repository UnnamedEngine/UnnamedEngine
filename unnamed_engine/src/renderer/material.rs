use std::{error::Error, fs, path::PathBuf};

/// A material holds the necessary information that a shader will need in order
/// to work, think of a material as a wrapper for a shader.
pub struct Material {
  pub shader: wgpu::ShaderModule,
  pub path: Option<PathBuf>,
}

impl Material {
  /// Creates a material from a given path
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

  /// Creates a material from a given string
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
}
