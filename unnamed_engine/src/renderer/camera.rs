//! ## Camera
//!
//! Define the camera that UnnamedEngine uses.
use std::{f32::consts::FRAC_2_PI, time::Duration};

use crate::event::event::Event;
use cgmath::{perspective, InnerSpace, Matrix4, Point3, Rad, Vector3};
use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Buffer, Device};
use winit::keyboard::KeyCode;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

const SAFE_FRAC_PI_2: f32 = FRAC_2_PI - 0.0001;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
  view_position: [f32; 4],
  view_projection: [[f32; 4]; 4],
}

impl CameraUniform {
  pub fn new() -> Self {
    use cgmath::SquareMatrix;
    Self {
      view_position: [0.0; 4],
      view_projection: cgmath::Matrix4::identity().into(),
    }
  }

  pub fn update_view_proj(&mut self, camera: &Camera, projection: &Projection) {
    self.view_position = camera.position.to_homogeneous().into();
    self.view_projection = (projection.calc_matrix() * camera.calc_matrix()).into();
  }
}

impl Default for CameraUniform {
  fn default() -> Self {
    Self::new()
  }
}

pub struct Projection {
  aspect: f32,
  fovy: Rad<f32>,
  znear: f32,
  zfar: f32,
}

impl Projection {
  pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
    Self {
      aspect: width as f32 / height as f32,
      fovy: fovy.into(),
      znear,
      zfar,
    }
  }

  pub fn resize(&mut self, width: u32, height: u32) {
    self.aspect = width as f32 / height as f32;
  }

  pub fn calc_matrix(&self) -> Matrix4<f32> {
    OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
  }
}

pub struct Camera {
  pub position: Point3<f32>,
  yaw: Rad<f32>,
  pitch: Rad<f32>,
}

impl Camera {
  pub fn new<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
    position: V,
    yaw: Y,
    pitch: P,
  ) -> Self {
    Self {
      position: position.into(),
      yaw: yaw.into(),
      pitch: pitch.into(),
    }
  }

  pub fn calc_matrix(&self) -> Matrix4<f32> {
    let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
    let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

    Matrix4::look_to_rh(
      self.position,
      Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
      Vector3::unit_y(),
    )
  }
}

pub struct CameraController {
  amount_left: f32,
  amount_right: f32,
  amount_forward: f32,
  amount_backward: f32,
  amount_up: f32,
  amount_down: f32,
  rotate_horizontal: f32,
  rotate_vertical: f32,
  scroll: f32,
  speed: f32,
  sensitivity: f32,
  pub camera: Camera,
  pub uniform: CameraUniform,
  pub projection: Projection,
  pub buffer: Buffer,
  pub bind_group_layout: BindGroupLayout,
  pub bind_group: BindGroup,
}

impl CameraController {
  /// Creates a new camera controller and initializes a camera with the passed
  /// descriptor
  pub fn new(device: &Device, speed: f32, sensitivity: f32, width: u32, height: u32) -> Self {
    let camera = Camera::new((0.0, 2.0, 5.0), cgmath::Deg(-90.0), cgmath::Deg(-45.0));

    let projection = Projection::new(width, height, cgmath::Deg(45.0), 0.05, 1000.0);

    let mut uniform = CameraUniform::new();
    uniform.update_view_proj(&camera, &projection);

    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Camera Buffer"),
      contents: bytemuck::cast_slice(&[uniform]),
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("camera_bind_group_layout"),
      entries: &[wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::VERTEX,
        ty: wgpu::BindingType::Buffer {
          ty: wgpu::BufferBindingType::Uniform,
          has_dynamic_offset: false,
          min_binding_size: None,
        },
        count: None,
      }],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("camera_bind_group"),
      layout: &bind_group_layout,
      entries: &[wgpu::BindGroupEntry {
        binding: 0,
        resource: buffer.as_entire_binding(),
      }],
    });

    Self {
      camera,
      uniform,
      projection,
      buffer,
      bind_group_layout,
      bind_group,
      amount_left: 0.0,
      amount_right: 0.0,
      amount_forward: 0.0,
      amount_backward: 0.0,
      amount_up: 0.0,
      amount_down: 0.0,
      rotate_horizontal: 0.0,
      rotate_vertical: 0.0,
      scroll: 0.0,
      speed,
      sensitivity,
    }
  }

  /// Process the events passed and returns true if the event got consumed,
  /// otherwise returns false
  pub fn process_events(&mut self, event: Event) -> bool {
    match event {
      Event::KeyboardInput { key, is_pressed } => {
        let amount = if is_pressed { 1.0 } else { 0.0 };
        match key {
          KeyCode::KeyW | KeyCode::ArrowUp => {
            self.amount_forward = amount;
            true
          }
          KeyCode::KeyS | KeyCode::ArrowDown => {
            self.amount_backward = amount;
            true
          }
          KeyCode::KeyA | KeyCode::ArrowLeft => {
            self.amount_left = amount;
            true
          }
          KeyCode::KeyD | KeyCode::ArrowRight => {
            self.amount_right = amount;
            true
          }
          KeyCode::Space => {
            self.amount_up = amount;
            true
          }
          KeyCode::ShiftLeft => {
            self.amount_down = amount;
            true
          }
          _ => false,
        }
      }
      Event::MouseMotion { x, y } => {
        self.rotate_horizontal = x;
        self.rotate_vertical = y;
        true
      }
      Event::MouseScroll { delta } => {
        self.scroll = delta.1;
        true
      }
      _ => false,
    }
  }

  pub fn update(&mut self, dt: Duration) {
    let dt = dt.as_secs_f32();

    // Move forward/backward and left/right
    let (yaw_sin, yaw_cos) = self.camera.yaw.0.sin_cos();
    let pitch = self.camera.pitch.0;
    let forward = Vector3::new(yaw_cos, pitch, yaw_sin).normalize();
    let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
    self.camera.position +=
      forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
    self.camera.position += right * (self.amount_right - self.amount_left) * self.speed * dt;

    // Move in/out (aka. "zoom")
    // Note: this isn't an actual zoom. The camera's position changes when zooming.
    let (pitch_sin, pitch_cos) = self.camera.pitch.0.sin_cos();
    let scrollward = Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
    self.camera.position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
    self.scroll = 0.0;

    // Move up/down. Since we don't use scroll, we can just modify the y coordinate directly
    self.camera.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

    // Rotate
    self.camera.yaw += Rad(self.rotate_horizontal) * self.sensitivity * dt;
    self.camera.pitch += Rad(-self.rotate_vertical) * self.sensitivity * dt;

    // If process_events ins't called every frame, these values will not get set to zero, and the
    // camera will rotate when moving in a non-cardinal direction.
    self.rotate_horizontal = 0.0;
    self.rotate_vertical = 0.0;

    // Keep the camera's angle from going too high/low.
    if self.camera.pitch < -Rad(SAFE_FRAC_PI_2) {
      self.camera.pitch = -Rad(SAFE_FRAC_PI_2);
    } else if self.camera.pitch > Rad(SAFE_FRAC_PI_2) {
      self.camera.pitch = Rad(SAFE_FRAC_PI_2);
    }

    self
      .uniform
      .update_view_proj(&self.camera, &self.projection);
  }

  pub fn resize(&mut self, width: u32, height: u32) {
    self.projection.resize(width, height);
  }
}
