struct VertexIn {
    @location(0) position: vec4<f32>,
}

struct FragmentIn {
    @builtin(position) position: vec4<f32>,
}

@vertex
fn vertex_main(vertex_in: VertexIn) -> FragmentIn {
    let position = vertex_in.position;
    return FragmentIn(position * vec4<f32>(0.5, 0.5, 0.5, 1.0));
}

@fragment
fn fragment_main(fragment_in: FragmentIn) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
