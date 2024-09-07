struct Uniforms {
    projection_matrix: mat4x4<f32>,
    projection_inv_matrix: mat4x4<f32>,
    view_matrix: mat4x4<f32>,
    model_matrix: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var t_albedo: texture_2d<f32>;
@group(1) @binding(1) var s_albedo: sampler;
@group(1) @binding(2) var t_normal: texture_2d<f32>;
@group(1) @binding(3) var s_normal: sampler;


struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coord: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec4<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coord: vec2<f32>,
}

@vertex
fn vertex_main(
    vertex: VertexInput,
) -> VertexOutput {
    var output: VertexOutput;

    output.clip_position = 
        uniforms.projection_matrix *
        uniforms.view_matrix *
        uniforms.model_matrix *
        vec4(vertex.position, 1.0);
    output.position =
        // uniforms.view_matrix *
        uniforms.model_matrix *
        vec4(vertex.position, 1.0);
    output.normal = vertex.normal;
    output.tex_coord = vertex.tex_coord;

    return output;
}

struct FragmentOutput {
    @location(0) albedo: vec4<f32>,
    @location(1) position: vec4<f32>,
    @location(2) normal: vec4<f32>,
}

@fragment
fn fragment_main(vertex: VertexOutput) -> FragmentOutput {
    let depth = vertex.clip_position.z;

    let albedo = textureSample(t_albedo, s_albedo, vertex.tex_coord);
    let position = vertex.position;
    // let normal = vec4<f32>(normalize(vertex.normal.xyz), 1.0);
    let normal = textureSample(t_normal, s_normal, vertex.tex_coord);

    return FragmentOutput(albedo, position, normal);
}
