use crate::vertex::VertexAttribute;
use crate::Result;
use crate::{ext::RgbaImageExt, texture::Texture};
use anyhow::anyhow;
use std::{fmt, path};
use tracing::{debug, info};
use wgpu::util::DeviceExt;

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
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

pub struct Material {
    pub name: Option<String>,
    pub base_color_texture: Texture,
    pub bind_group: Option<wgpu::BindGroup>,
}

impl Model {
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
                    factor: (base_color_factor, None),
                    image: Some(image),
                    view: None,
                    sampler: None,
                }
            } else {
                info!("Base color factor: {:?}", base_color_factor);
                Texture {
                    name: None,
                    factor: (base_color_factor, None),
                    image: None,
                    view: None,
                    sampler: None,
                }
            };

            materials.push(Material {
                name: material.name().map(str::to_owned),
                base_color_texture,
                bind_group: None,
            });
        }

        Ok(Model { meshes, materials })
    }

    pub fn allocate_buffers(&mut self, device: &wgpu::Device) {
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

    pub fn load_textures(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) {
        for material in &mut self.materials {
            material.base_color_texture.load(device, queue);

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Material Bind Group"),
                layout: bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            material
                                .base_color_texture
                                .factor
                                .1
                                .as_ref()
                                .unwrap()
                                .as_entire_buffer_binding(),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            material.base_color_texture.view.as_ref().unwrap(),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(
                            material.base_color_texture.sampler.as_ref().unwrap(),
                        ),
                    },
                ],
            });

            material.bind_group = Some(bind_group);
        }
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        for mesh in &self.meshes {
            for primitive in &mesh.primitives {
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
                let material = &self.materials[primitive.material_index];
                render_pass.set_bind_group(0, material.bind_group.as_ref().unwrap(), &[]);
                render_pass.draw_indexed(0..primitive.indices.0.len() as u32, 0, 0..1);
            }
        }
    }
}

pub struct OldModel {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
}

impl OldModel {
    pub fn load() -> OldModel {
        todo!()
    }

    pub fn rect(device: &wgpu::Device) -> OldModel {
        #[rustfmt::skip]
        let vertices: Vec<f32> = vec![
            -1.,  1., 0., 1.,
             1.,  1., 0., 1.,
            -1., -1., 0., 1.,
             1., -1., 0., 1.,
        ];

        let indices: Vec<u32> = vec![0, 3, 2, 0, 1, 3];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        OldModel {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
        }
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}
