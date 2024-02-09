use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Buffer, Device};
////////////////////////////////////////////////////////////////////////////////
//                ██╗   ██╗███╗   ██╗███████╗███╗   ██╗                       //
//                ██║   ██║████╗  ██║██╔════╝████╗  ██║                       //
//                ██║   ██║██╔██╗ ██║█████╗  ██╔██╗ ██║                       //
//                ██║   ██║██║╚██╗██║██╔══╝  ██║╚██╗██║                       //
//                ╚██████╔╝██║ ╚████║███████╗██║ ╚████║                       //
//                 ╚═════╝ ╚═╝  ╚═══╝╚══════╝╚═╝  ╚═══╝ LIB                   //
////////////////////////////////////////////////////////////////////////////////
// ? Defines the camera.
use winit::keyboard::KeyCode;
use crate::event::event::Event;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
  view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
  pub fn new() -> Self {
    use cgmath::SquareMatrix;
    Self {
      view_proj: cgmath::Matrix4::identity().into(),
    }
  }

  pub fn update_view_proj(&mut self, camera: &mut Camera) {
    self.view_proj = camera.build_view_projection_matrix().into();
  }
}

pub struct CameraDescriptor {
  pub speed: f32,
  pub fovy: f32,
  pub near: f32,
  pub far: f32,
  pub v_width: f32,
  pub v_height: f32,
}

pub struct Camera {
  pub desc: CameraDescriptor,
  pub eye: cgmath::Point3<f32>,
  pub target: cgmath::Point3<f32>,
  pub up: cgmath::Vector3<f32>,
  pub aspect: f32,
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
  1.0, 0.0, 0.0, 0.0,
  0.0, 1.0, 0.0, 0.0,
  0.0, 0.0, 0.5, 0.5,
  0.0, 0.0, 0.0, 1.0,
);

impl Camera {
  pub fn build_view_projection_matrix(&mut self) -> cgmath::Matrix4<f32> {
    let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
    self.aspect = self.desc.v_width / self.desc.v_height;
    let proj = cgmath::perspective(cgmath::Deg(self.desc.fovy), self.aspect, self.desc.near, self.desc.far);
    return OPENGL_TO_WGPU_MATRIX * proj * view;
  }
}

pub struct CameraController {
  is_forward_pressed: bool,
  is_backward_pressed: bool,
  is_left_pressed: bool,
  is_right_pressed: bool,
  pub camera: Camera,
  pub uniform: CameraUniform,
  pub buffer: Buffer,
  pub bind_group_layout: BindGroupLayout,
  pub bind_group: BindGroup,
}

impl CameraController {
  /// Creates a new camera controller and initializes a camera with the passed
  /// descriptor
  pub fn new(device: &Device, desc: CameraDescriptor) -> Self {
    let mut camera = Camera {
      eye: (0.0, 1.0, 2.0).into(),
      target: (0.0, 0.0, 0.0).into(),
      up: cgmath::Vector3::unit_y(),
      aspect: (desc.v_width / desc.v_height),
      desc,
    };

    let mut uniform = CameraUniform::new();
    uniform.update_view_proj(&mut camera);

    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Camera Buffer"),
      contents: bytemuck::cast_slice(&[uniform]),
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      }
    );

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("camera_bind_group_layout"),
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::VERTEX,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None
          },
          count: None,
        },
      ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("camera_bind_group"),
      layout: &bind_group_layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: buffer.as_entire_binding(),
        }
      ]
    });

    Self {
      is_forward_pressed: false,
      is_backward_pressed: false,
      is_left_pressed: false,
      is_right_pressed: false,
      camera,
      uniform,
      buffer,
      bind_group_layout,
      bind_group,
    }
  }

  /// Process the events passed and returns true if the event got consumed,
  /// otherwise returns false
  pub fn process_events(&mut self, event: &Event) -> bool {
    match event {
      Event::KeyboardInput {
        key,
        is_pressed } => {
          match key {
            KeyCode::KeyW | KeyCode::ArrowUp => {
              self.is_forward_pressed = *is_pressed;
              true
            },
            KeyCode::KeyA | KeyCode::ArrowLeft => {
              self.is_left_pressed = *is_pressed;
              true
            },
            KeyCode::KeyS | KeyCode::ArrowDown => {
              self.is_backward_pressed = *is_pressed;
              true
            },
            KeyCode::KeyD | KeyCode::ArrowRight => {
              self.is_right_pressed = *is_pressed;
              true
            },
            _ => false
          }
      },
      _ => false
    }
  }

  pub fn update_viewport(&mut self, width: u32, height: u32) {
    self.camera.desc.v_width = width as f32;
    self.camera.desc.v_height = height as f32;
  }

  pub fn update(&mut self) {
    use cgmath::InnerSpace;
    let forward = self.camera.target - self.camera.eye;
    let forward_norm = forward.normalize();
    let forward_mag = forward.magnitude();

    if self.is_forward_pressed && forward_mag > self.camera.desc.speed {
      self.camera.eye += forward_norm * self.camera.desc.speed;
    }
    if self.is_backward_pressed {
      self.camera.eye -= forward_norm * self.camera.desc.speed;
    }

    let right = forward_norm.cross(self.camera.up);

    let forward = self.camera.target - self.camera.eye;
    let forward_mag = forward.magnitude();

    if self.is_right_pressed {
      self.camera.eye = self.camera.target - (forward + right * self.camera.desc.speed).normalize() * forward_mag;
    }
    if self.is_left_pressed {
      self.camera.eye = self.camera.target - (forward - right * self.camera.desc.speed).normalize() * forward_mag;
    }

    self.uniform.update_view_proj(&mut self.camera);
  }
}
