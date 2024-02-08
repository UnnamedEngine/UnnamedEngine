////////////////////////////////////////////////////////////////////////////////
//                ██╗   ██╗███╗   ██╗███████╗███╗   ██╗                       //
//                ██║   ██║████╗  ██║██╔════╝████╗  ██║                       //
//                ██║   ██║██╔██╗ ██║█████╗  ██╔██╗ ██║                       //
//                ██║   ██║██║╚██╗██║██╔══╝  ██║╚██╗██║                       //
//                ╚██████╔╝██║ ╚████║███████╗██║ ╚████║                       //
//                 ╚═════╝ ╚═╝  ╚═══╝╚══════╝╚═╝  ╚═══╝ LIB                   //
////////////////////////////////////////////////////////////////////////////////
// ? Defines a middleware renderer that stores and executes everything related
// ? to the graphics library.

use wgpu::{util::DeviceExt, BindGroup, TextureFormat};

use super::{camera::CameraController, texture::{self, Texture}};

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

const VERTICES: &[Vertex] = &[
  Vertex { position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 1.0] },
  Vertex { position: [0.5, -0.5, 0.0],  tex_coords: [1.0, 1.0] },
  Vertex { position: [-0.5, 0.5, 0.0],  tex_coords: [0.0, 0.0] },
  Vertex { position: [0.5, 0.5, 0.0],   tex_coords: [1.0, 0.0] },
];

const INDICES: &[u16] = &[
  0, 1, 2,
  2, 1, 3,
];


pub struct MiddlewareRenderer {
  diffuse_texture: Texture,
  diffuse_bind_group: BindGroup,
  shader: wgpu::ShaderModule,
  render_pipeline: wgpu::RenderPipeline,
  vertex_buffer: wgpu::Buffer,
  index_buffer: wgpu::Buffer,
  num_indices: u32,
}

impl MiddlewareRenderer {
  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    camera_controller: &CameraController,
    texture_format: TextureFormat,
  ) -> Self {
    let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("texture_bind_group_layout"),
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
          },
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 1,
          visibility: wgpu::ShaderStages::FRAGMENT,
          // This should match the filterable filed of the corresponding Texture
          // entry above
          ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
          count: None,
        },
      ],
    });

    let diffuse_bytes = include_bytes!("../../res/dirt.png");
    let diffuse_texture = texture::Texture::from_bytes(device, queue, diffuse_bytes, "dirt.png").unwrap();

    let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("diffuse_bind_group"),
      layout: &texture_bind_group_layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
        }
      ]
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("render_pipeline_layout"),
      bind_group_layouts: &[
        &texture_bind_group_layout,
        &camera_controller.bind_group_layout,
      ],
      push_constant_ranges: &[],
    });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("shader"),
      source: wgpu::ShaderSource::Wgsl(include_str!("../shader.wgsl").into()),
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("render_pipeline"),
      layout: Some(&render_pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: "vs_main",
        buffers: &[Vertex::desc()],
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: "fs_main",
        targets: &[Some(wgpu::ColorTargetState {
          format: texture_format,
          blend: Some(wgpu::BlendState::REPLACE),
          write_mask: wgpu::ColorWrites::ALL,
        })],
      }),
      primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: Some(wgpu::Face::Back),
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
      multiview: None
    });

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("vertex_buffer"),
      contents: bytemuck::cast_slice(VERTICES),
      usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("index_buffer"),
      contents: bytemuck::cast_slice(INDICES),
      usage: wgpu::BufferUsages::INDEX,
    });

    let num_indices = INDICES.len() as u32;

    Self {
      diffuse_texture,
      diffuse_bind_group,
      shader,
      render_pipeline,
      vertex_buffer,
      index_buffer,
      num_indices,
    }
  }

  pub fn render(
    &mut self,
    surface: &wgpu::Surface,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    camera_controller: &CameraController,
  ) -> Result<(), wgpu::SurfaceError> {
    let output = surface.get_current_texture()?;

    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("render_encoder"),
    });


    {
      let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("render_pass"),
        color_attachments: &[
          // This is what @location(0) in the fragment shader targets
          Some(wgpu::RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: wgpu::Operations {
              load: wgpu::LoadOp::Clear(wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
              }),
              store: wgpu::StoreOp::Store,
            },
          })
        ],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
      });

      render_pass.set_pipeline(&self.render_pipeline);
      render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
      render_pass.set_bind_group(1, &camera_controller.bind_group, &[]);
      render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
      render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
      render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }

    // Submit will accept anything that implements IntoIter
    queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
  }
}
