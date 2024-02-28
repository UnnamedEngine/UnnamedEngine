use wgpu::util::DeviceExt;

use super::{texture::{self, Texture}, viewport::Viewport};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
  position: [f32; 3],
  tex_coords: [f32; 2],
}

impl Vertex {
  fn desc() -> wgpu::VertexBufferLayout<'static> {
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
          format: wgpu::VertexFormat::Float32x2,
        }
      ]
    }
  }
}

pub const SCREEN_VERTICES: &[Vertex] = &[
  Vertex { position: [-1.0, -1.0, 0.0], tex_coords: [0.0, 1.0] }, // 0
  Vertex { position: [ 1.0, -1.0, 0.0], tex_coords: [1.0, 1.0] }, // 1
  Vertex { position: [-1.0,  1.0, 0.0], tex_coords: [0.0, 0.0] }, // 2
  Vertex { position: [ 1.0,  1.0, 0.0], tex_coords: [1.0, 0.0] }, // 3
];

pub const SCREEN_INDICES: &[u16] = &[
  0, 1, 2, 2, 1, 3,
];

pub struct Screen {
  pub diffuse_texture: Texture,
  pub depth_texture: Texture,
  pub shader: wgpu::ShaderModule,
  pub vertex_buffer: wgpu::Buffer,
  pub index_buffer: wgpu::Buffer,
  pub pipeline: wgpu::RenderPipeline,
}

impl Screen {
  pub fn new(
    device: &wgpu::Device,
    viewport: &Viewport,
    bind_group_layout: &wgpu::BindGroupLayout,
  ) -> Self {
    let mut diffuse_texture = texture::Texture::create_diffuse_texture(
      device,
      &viewport.config,
      "screen_diffuse_texture",
      Some(viewport.config.format),
      Some(
        wgpu::TextureUsages::RENDER_ATTACHMENT |
        wgpu::TextureUsages::TEXTURE_BINDING
      ),
    );
    diffuse_texture.set_bind_group(device, bind_group_layout);

    let depth_texture = texture::Texture::create_depth_texture(device, &viewport.config, "screen_depth_texture");

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: None,
      source: wgpu::ShaderSource::Wgsl(include_str!("../screen.wgsl").into()),
    });

    let vertex_buffer = device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: Some("screen_vertex_buffer"),
        contents: bytemuck::cast_slice(SCREEN_VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
      }
    );

    let index_buffer = device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: Some("screen_index_buffer"),
        contents: bytemuck::cast_slice(SCREEN_INDICES),
        usage: wgpu::BufferUsages::INDEX,
      }
    );

    let pipeline_layout = device.create_pipeline_layout(
      &wgpu::PipelineLayoutDescriptor {
        label: Some("screen_render_pipeline_layout"),
        bind_group_layouts: &[
          bind_group_layout,
        ],
        push_constant_ranges: &[],
      }
    );

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("screen_render_pipeline"),
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: "vs_main",
        buffers: &[Vertex::desc()],
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: "fs_main",
        targets: &[Some(wgpu::ColorTargetState {
          format: viewport.format,
          blend: Some(wgpu::BlendState::REPLACE),
          write_mask: wgpu::ColorWrites::ALL,
        })],
      }),
      primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: None,
        unclipped_depth: false,
        polygon_mode: wgpu::PolygonMode::Fill,
        conservative: false,
      },
      depth_stencil: None,
      multisample: wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
      multiview: None,
    });

    Self {
      diffuse_texture,
      depth_texture,
      shader,
      vertex_buffer,
      index_buffer,
      pipeline,
    }
  }

  pub fn resize(
    &mut self,
    device: &wgpu::Device,
    viewport: &Viewport,
    bind_group_layout: &wgpu::BindGroupLayout,
  ) {
    self.diffuse_texture = texture::Texture::create_diffuse_texture(
      device,
      &viewport.config,
      "screen_diffuse_texture",
      Some(viewport.config.format),
      Some(
        wgpu::TextureUsages::RENDER_ATTACHMENT |
        wgpu::TextureUsages::TEXTURE_BINDING
      ),
    );
    self.diffuse_texture.set_bind_group(device, bind_group_layout);

    self.depth_texture = texture::Texture::create_depth_texture(
      device,
      &viewport.config,
      "screen_depth_texture",
    );
  }
}
