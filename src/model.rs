use crate::ext::RgbaImageExt;
use crate::material::Material;
use crate::texture::Texture;
use crate::uniform::{ModelBinding, Uniforms};
use crate::vertex::VertexAttribute;
use crate::Result;

use anyhow::anyhow;
use std::{fmt, path};
use tracing::{debug, info};
use wgpu::util::DeviceExt;

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub uniforms: Option<Uniforms<ModelBinding>>,
}

pub struct Mesh {
    pub name: Option<String>,
    pub primitives: Vec<Primitive>,
}

pub struct Primitive {
    pub positions: (Vec<[f32; 3]>, Option<wgpu::Buffer>),
    pub tex_coords: (Vec<[f32; 2]>, Option<wgpu::Buffer>),
    pub normals: (Vec<[f32; 3]>, Option<wgpu::Buffer>),
    pub indices: (Vec<u32>, Option<wgpu::Buffer>),
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

    pub fn load_gltf<P: AsRef<path::Path> + fmt::Debug>(path: P) -> Result<Model> {
        info!("Loading model from {:?}", path);
        let (gltf, buffers, images) = gltf::import(path)?;

        for buffer in &buffers {
            debug!("Found buffer of size {}", buffer.len());
        }

        let mut meshes = vec![];

        for mesh in gltf.meshes() {
            info!("Found mesh {:?}", mesh.name());

            let mut primitives = vec![];

            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                let positions = reader
                    .read_positions()
                    .map(|iter| iter.collect::<Vec<_>>())
                    .ok_or_else(|| anyhow!("No positions found"))?;

                debug!("Found {} positions", positions.len());

                let tex_coords = reader
                    .read_tex_coords(0)
                    .map(|iter| iter.into_f32().collect::<Vec<_>>())
                    .unwrap_or_else(|| {
                        debug!("No tex coords found, using default");
                        vec![[0.0, 0.0]; positions.len()]
                    });

                debug!("Found {} tex coords", tex_coords.len());

                let normals = reader
                    .read_normals()
                    .map(|iter| iter.collect::<Vec<_>>())
                    .ok_or_else(|| anyhow!("No normals found"))?;

                debug!("Found {} normals", normals.len());

                let indices = reader
                    .read_indices()
                    .map(|iter| iter.into_u32().collect::<Vec<_>>())
                    .ok_or_else(|| anyhow!("No indices found"))?;

                debug!("Found {} indices", indices.len());

                let material_index = primitive.material().index().unwrap();

                primitives.push(Primitive {
                    positions: (positions, None),
                    tex_coords: (tex_coords, None),
                    normals: (normals, None),
                    indices: (indices, None),
                    material_index,
                });
            }

            meshes.push(Mesh {
                name: mesh.name().map(str::to_owned),
                primitives,
            });
        }

        let mut materials = vec![];

        for material in gltf.materials() {
            info!("Found material {:?}", material.name());
            let pbr = material.pbr_metallic_roughness();
            let base_color_factor = pbr.base_color_factor();
            let base_color_texture = if let Some(texture_info) = pbr.base_color_texture() {
                // TODO: figure out what this is used for
                #[allow(unused_variables)]
                let tex_coord = texture_info.tex_coord();

                let texture = texture_info.texture();

                // TODO: use this sampler info
                #[allow(unused_variables)]
                let sampler = texture.sampler();

                let image = &images[texture.source().index()];

                info!("Base color texture: {:?}", texture.name());
                info!("Base color texture format: {:?}", image.format);

                let image = image::RgbaImage::from_gltf_image(image)
                    .ok_or(anyhow!("Failed to convert gltf image to rgba"))?;

                Texture {
                    name: texture.name().map(str::to_owned),
                    image: Some(image),
                    view: None,
                    sampler: None,
                }
            } else {
                info!("Base color factor: {:?}", base_color_factor);
                Texture {
                    name: None,
                    image: None,
                    view: None,
                    sampler: None,
                }
            };

            materials.push(Material {
                name: material.name().map(str::to_owned),
                base_color_factor: (base_color_factor, None),
                base_color_texture,
                bind_group: None,
            });
        }

        Ok(Model {
            meshes,
            materials,
            uniforms: None,
        })
    }

    pub fn allocate_buffers(&mut self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) {
        self.uniforms = Some(Uniforms::new(ModelBinding::new(), device, layout));

        for mesh in &mut self.meshes {
            for (index, primitive) in mesh.primitives.iter_mut().enumerate() {
                let debug_label = format!(
                    "{:?}#{}",
                    mesh.name.as_deref().unwrap_or("Unnamed Mesh"),
                    index
                );

                primitive.positions.1 = Some(device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some(&format!("Position Buffer {}", debug_label)),
                        contents: bytemuck::cast_slice(&primitive.positions.0),
                        usage: wgpu::BufferUsages::VERTEX,
                    },
                ));

                primitive.tex_coords.1 = Some(device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some(&format!("Tex Coord Buffer {}", debug_label)),
                        contents: bytemuck::cast_slice(&primitive.tex_coords.0),
                        usage: wgpu::BufferUsages::VERTEX,
                    },
                ));

                primitive.normals.1 = Some(device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some(&format!("Normal Buffer {}", debug_label)),
                        contents: bytemuck::cast_slice(&primitive.normals.0),
                        usage: wgpu::BufferUsages::VERTEX,
                    },
                ));

                primitive.indices.1 = Some(device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some(&format!("Index Buffer {}", debug_label)),
                        contents: bytemuck::cast_slice(&primitive.indices.0),
                        usage: wgpu::BufferUsages::INDEX,
                    },
                ));
            }
        }
    }

    pub fn load_materials(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) {
        for material in &mut self.materials {
            material.load(device, queue, bind_group_layout);
        }
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        for mesh in &self.meshes {
            for primitive in &mesh.primitives {
                let material = &self.materials[primitive.material_index];
                render_pass.set_bind_group(
                    Material::BIND_GROUP_INDEX,
                    material.bind_group.as_ref().unwrap(),
                    &[],
                );

                render_pass.set_vertex_buffer(
                    VertexAttribute::Position.location(),
                    primitive.positions.1.as_ref().unwrap().slice(..),
                );
                render_pass.set_vertex_buffer(
                    VertexAttribute::TexCoord.location(),
                    primitive.tex_coords.1.as_ref().unwrap().slice(..),
                );
                render_pass.set_vertex_buffer(
                    VertexAttribute::Normal.location(),
                    primitive.normals.1.as_ref().unwrap().slice(..),
                );

                render_pass.set_index_buffer(
                    primitive.indices.1.as_ref().unwrap().slice(..),
                    wgpu::IndexFormat::Uint32,
                );

                render_pass.draw_indexed(0..primitive.indices.0.len() as u32, 0, 0..1);
            }
        }
    }
}
