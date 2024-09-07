struct Uniforms {
    projection_matrix: mat4x4<f32>,
    view_matrix: mat4x4<f32>,
    model_matrix: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coord: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vertex_main(
    vertex: VertexInput,
) -> VertexOutput {
    var output: VertexOutput;

    output.position = 
        uniforms.projection_matrix *
        uniforms.view_matrix *
        uniforms.model_matrix *
        vec4(vertex.position, 1.0);

    output.color = vec4(vertex.position, 1.0);

    return output;
}

@fragment
fn fragment_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vertex.color;
}
