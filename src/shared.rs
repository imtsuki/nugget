#[repr(C)]
pub struct VertexIn {
    position: [f32; 3],
}

#[repr(C)]
pub struct FragmentIn {
    position: [f32; 4],
}