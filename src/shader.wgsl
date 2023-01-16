struct VertexIn {
    @location(0) position: vec3<f32>,
}

struct FragmentIn {
    @builtin(position) position: vec4<f32>,
}

@vertex
fn vertex_main(vertex_in: VertexIn) -> FragmentIn {
    let position = vec4<f32>(vertex_in.position, 1.0);
    return FragmentIn(position);
}

@fragment
fn fragment_main(fragment_in: FragmentIn) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
