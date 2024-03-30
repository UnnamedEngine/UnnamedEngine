//! ## Renderer
//!
//! Defines and implements the renderer and the related structs.

use std::{error::Error, sync::Arc, time::Duration};

use cgmath::Rotation3;
use egui_wgpu::ScreenDescriptor;
use instant::Instant;
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::{core::module::Module, event::event::Event, gui::{egui_renderer::EguiRenderer, gui::gui}, voxel::{rendering::{ChunkMesh, Vertex}, Chunk, CHUNK_AREA, CHUNK_SIZE, CHUNK_VOLUME}};

use super::{
  camera::CameraController, material::Material, middleware_renderer::MiddlewareRenderer, screen::Screen, texture::{self, Texture}, transform::{Transform, TransformRaw}, viewport::Viewport
};

pub struct RenderingStats {
  pub bytes: usize,
  pub delta: Duration,
}

impl Default for RenderingStats {
  fn default() -> Self {
    Self {
      bytes: Default::default(),
      delta: Default::default(),
    }
  }
}
/// ## Renderer
///
/// A renderer is the most important wrapper around rendering, it contains the
/// required data for rendering and defines the general flow of the rendering.
pub struct Renderer {
  pub camera_controller: CameraController,
  pub egui: EguiRenderer,

  middleware: MiddlewareRenderer,
  texture_bind_group_layout: wgpu::BindGroupLayout,
  screen: Screen,
  pipeline: wgpu::RenderPipeline,
  material: Material,
  stats: RenderingStats,

  transforms: Vec<Transform>,
  instance_buffer: wgpu::Buffer,

  test_texture: Texture,
  chunk_mesh: ChunkMesh,
}

impl Module for Renderer {
  fn process_events(&mut self, event: Event) -> Result<(), Box<dyn Error>> {
    self.camera_controller.process_events(event);
    match event {
      Event::Resize { width, height } => {
        if width > 0 && height > 0 {
          self.middleware.viewport.resize(&self.middleware.device, width, height);
          self.camera_controller.resize(width, height);
          self.screen.resize(
            &self.middleware.device,
            &self.middleware.viewport,
            &self.texture_bind_group_layout,
          );

          // Request a redraw just in case
          self.middleware.viewport.desc.window.request_redraw();
        }
      }
      Event::Redraw => {
        self.middleware.viewport.desc.window.request_redraw();
      }
      _ => {}
    }

    Ok(())
  }

  fn update(&mut self, dt: Duration) -> Result<(), Box<dyn Error>> {
    self.camera_controller.update(dt);
    self.middleware.queue.write_buffer(
      &self.camera_controller.buffer,
      0,
      bytemuck::cast_slice(&[self.camera_controller.uniform]),
    );

    Ok(())
  }

  fn render(&mut self) -> Result<(), Box<dyn Error>> {
    // Delta measuring
    let delta_start = Instant::now();

    // Rendering start
    let output = self.middleware.viewport.get_current_texture();

    let renderer_view = &self.screen.diffuse_texture.view;
    let window_view = output
      .texture
      .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = self.middleware.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("render_encoder"),
    });

    // I don't really know why this needs to be inside of a inner scope, but it
    // needs...
    {
      // Will render to the `Screen`
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
      if let Some(test_texture_bind_group) = &self.test_texture.bind_group {
        render_pass.set_bind_group(0, test_texture_bind_group, &[]);
      }
      render_pass.set_bind_group(1, &self.camera_controller.bind_group, &[]);
      render_pass.set_vertex_buffer(0, self.chunk_mesh.vertex_buffer.slice(..));

      render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
      render_pass.set_index_buffer(self.chunk_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

      render_pass.draw_indexed(0..self.chunk_mesh.indices, 0, 0..self.transforms.len() as _);
    }

    // Renders the `Screen` texture into the window
    self.screen.draw(&mut encoder, &window_view);

    let screen_descriptor = ScreenDescriptor {
      size_in_pixels: [
        self.middleware.viewport.config.width,
        self.middleware.viewport.config.height,
        ],
        pixels_per_point: self.middleware.viewport.desc.window.scale_factor() as f32,
    };

    self.egui.draw(
      &self.middleware.device,
      &self.middleware.queue,
      &mut encoder,
      &self.middleware.viewport.desc.window,
      &window_view,
      screen_descriptor,
      &self.stats,
      gui,
    );

    // Submit will accept anything that implements IntoIter
    self.middleware.queue.submit(std::iter::once(encoder.finish()));
    output.present();

    // Delta measuring
    let delta_end = Instant::now();
    self.stats.delta = delta_end.duration_since(delta_start);

    Ok(())
  }

  fn viewport(&mut self) -> Option<&mut Viewport> {
    Some(&mut self.middleware.viewport)
  }

  fn process_window_events(&mut self, event: &winit::event::WindowEvent) {
    self.egui.handle_input(&self.middleware.viewport.desc.window, event);
  }
}

impl Renderer {
  pub async fn new(viewport: (Arc<Window>, wgpu::Color)) -> Self {
    let middleware = MiddlewareRenderer::new(viewport).await;

    let egui = EguiRenderer::new(
      &middleware.device,
      middleware.viewport.format,
      None,
      1,
      &middleware.viewport.desc.window,
    );

    let camera_controller = CameraController::new(
      &middleware.device,
      4.0,
      0.4,
      middleware.viewport.config.width,
      middleware.viewport.config.height,
    );

    let texture_bind_group_layout = middleware.device
      .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            // This should match the filterable field of the corresponding
            // texture entry above
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
          },
        ],
      });

    let screen = Screen::new(
      &middleware.device,
      &middleware.viewport,
      &texture_bind_group_layout,
    );

    let pipeline_layout = middleware.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("render_pipeline_layout"),
      bind_group_layouts: &[
        &texture_bind_group_layout,
        &camera_controller.bind_group_layout,
      ],
      push_constant_ranges: &[],
    });

    let material = Material::from_string(
      &middleware.device,
      String::from(include_str!("../shader.wgsl")),
    );

    let pipeline = middleware.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("render_pipeline"),
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &material.shader,
        entry_point: "vs_main",
        buffers: &[Vertex::desc(), TransformRaw::desc()],
      },
      fragment: Some(wgpu::FragmentState {
        module: &material.shader,
        entry_point: "fs_main",
        targets: &[Some(wgpu::ColorTargetState {
          format: middleware.viewport.format,
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

    // TODO remove this section later
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

    let mut stats = RenderingStats::default();

    let chunk_mesh = ChunkMesh::new(&middleware.device, &chunk, &mut stats);

    let instance_data = transforms.iter().map(Transform::to_raw).collect::<Vec<_>>();
    let instance_buffer = middleware.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("instance_buffer"),
      contents: bytemuck::cast_slice(&instance_data),
      usage: wgpu::BufferUsages::VERTEX,
    });

    let mut test_texture = texture::Texture::from_bytes(
      &middleware.device,
      &middleware.queue,
      include_bytes!("../../assets/textures/dirt.png"),
      "dirt.png")
      .unwrap();
    test_texture.set_bind_group(&middleware.device, &texture_bind_group_layout);

    Self {
      camera_controller,
      egui,
      middleware,
      texture_bind_group_layout,
      screen,
      pipeline,
      material,
      stats,

      transforms,
      instance_buffer,

      test_texture,
      chunk_mesh,
    }
  }
}
