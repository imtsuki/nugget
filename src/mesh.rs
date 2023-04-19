use crate::resources;
use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct Mesh {
    pub name: Option<String>,
    pub primitives: Vec<Primitive>,
}

#[derive(Debug)]
pub struct Primitive {
    pub positions: wgpu::Buffer,
    pub tex_coords: wgpu::Buffer,
    pub normals: wgpu::Buffer,
    pub indices: wgpu::Buffer,
    pub material_index: usize,
}

impl Mesh {
    pub fn new(mesh: &resources::Mesh, device: &wgpu::Device) -> Mesh {
        let mut primitives = vec![];
        for (index, primitive) in mesh.primitives.iter().enumerate() {
            let debug_label = format!(
                "{:?}#{}",
                mesh.name.as_deref().unwrap_or("Unnamed Mesh"),
                index
            );

            let primitive = Primitive::new(primitive, &debug_label, device);

            primitives.push(primitive);
        }

        Mesh {
            name: mesh.name.clone(),
            primitives,
        }
    }
}

impl Primitive {
    pub fn new(
        primitive: &resources::Primitive,
        debug_label: &str,
        device: &wgpu::Device,
    ) -> Primitive {
        let positions = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Position Buffer {}", debug_label)),
            contents: bytemuck::cast_slice(&primitive.positions),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let tex_coords = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Tex Coord Buffer {}", debug_label)),
            contents: bytemuck::cast_slice(&primitive.tex_coords),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let normals = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Normal Buffer {}", debug_label)),
            contents: bytemuck::cast_slice(&primitive.normals),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Index Buffer {}", debug_label)),
            contents: bytemuck::cast_slice(&primitive.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Primitive {
            material_index: primitive.material_index,
            positions,
            tex_coords,
            normals,
            indices,
        }
    }
}
