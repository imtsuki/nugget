use crate::material::Material;
use crate::resources;
use crate::uniform::{ModelBinding, Uniforms};
use crate::vertex::VertexAttribute;

use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub uniforms: Uniforms<ModelBinding>,
}

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

impl Model {
    pub const BIND_GROUP_INDEX: u32 = 1;

    pub const BIND_GROUP_LAYOUT_DESCRIPTOR: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            label: Some("Model Uniforms Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        };

    pub fn new(
        meshes: Vec<Mesh>,
        materials: Vec<Material>,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let uniforms = Uniforms::new(ModelBinding::new(), device, layout);

        Self {
            meshes,
            materials,
            uniforms,
        }
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        for mesh in &self.meshes {
            for primitive in &mesh.primitives {
                let material = &self.materials[primitive.material_index];
                render_pass.set_bind_group(Material::BIND_GROUP_INDEX, &material.bind_group, &[]);

                render_pass.set_vertex_buffer(
                    VertexAttribute::Position.location(),
                    primitive.positions.slice(..),
                );
                render_pass.set_vertex_buffer(
                    VertexAttribute::TexCoord.location(),
                    primitive.tex_coords.slice(..),
                );
                render_pass.set_vertex_buffer(
                    VertexAttribute::Normal.location(),
                    primitive.normals.slice(..),
                );

                render_pass
                    .set_index_buffer(primitive.indices.slice(..), wgpu::IndexFormat::Uint32);

                // TODO: stride?
                render_pass.draw_indexed(0..(primitive.indices.size() / 4) as u32, 0, 0..1);
            }
        }
    }
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
