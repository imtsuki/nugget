use crate::Result;
use anyhow::anyhow;
use std::{fmt, path};
use tracing::{debug, info};
use wgpu::util::DeviceExt;

pub struct Model {
    pub meshes: Vec<Mesh>,
}

pub struct Mesh {
    pub name: Option<String>,
    pub primitives: Vec<Primitive>,
}

pub struct Primitive {
    pub positions: (Vec<[f32; 3]>, Option<wgpu::Buffer>),
    pub normals: (Vec<[f32; 3]>, Option<wgpu::Buffer>),
    pub indices: (Vec<u32>, Option<wgpu::Buffer>),
}

impl Model {
    pub fn load_gltf<P: AsRef<path::Path> + fmt::Debug>(path: P) -> Result<Model> {
        info!("Loading model from {:?}", path);
        let (gltf, buffers, _images) = gltf::import(path)?;

        for buffer in &buffers {
            debug!("Found buffer of size {}", buffer.len());
        }

        let mut meshes = vec![];

        for mesh in gltf.meshes() {
            let name = mesh.name().map(str::to_owned);
            let mut primitives = vec![];
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                let positions = reader
                    .read_positions()
                    .map(|iter| iter.collect::<Vec<_>>())
                    .ok_or_else(|| anyhow!("No positions found"))?;

                debug!("Found {} positions", positions.len());

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

                primitives.push(Primitive {
                    positions: (positions, None),
                    normals: (normals, None),
                    indices: (indices, None),
                });
            }

            meshes.push(Mesh { name, primitives });
        }

        Ok(Model { meshes })
    }

    pub fn allocate_buffers(&mut self, device: &wgpu::Device) {
        for mesh in &mut self.meshes {
            for primitive in &mut mesh.primitives {
                primitive.positions.1 = Some(device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Position Buffer"),
                        contents: bytemuck::cast_slice(&primitive.positions.0),
                        usage: wgpu::BufferUsages::VERTEX,
                    },
                ));

                primitive.normals.1 = Some(device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Normal Buffer"),
                        contents: bytemuck::cast_slice(&primitive.normals.0),
                        usage: wgpu::BufferUsages::VERTEX,
                    },
                ));

                primitive.indices.1 = Some(device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Index Buffer"),
                        contents: bytemuck::cast_slice(&primitive.indices.0),
                        usage: wgpu::BufferUsages::INDEX,
                    },
                ));
            }
        }
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        for mesh in &self.meshes {
            for primitive in &mesh.primitives {
                render_pass.set_vertex_buffer(0, primitive.positions.1.as_ref().unwrap().slice(..));
                // render_pass.set_vertex_buffer(1, primitive.normals.1.as_ref().unwrap().slice(..));
                render_pass.set_index_buffer(
                    primitive.indices.1.as_ref().unwrap().slice(..),
                    wgpu::IndexFormat::Uint32,
                );
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
