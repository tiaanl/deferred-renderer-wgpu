const PI: f32 = 3.14159265359;

@group(0) @binding(0) var t_depth: texture_depth_2d;
@group(0) @binding(1) var t_albedo: texture_2d<f32>;
@group(0) @binding(2) var s_albedo: sampler;
@group(0) @binding(3) var t_position: texture_2d<f32>;
@group(0) @binding(4) var s_position: sampler;
@group(0) @binding(5) var t_normal: texture_2d<f32>;
@group(0) @binding(6) var s_normal: sampler;

struct Camera {
    projection_matrix: mat4x4<f32>,
    view_matrix: mat4x4<f32>,
    position: vec3<f32>,
}
@group(1) @binding(0) var<uniform> camera: Camera;

struct PointLight {
    position: vec3<f32>,
    intensity: f32,
    color: vec3<f32>,
    shininess: f32,
    ambient: f32,
}
@group(2) @binding(0) var<uniform> point_light: PointLight;

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

fn fresnel_schlick(h_dot_v: f32, base_reflectivity: vec3<f32>) -> vec3<f32> {
    return base_reflectivity + (1.0 - base_reflectivity) * pow(1.0 - h_dot_v, 5.0);
}

fn distribution_ggx(n_dot_h: f32, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    var denom = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
    denom = PI * denom * denom;
    return a2 / max(denom, 0.000001);
}

fn geometry_smith(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    let ggx1 = n_dot_v / (n_dot_v * (1.0 - k) + k);
    let ggx2 = n_dot_l / (n_dot_l * (1.0 - k) + k);
    return ggx1 + ggx2;
}

@fragment
fn fragment_debug(vertex_output: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_albedo, s_albedo, vertex_output.tex_coord);
}

fn diffuse(
    intensity: f32,
    color: vec3<f32>,
    direction_to_light: vec3<f32>,
    surface_normal: vec3<f32>,
) -> vec3<f32> {
    let radiance = dot(direction_to_light, surface_normal);
    return color * intensity * max(radiance, 0.0);
}

fn specular(
    intensity: f32,
    color: vec3<f32>,
    direction_to_camera: vec3<f32>,
    direction_to_light_reflected: vec3<f32>,
    shininess: f32,
) -> vec3<f32> {
    let x = dot(direction_to_light_reflected, direction_to_camera);
    let radiance = pow(x, shininess);
    return color * intensity * max(radiance, 0.0);
}

@fragment
fn fragment_main(vertex_output: VertexOutput) -> @location(0) vec4<f32> {
    let fullscreen_uv = vec2<i32>(floor(vertex_output.position.xy));
    let depth = textureLoad(t_depth, fullscreen_uv, 0);

    if depth >= 1.0 {
        // Black background for infinite depth.
        return vec4(0.0, 0.0, 0.0, 1.0);
    }

    let world_position = textureLoad(t_position, fullscreen_uv, 0).xyz;
    let world_normal = normalize(textureLoad(t_normal, fullscreen_uv, 0).xyz);

    let direction_to_light = normalize(point_light.position - world_position);
    let direction_to_camera = normalize(camera.position - world_position);

    let material_color = vec3(0.8, 0.1, 0.1);

    let diffuse = diffuse(
        point_light.intensity,
        material_color,
        direction_to_light,
        world_normal,
    );

    // Phong model
    let r = reflect(-direction_to_light, world_normal);
    // Blinn model
    // let r = normalize(direction_to_light + direction_to_camera);

    let specular = specular(
        point_light.intensity,
        material_color,
        direction_to_camera,
        r,
        point_light.shininess,
    );

    let ambient = material_color * point_light.ambient;

    return vec4(diffuse + specular + ambient, 1.0);

    /*
    let roughness = 0.1;
    let metallic = 0.1;

    let depth = textureLoad(t_depth, vec2<i32>(floor(vertex_output.position.xy)), 0);

    if depth >= 1.0 {
        // Black background for infinite depth.
        return vec4(0.0, 0.0, 0.0, 1.0);
    }

    let fragment_position = textureSample(t_position, s_position, vertex_output.tex_coord).xyz;
    let camera_position = camera.position;
    let albedo = textureSample(t_albedo, s_albedo, vertex_output.tex_coord).xyz;

    let n = textureSample(t_normal, s_normal, vertex_output.tex_coord).xyz;
    let v = normalize(camera.position - fragment_position);

    let l = normalize(point_light.position - fragment_position);
    let h = normalize(v + l);

    let distance = length(point_light.position - fragment_position);
    let attenuation = 1.0 / (distance * distance);
    let radiance = point_light.color * attenuation;

    let n_dot_v = max(dot(n, v), 0.000001);
    let n_dot_l = max(dot(n, l), 0.000001);
    let h_dot_v = max(dot(h, v), 0.0);
    let n_dot_h = max(dot(n, h), 0.0);

    let d = distribution_ggx(n_dot_h, roughness);
    let g = geometry_smith(n_dot_v, n_dot_l, roughness);
    let base_reflectivity = vec3(0.9, 0.9, 0.9);
    let f = fresnel_schlick(h_dot_v, base_reflectivity);

    var specular = d * g * f;
    specular /= 4.0 * n_dot_v * n_dot_l;

    var kd = vec3(1.0, 1.0, 1.0) - f;
    kd *= 1.0 - metallic;

    let lo = (kd * albedo / PI + specular) * radiance * n_dot_l;

    let ambient = vec3(0.1, 0.1, 0.1) * albedo;

    let color = ambient + lo;

    // TODO: hdr tonemapping
    // TODO: gamma correct

    return vec4(color, 1.0);

    // albedo value
    // return textureSample(t_albedo, s_albedo, vertex_output.tex_coord);
    */
}
