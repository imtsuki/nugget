pub struct Node {
    pub name: Option<String>,
    pub mesh_index: Option<usize>,
    pub children: Vec<usize>,
    pub transform: [[f32; 4]; 4],
}
