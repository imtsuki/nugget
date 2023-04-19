use crate::resources;

#[derive(Debug)]
pub struct Entity {
    pub name: Option<String>,
    pub mesh_index: Option<usize>,
    pub children: Vec<usize>,
    pub transform: glam::Mat4,
}

impl From<resources::Node> for Entity {
    fn from(node: resources::Node) -> Self {
        Self {
            name: node.name,
            mesh_index: node.mesh_index,
            children: node.children,
            transform: node.transform,
        }
    }
}
