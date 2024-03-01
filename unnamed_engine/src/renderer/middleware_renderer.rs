//! ## Middleware Renderer
//!
//! Defines a middleware that stores and executes everything related to the graphics library.
use cgmath::{InnerSpace, Rotation3, Zero};
use egui_wgpu::ScreenDescriptor;
use wgpu::util::DeviceExt;

use crate::gui::{egui_renderer::EguiRenderer, gui::gui};

use super::{camera::CameraController, screen::Screen, texture::{self, Texture}, transform::{Transform, TransformRaw}, viewport::Viewport};

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
  Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0, 1.0] },
  Vertex { position: [ 0.5, -0.5, -0.5], tex_coords: [1.0, 1.0] },
  Vertex { position: [-0.5,  0.5, -0.5], tex_coords: [0.0, 0.0] },
  Vertex { position: [ 0.5,  0.5, -0.5], tex_coords: [1.0, 0.0] },
  Vertex { position: [-0.5, -0.5,  0.5], tex_coords: [0.0, 1.0] },
  Vertex { position: [ 0.5, -0.5,  0.5], tex_coords: [1.0, 1.0] },
  Vertex { position: [-0.5,  0.5,  0.5], tex_coords: [0.0, 0.0] },
  Vertex { position: [ 0.5,  0.5,  0.5], tex_coords: [1.0, 0.0] },
];

const INDICES: &[u16] = &[
  0, 1, 2, 2, 1, 3,
];

const NUM_INSTANCES_PER_ROW: u32 = 10;
const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(NUM_INSTANCES_PER_ROW as f32 * 0.5, 0.0, NUM_INSTANCES_PER_ROW as f32 * 0.5);


pub struct MiddlewareRenderer {
  texture_bind_group_layout: wgpu::BindGroupLayout,
  shader: wgpu::ShaderModule,
  pipeline: wgpu::RenderPipeline,
  vertex_buffer: wgpu::Buffer,
  index_buffer: wgpu::Buffer,
  transforms: Vec<Transform>,
  instance_buffer: wgpu::Buffer,
  num_indices: u32,
  screen: Screen,

  test_texture: Texture,
}

impl MiddlewareRenderer {
  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    camera_controller: &CameraController,
    viewport: &Viewport,
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

    let screen = Screen::new(device, viewport, &texture_bind_group_layout);

    let test_bytes = include_bytes!("../../res/dirt.png");
    let mut test_texture = texture::Texture::from_bytes(device, queue, test_bytes, "dirt.png").unwrap();
    test_texture.set_bind_group(device, &texture_bind_group_layout);

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("pipeline_layout"),
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

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("pipeline"),
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: "vs_main",
        buffers: &[Vertex::desc(), TransformRaw::desc()],
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
        cull_mode: None, //Some(wgpu::Face::Back),
        unclipped_depth: false,
        polygon_mode: wgpu::PolygonMode::Fill,
        conservative: false,
      },
      depth_stencil: Some(wgpu::DepthStencilState {
        format: texture::Texture::DEPTH_FORMAT,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::Less,
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
      }),
      multisample: wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
      multiview: None,
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

    let transforms = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
      (0..NUM_INSTANCES_PER_ROW).map(move |x| {
        let position = cgmath::Vector3 {x: x as f32, y: 0.0, z: z as f32} - INSTANCE_DISPLACEMENT;

        let rotation = if position.is_zero() {
          cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
        } else {
          cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
        };

        let scale  = cgmath::Vector3{x: 1.0, y: 1.0, z: 1.0};

        Transform {
          position, rotation, scale
        }
      })
    }).collect::<Vec<_>>();

    let instance_data = transforms.iter().map(Transform::to_raw).collect::<Vec<_>>();
    let instance_buffer = device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: Some("instance_buffer"),
        contents: bytemuck::cast_slice(&instance_data),
        usage: wgpu::BufferUsages::VERTEX,
      }
    );

    Self {
      texture_bind_group_layout,
      shader,
      pipeline,
      vertex_buffer,
      index_buffer,
      num_indices,
      transforms,
      instance_buffer,
      screen,
      test_texture,
    }
  }

  pub fn render(
    &mut self,
    viewport: &mut Viewport,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    camera_controller: &CameraController,
    egui: &mut EguiRenderer,
  ) -> Result<(), wgpu::SurfaceError> {
    let output = viewport.get_current_texture();

    let renderer_view = &self.screen.diffuse_texture.view;
    let window_view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("render_encoder"),
    });

    {
      let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("render_pass"),
        color_attachments: &[
          // This is what @location(0) in the fragment shader targets
          Some(wgpu::RenderPassColorAttachment {
            view: renderer_view,
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
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
          view: &self.screen.depth_texture.view,
          depth_ops: Some(wgpu::Operations {
            load: wgpu::LoadOp::Clear(1.0),
            store: wgpu::StoreOp::Store,
          }),
          stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
      });

      render_pass.set_pipeline(&self.pipeline);
      if let Some(test_texture) = &self.test_texture.bind_group {
        render_pass.set_bind_group(0, test_texture, &[]);
      }
      render_pass.set_bind_group(1, &camera_controller.bind_group, &[]);
      render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

      render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
      render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

      render_pass.draw_indexed(0..self.num_indices, 0, 0..self.transforms.len() as _);
    }

    self.screen.draw(
      &mut encoder,
      &window_view,
    );

    let screen_descriptor = ScreenDescriptor {
      size_in_pixels: [viewport.config.width, viewport.config.height],
      pixels_per_point: viewport.desc.window.scale_factor() as f32,
    };

    egui.draw(
      device,
      queue,
      &mut encoder,
      &viewport.desc.window,
      &window_view,
      screen_descriptor,
      gui,
    );

    // Submit will accept anything that implements IntoIter
    queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
  }

  pub fn resize(&mut self, device: &wgpu::Device, viewport: &Viewport) {
    self.screen.resize(device, viewport, &self.texture_bind_group_layout);
    //self.depth_texture = texture::Texture::create_depth_texture(device, &viewport.config, "depth_texture");
  }
}
