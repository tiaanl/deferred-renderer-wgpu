struct Camera {
    projection_matrix: mat4x4<f32>,
    view_matrix: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) obj_position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) world_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex fn vertex_main(vertex: VertexInput) -> VertexOutput {
    let world_position = camera.projection_matrix * camera.view_matrix * vec4<f32>(vertex.position + vertex.obj_position, 1.0);

    var output: VertexOutput;
    output.world_position = world_position;
    output.color = vertex.color;
    return output;
}

@fragment fn fragment_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vertex.color;
}
