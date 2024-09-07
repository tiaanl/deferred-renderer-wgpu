@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

@vertex
fn vertex_main(
    @builtin(vertex_index) vertex_index: u32
) -> VertexOutput {
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
    return textureSample(t_diffuse, s_diffuse, vertex_output.tex_coord);
    //return vec4<f32>(vertex_output.tex_coord, 0.0, 1.0);
}
