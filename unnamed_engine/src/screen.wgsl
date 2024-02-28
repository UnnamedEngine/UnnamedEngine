// Vertex shader
struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
  @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
  screen: VertexInput,
) -> VertexOutput {
  var out: VertexOutput;
  out.tex_coords = screen.tex_coords;
  out.clip_position = vec4<f32>(screen.position, 1.0);
  return out;
}

// Fragment shader
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

fn grayscale(color: vec4<f32>) -> vec4<f32> {
  var lum = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
  return vec4<f32>(lum, lum, lum, color.a);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  var texture = textureSample(t_diffuse, s_diffuse, in.tex_coords);
  return texture;
}




