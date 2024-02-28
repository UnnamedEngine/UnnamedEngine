use cgmath::SquareMatrix;

#[derive(Debug)]
pub struct Transform {
  pub position: cgmath::Vector3<f32>,
  pub rotation: cgmath::Quaternion<f32>,
  pub scale: cgmath::Vector3<f32>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformRaw {
  pub model: [[f32; 4]; 4],
}

impl Transform {
  pub fn to_raw(&self) -> TransformRaw {
    TransformRaw {
      model: (
        cgmath::Matrix4::from_translation(self.position) *
        cgmath::Matrix4::from(self.rotation) *
        cgmath::Matrix4::from_diagonal(
          cgmath::Vector4::new(
            self.scale.x,
            self.scale.y,
            self.scale.z,
            1.0))
      ).into(),
    }
  }
}

impl TransformRaw {
  pub fn desc() -> wgpu::VertexBufferLayout<'static> {
    use std::mem;
    wgpu::VertexBufferLayout {
      array_stride: mem::size_of::<TransformRaw>() as wgpu::BufferAddress,
      // Shader will only change to use the next instance when the shader starts processing a new
      // instance
      step_mode: wgpu::VertexStepMode::Instance,
      attributes: &[
        // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot for
        // each vec4. We'll have to reassemble the mat4 in the shader.
        wgpu::VertexAttribute {
          offset: 0,
          shader_location: 5,
          format: wgpu:: VertexFormat::Float32x4,
        },
        wgpu::VertexAttribute {
          offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
          shader_location: 6,
          format: wgpu::VertexFormat::Float32x4,
        },
        wgpu::VertexAttribute {
          offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
          shader_location: 7,
          format: wgpu::VertexFormat::Float32x4,
        },
        wgpu::VertexAttribute {
          offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
          shader_location: 8,
          format: wgpu::VertexFormat::Float32x4,
        },
      ],
    }
  }
}
