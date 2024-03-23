//! ## Middleware Renderer
//!
//! Defines a middleware that stores and executes everything related to the graphics library.
use cgmath::Rotation3;
use egui_wgpu::ScreenDescriptor;
use wgpu::util::DeviceExt;

use crate::{gui::{egui_renderer::EguiRenderer, gui::gui}, voxel::{rendering::{ChunkMesh, Vertex}, Chunk, CHUNK_AREA, CHUNK_SIZE, CHUNK_VOLUME}};

use super::{
  camera::CameraController, material::Material, screen::Screen, texture::{self, Texture}, transform::{Transform, TransformRaw}, viewport::Viewport
};

pub struct MiddlewareRenderer {
  texture_bind_group_layout: wgpu::BindGroupLayout,
  material: Material,
  pipeline: wgpu::RenderPipeline,
  transforms: Vec<Transform>,
  instance_buffer: wgpu::Buffer,
  screen: Screen,

  test_texture: Texture,
  chunk_mesh: ChunkMesh,
}

impl MiddlewareRenderer {
  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    camera_controller: &CameraController,
    viewport: &Viewport,
  ) -> Self {
    let texture_bind_group_layout =
      device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

    let test_bytes = include_bytes!("../../assets/textures/dirt.png");
    let mut test_texture =
      texture::Texture::from_bytes(device, queue, test_bytes, "dirt.png").unwrap();
    test_texture.set_bind_group(device, &texture_bind_group_layout);

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("pipeline_layout"),
      bind_group_layouts: &[
        &texture_bind_group_layout,
        &camera_controller.bind_group_layout,
      ],
      push_constant_ranges: &[],
    });

    let material = Material::from_string(&device, include_str!("../shader.wgsl").into());

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("pipeline"),
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: material.shader(),
        entry_point: "vs_main",
        buffers: &[Vertex::desc(), TransformRaw::desc()],
      },
      fragment: Some(wgpu::FragmentState {
        module: material.shader(),
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
        cull_mode: Some(wgpu::Face::Back),
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


    let mut voxels = [0; CHUNK_VOLUME];
    for x in 0..CHUNK_SIZE - 1 {
      for z in 0..CHUNK_SIZE - 1 {
        voxels[(x * CHUNK_AREA) + z] = 1;
      }
    }

    let chunk = Chunk::new(Default::default(), voxels);
    let mut transforms: Vec<Transform> = Vec::new();
    let position = cgmath::Vector3 {
      x: 0.0,
      y: 0.0,
      z: 0.0,
    };

    let rotation = cgmath::Quaternion::from_angle_x(cgmath::Deg(0.0));

    let scale = cgmath::Vector3 {
      x: 1.0,
      y: 1.0,
      z: 1.0,
    };

    transforms.push(
      Transform {
        position,
        rotation,
        scale,
      });

    let chunk_mesh = ChunkMesh::new(device, &chunk);

    let instance_data = transforms.iter().map(Transform::to_raw).collect::<Vec<_>>();
    let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("instance_buffer"),
      contents: bytemuck::cast_slice(&instance_data),
      usage: wgpu::BufferUsages::VERTEX,
    });

    Self {
      texture_bind_group_layout,
      material,
      pipeline,
      transforms,
      instance_buffer,
      screen,
      test_texture,
      chunk_mesh,
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
    let window_view = output
      .texture
      .create_view(&wgpu::TextureViewDescriptor::default());

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
          }),
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
      render_pass.set_vertex_buffer(0, self.chunk_mesh.vertex_buffer.slice(..));

      render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
      render_pass.set_index_buffer(self.chunk_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

      render_pass.draw_indexed(0..self.chunk_mesh.indices, 0, 0..self.transforms.len() as _);
    }

    self.screen.draw(&mut encoder, &window_view);

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
    self
      .screen
      .resize(device, viewport, &self.texture_bind_group_layout);
    //self.depth_texture = texture::Texture::create_depth_texture(device, &viewport.config, "depth_texture");
  }
}
