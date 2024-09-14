
struct Camera {
    projection_matrix: mat4x4<f32>,
    view_matrix: mat4x4<f32>,
    position: vec3<f32>,
}

@group(0) @binding(0) var<uniform> camera: Camera;

@group(1) @binding(0) var t_albedo: texture_2d<f32>;
@group(1) @binding(1) var s_albedo: sampler;
@group(1) @binding(2) var t_normal: texture_2d<f32>;
@group(1) @binding(3) var s_normal: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coord: vec2<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
}

@vertex
fn vertex_main(
    vertex: VertexInput,
) -> VertexOutput {
    let world_position = vec4(vertex.position, 1.0);
    let clip_position = camera.projection_matrix * camera.view_matrix * world_position;

    return VertexOutput(clip_position, vertex.tex_coord, vertex.normal, world_position.xyz);
}

struct FragmentOutput {
    @location(0) albedo: vec4<f32>,
    @location(1) position: vec4<f32>,
    @location(2) normal: vec4<f32>,
}

@fragment
fn fragment_main(vertex: VertexOutput) -> FragmentOutput {
    // let albedo: vec4<f32> = textureSample(t_albedo, s_albedo, vertex.tex_coord);
    let albedo = vec4(0.1, 0.2, 0.3, 1.0);  // A solid color.

    let position = vec4(vertex.world_position, 1.0);

    // let normal = textureSample(t_normal, s_normal, vertex.tex_coord);
    let normal = vec4(vertex.world_normal, 1.0);  // Flat normals.

    return FragmentOutput(albedo, position, normal);
}
