@group(0) @binding(0) var<uniform> u_screen_size: vec2<f32>;

@group(1) @binding(0) var t_font: texture_2d<f32>;
@group(1) @binding(1) var s_font: sampler;

fn screen_space_to_clip_space(position: vec2<f32>, screen_size: vec2<f32>) -> vec2<f32> {
    return vec2(
        2.0 * position.x / screen_size.x - 1.0,
        1.0 - 2.0 * position.y / screen_size.y,
    );
}

fn unpack_color(color: u32) -> vec4<f32> {
    return vec4<f32>(
        f32(color & 255u),
        f32((color >> 8u) & 255u),
        f32((color >> 16u) & 255u),
        f32((color >> 24u) & 255u),
    ) / 255.0;
}

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) color: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) color: vec4<f32>,
}

@vertex
fn vertex_main(input: VertexInput, @builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let position = vec4(screen_space_to_clip_space(input.position, u_screen_size), 0.0, 1.0);
    let color = unpack_color(input.color);
    return VertexOutput(position, input.tex_coord, color);
}

@fragment
fn fragment_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let texel = textureSample(t_font, s_font, vertex.tex_coord);
    return vec4(vertex.color.xyz, texel.a);
}
