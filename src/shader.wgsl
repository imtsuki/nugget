struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec4<f32>,
}

struct FragmentIn {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_tangent: vec3<f32>,
    @location(3) world_bitangent: vec3<f32>,
}

struct CameraBinding {
    view_matrix: mat4x4<f32>,
    projection_matrix: mat4x4<f32>,
}

struct ModelBinding {
    model_matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraBinding;

@group(1) @binding(0)
var<uniform> model: ModelBinding;

@vertex
fn vertex_main(vertex_in: VertexIn) -> FragmentIn {
    let position = camera.projection_matrix
        * camera.view_matrix
        * model.model_matrix
        * vec4<f32>(vertex_in.position, 1.0);

    let normal_matrix = camera.view_matrix * model.model_matrix;

    let normal = vec4<f32>(vertex_in.normal, 0.0);
    let world_normal = normal_matrix * normal;
    let tangent = vec4<f32>(vertex_in.tangent.xyz, 0.0);
    let world_tangent = normal_matrix * tangent;
    let bitangent = vec4<f32>(cross(vertex_in.normal, vertex_in.tangent.xyz) * vertex_in.tangent.w, 0.0);
    let world_bitangent = normal_matrix * bitangent;

    return FragmentIn(position, vertex_in.tex_coord, world_normal.xyz, world_tangent.xyz, world_bitangent.xyz);
}

@group(2) @binding(0)
var<uniform> base_color_factor: vec4<f32>;
@group(2) @binding(1)
var base_color_texture: texture_2d<f32>;
@group(2) @binding(2)
var base_color_sampler: sampler;
@group(2) @binding(3)
var normal_texture: texture_2d<f32>;
@group(2) @binding(4)
var normal_sampler: sampler;

@fragment
fn fragment_main(fragment_in: FragmentIn) -> @location(0) vec4<f32> {
    // The color(s) returned from a fragment function are assumed to be in RGBA order,
    // regardless of the pixel format of the render target.

    // As per the spec, color is multiplied, in linear space, with the base color factor
    let base_color = base_color_factor * textureSample(base_color_texture, base_color_sampler, fragment_in.tex_coord);

    var normal = textureSample(normal_texture, normal_sampler, fragment_in.tex_coord).xyz;
    normal = normal * 2.0 - 1.0;
    normal = normalize(fragment_in.world_tangent * normal.x + fragment_in.world_bitangent * normal.y + fragment_in.world_normal * normal.z);

    let light_direction = vec3<f32>(-0.25, 0.5, -0.5);

    let light = normalize(light_direction);
    let normal_dot_light = max(dot(normal, light), 0.0);
    let surface_color = base_color.rgb * (0.1 + normal_dot_light);

    return vec4(surface_color, base_color.a);
}
