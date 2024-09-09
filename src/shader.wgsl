
struct Camera {
    projection_matrix: mat4x4<f32>,
    view_matrix: mat4x4<f32>,
    position: vec3<f32>,
}

@group(0) @binding(0) var<uniform> camera: Camera;

@group(1) @binding(0) var t_diffuse: texture_2d<f32>;
@group(1) @binding(1) var s_diffuse: sampler;
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
    var output: VertexOutput;

    output.tex_coord = vertex.tex_coord;

    output.world_normal = vertex.normal;

    let world_position = vec4(vertex.position, 1.0);
    output.world_position = world_position.xyz;

    output.clip_position = camera.projection_matrix * camera.view_matrix * world_position;

    return output;
}

struct FragmentOutput {
    @location(0) albedo: vec4<f32>,
    @location(1) position: vec4<f32>,
    @location(2) normal: vec4<f32>,
}

@fragment
fn fragment_main(vertex: VertexOutput) -> FragmentOutput {
    // Albedo
    let albedo: vec4<f32> = textureSample(t_diffuse, s_diffuse, vertex.tex_coord);

    // Position
    let position = vec4(vertex.world_position, 1.0);

    // Normal
    let normal = textureSample(t_normal, s_normal, vertex.tex_coord);
    // let normal = vec4(vertex.world_normal, 1.0);

    return FragmentOutput(albedo, position, normal);
}
