pub struct Material {
    pub name: Option<String>,
    pub base_color_factor: [f32; 4],
    pub base_color_texture_index: Option<usize>,
    pub normal_texture_index: Option<usize>,
}
