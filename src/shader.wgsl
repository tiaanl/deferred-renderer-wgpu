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

struct PointLight {
    position: vec3<f32>,
    color: vec3<f32>,
}
@group(2) @binding(0) var<uniform> point_light: PointLight;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coord: vec2<f32>,
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

    let world_position = uniforms.model_matrix * vec4(vertex.position, 1.0);
    output.world_position = world_position.xyz;

    output.clip_position = uniforms.projection_matrix * uniforms.view_matrix * world_position;

    return output;
}

struct FragmentOutput {
    @location(0) albedo: vec4<f32>,
    @location(1) position: vec4<f32>,
    @location(2) normal: vec4<f32>,
}

@fragment
fn fragment_main(vertex: VertexOutput) -> FragmentOutput {
    // Depth
    let depth = vertex.clip_position.z;

    let light_dir = normalize(point_light.position - vertex.world_position);
    let diffuse_strength = max(dot(vertex.world_normal, light_dir), 0.0);
    let diffuse_color = point_light.color * diffuse_strength;

    // Albedo
    let object_color: vec4<f32> = textureSample(t_albedo, s_albedo, vertex.tex_coord);
    let ambient_strength = 0.1;
    let ambient_color = point_light.color * ambient_strength;
    let result = (ambient_color + diffuse_color) * object_color.xyz;
    let albedo = vec4<f32>(result, object_color.a);

    // Position
    let position = vec4(vertex.world_position, 1.0);

    // Normal
    let normal = textureSample(t_normal, s_normal, vertex.tex_coord);

    // 
    return FragmentOutput(albedo, position, normal);
}
