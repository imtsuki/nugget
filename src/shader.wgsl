struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

struct FragmentIn {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) normal: vec3<f32>,
}

struct Uniforms {
    model_matrix: mat4x4<f32>,
    view_matrix: mat4x4<f32>,
    projection_matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vertex_main(vertex_in: VertexIn) -> FragmentIn {
    let position = uniforms.projection_matrix
        * uniforms.view_matrix
        * uniforms.model_matrix
        * vec4<f32>(vertex_in.position, 1.0);
    let normal = vec4<f32>(vertex_in.normal, 0.0);
    return FragmentIn(position, vertex_in.tex_coord, vertex_in.normal);
}

@group(1) @binding(0)
var<uniform> base_color_factor: vec4<f32>;
@group(1) @binding(1)
var base_color_texture: texture_2d<f32>;
@group(1) @binding(2)
var base_color_sampler: sampler;

@fragment
fn fragment_main(fragment_in: FragmentIn) -> @location(0) vec4<f32> {
    // The color(s) returned from a fragment function are assumed to be in RGBA order,
    // regardless of the pixel format of the render target.

    // As per the spec, color is multiplied, in linear space, with the base color factor
    let base_color = base_color_factor * textureSample(base_color_texture, base_color_sampler, fragment_in.tex_coord);

    let sky = vec4<f32>(base_color.rgb, 1.0);
    let earth = vec4<f32>(base_color.rgb * 0.5, 1.0);
    let intensity = fragment_in.normal.y * 0.5 + 0.5;

    return mix(earth, sky, intensity);
}
