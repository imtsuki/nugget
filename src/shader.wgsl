@vertex
fn vertex_main(@location(0) position: vec4<f32>) -> @builtin(position) vec4<f32> {
    return position;
}

@fragment
fn fragment_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
