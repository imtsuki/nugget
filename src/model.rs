use crate::entity::Entity;
use crate::material::Material;
use crate::mesh::Mesh;
use crate::uniform::{EntityBinding, UniformsArray};
use crate::vertex::VertexAttribute;

#[derive(Debug)]
pub struct Model {
    pub root_entity: Entity,
    pub entities: Vec<Entity>,
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub uniforms: UniformsArray<EntityBinding>,
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
                    has_dynamic_offset: true,
                    min_binding_size: None,
                },
                count: None,
            }],
        };

    pub fn new(
        meshes: Vec<Mesh>,
        materials: Vec<Material>,
        entities: Vec<Entity>,
        root_entity: Entity,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let uniforms = UniformsArray::new(entities.len(), device, layout);

        let model = Self {
            root_entity,
            entities,
            meshes,
            materials,
            uniforms,
        };

        model.calculate_uniforms(&model.root_entity, model.root_entity.transform, queue);

        model
    }

    fn calculate_uniforms(
        &self,
        entity: &Entity,
        parent_transform: glam::Mat4,
        queue: &wgpu::Queue,
    ) {
        for &index in &entity.children {
            let entity = &self.entities[index];

            let transform = parent_transform * entity.transform;

            let data = EntityBinding { transform };

            self.uniforms.update(data, index, queue);

            self.calculate_uniforms(entity, transform, queue);
        }
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.render_impl(&self.root_entity, render_pass)
    }

    fn render_impl<'a>(&'a self, entity: &Entity, render_pass: &mut wgpu::RenderPass<'a>) {
        for &index in &entity.children {
            let entity = &self.entities[index];

            if let Some(mesh_index) = entity.mesh_index {
                render_pass.set_bind_group(
                    Model::BIND_GROUP_INDEX,
                    &self.uniforms.bind_group,
                    &[self.uniforms.offset(index) as _],
                );

                let mesh = &self.meshes[mesh_index];

                for primitive in &mesh.primitives {
                    let material = &self.materials[primitive.material_index];
                    render_pass.set_bind_group(
                        Material::BIND_GROUP_INDEX,
                        &material.bind_group,
                        &[],
                    );

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

            self.render_impl(entity, render_pass);
        }
    }
}
