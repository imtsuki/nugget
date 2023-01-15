use std::{fmt, path};

use crate::Result;

use tracing::debug;

pub struct Model {
    pub meshes: Vec<Mesh>,
}

pub struct Mesh {
    pub name: Option<String>,
    pub primitives: Vec<Primitive>,
}

pub struct Primitive {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

pub fn load_gltf<P: AsRef<path::Path> + fmt::Debug>(path: P) -> Result<Model> {
    let (gltf, buffers, _images) = gltf::import(path)?;

    dbg!(&gltf);

    for buffer in &buffers {
        debug!("Found buffer of size {}", buffer.len());
    }

    let mut meshes = vec![];

    for mesh in gltf.meshes() {
        let name = mesh.name().map(str::to_owned);
        let mut primitives = vec![];
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            if let Some(positions) = reader.read_positions().map(|iter| iter.collect::<Vec<_>>()) {
                debug!("Found {} positions", positions.len());
            }

            if let Some(normals) = reader.read_normals().map(|iter| iter.collect::<Vec<_>>()) {
                debug!("Found {} normals", normals.len());
            }

            if let Some(indices) = reader
                .read_indices()
                .map(|iter| iter.into_u32().collect::<Vec<_>>())
            {
                debug!("Found {} indices", indices.len());
            }

            primitives.push(Primitive {
                positions: reader
                    .read_positions()
                    .map(|iter| iter.collect::<Vec<_>>())
                    .unwrap(),
                normals: reader
                    .read_normals()
                    .map(|iter| iter.collect::<Vec<_>>())
                    .unwrap(),
                indices: reader
                    .read_indices()
                    .map(|iter| iter.into_u32().collect::<Vec<_>>())
                    .unwrap(),
            });
        }

        meshes.push(Mesh { name, primitives });
    }

    Ok(Model { meshes })
}
