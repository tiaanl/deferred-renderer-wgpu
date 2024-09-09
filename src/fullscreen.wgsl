@group(0) @binding(0) var t_albedo: texture_2d<f32>;
@group(0) @binding(1) var s_albedo: sampler;
@group(0) @binding(2) var t_position: texture_2d<f32>;
@group(0) @binding(3) var s_position: sampler;
@group(0) @binding(4) var t_normal: texture_2d<f32>;
@group(0) @binding(5) var s_normal: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

@vertex
fn vertex_main(
    @builtin(vertex_index) vertex_index: u32
) -> VertexOutput {
    // Create a fullscreen texture.
    let tex_coord = vec2<f32>(
        f32(vertex_index >> 1u),
        f32(vertex_index & 1u)
    ) * 2.0;
    let position = vec4<f32>(
        tex_coord * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0),
        0.0,
        1.0
    );

    return VertexOutput(position, tex_coord);
}

@fragment
fn fragment_main(vertex_output: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_albedo, s_albedo, vertex_output.tex_coord);
}
