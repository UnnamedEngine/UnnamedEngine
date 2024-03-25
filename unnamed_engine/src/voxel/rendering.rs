use std::mem::size_of;

use wgpu::util::DeviceExt;

use crate::renderer::middleware_renderer::RenderingStats;

use super::Chunk;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
  position: [f32; 3],
  color: [f32; 3],
}

impl Vertex {
  pub fn desc() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
      array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &[
        wgpu::VertexAttribute {
          offset: 0,
          shader_location: 0,
          format: wgpu::VertexFormat::Float32x3,
        },
        wgpu::VertexAttribute {
          offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
          shader_location: 1,
          format: wgpu::VertexFormat::Float32x3,
        },
      ],
    }
  }
}

const VERTICES: &[Vertex] = &[
  Vertex { // 0
    position: [-0.5, -0.5, 0.5],
    color: [1.0, 0.0, 0.0],
  },
  Vertex { // 1
    position: [0.5, -0.5, 0.5],
    color: [0.0, 1.0, 0.0],
  },
  Vertex { // 2
    position: [-0.5, 0.5, 0.5],
    color: [0.0, 0.0, 1.0],
  },
  Vertex { // 3
    position: [0.5, 0.5, 0.5],
    color: [1.0, 1.0, 0.0],
  },
  Vertex { // 4
    position: [-0.5, -0.5, -0.5],
    color: [0.0, 1.0, 1.0],
  },
  Vertex { // 5
    position: [0.5, -0.5, -0.5],
    color: [1.0, 1.0, 0.0],
  },
  Vertex { // 6
    position: [-0.5, 0.5, -0.5],
    color: [1.0, 0.0, 0.0],
  },
  Vertex { // 7
    position: [0.5, 0.5, -0.5],
    color: [0.0, 0.0, 1.0],
  },
];

const INDICES: &[u32] = &[
  0, 1, 2, 2, 1, 3,
  5, 4, 7, 7, 4, 6,
  4, 0, 6, 6, 0, 2,
  1, 5, 3, 3, 5, 7,
  2, 3, 6, 6, 3, 7,
  4, 5, 0, 0, 5, 1,
  ];

pub struct ChunkMesh {
  pub vertex_buffer: wgpu::Buffer,
  pub index_buffer: wgpu::Buffer,
  pub indices: u32,
}

impl ChunkMesh {
  pub fn new(
    device: &wgpu::Device,
    chunk: &Chunk,
    stats: &mut RenderingStats,
  ) -> Self {
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut counter: u32 = 0;
    for (position, _voxel) in chunk.iter() {
      for vertex in VERTICES {
        let mut vertex = vertex.clone();
        vertex.position = [
          vertex.position[0] + position.x as f32,
          vertex.position[1] + position.y as f32,
          vertex.position[2] + position.z as f32,
        ];
        vertices.push(vertex);
      }
      for index in INDICES {
        indices.push(*index + 8 * counter);
      }
      counter += 1;
    }
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("chunk_vertex_buffer"),
      contents: bytemuck::cast_slice(vertices.as_slice()),
      usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("chunk_index_buffer"),
      contents: bytemuck::cast_slice(indices.as_slice()),
      usage: wgpu::BufferUsages::INDEX,
    });

    let vertices_byte_size = size_of::<Vertex>() * vertices.len();
    let indices_byte_size = size_of::<u32>() * indices.len();

    stats.bytes += vertices_byte_size + indices_byte_size;

    Self {
      vertex_buffer,
      index_buffer,
      indices: (counter) * INDICES.len() as u32,
    }
  }
}
