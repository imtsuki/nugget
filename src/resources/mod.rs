use std::{fmt, path};

use anyhow::anyhow;
use tracing::{debug, info};

use crate::Result;

mod material;
mod mesh;
mod node;
mod scene;
mod texture;

pub use material::Material;
pub use mesh::{Mesh, Primitive};
pub use node::Node;
pub use scene::Scene;
pub use texture::{Sampler, Texture};

#[cfg(target_arch = "wasm32")]
pub type Image = web_sys::ImageBitmap;

#[cfg(not(target_arch = "wasm32"))]
pub type Image = image::RgbaImage;

pub type Buffer = Vec<u8>;

pub struct Resources {
    pub scenes: Vec<Scene>,
    pub nodes: Vec<Node>,
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub textures: Vec<Texture>,
    pub buffers: Vec<Buffer>,
    pub images: Vec<Image>,
    pub default_scene_index: usize,
}

impl fmt::Debug for Resources {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Resources")
            .field("scenes", &self.scenes.len())
            .field("nodes", &self.nodes.len())
            .field("meshes", &self.meshes.len())
            .field("materials", &self.materials.len())
            .field("textures", &self.textures.len())
            .field("images", &self.images.len())
            .field("default_scene_index", &self.default_scene_index)
            .finish()
    }
}

impl Resources {
    pub async fn load_gltf<P: AsRef<path::Path> + fmt::Debug>(path: P) -> Result<Resources> {
        let (gltf, buffers, images) = import_gltf(path).await?;

        let mut textures = vec![];

        for texture in gltf.textures() {
            info!(
                index = texture.index(),
                name = texture.name(),
                "Loading texture"
            );

            let name = texture.name().map(str::to_owned);
            let source_index = texture.source().index();
            let sampler = texture.sampler().into();

            let texture = Texture::new(name, source_index, sampler);

            textures.push(texture);
        }

        info!(textures = textures.len(), "Loaded textures");

        let mut materials = vec![];

        for material in gltf.materials() {
            info!(
                index = material.index(),
                name = material.name(),
                "Loading material"
            );

            let name = material.name().map(str::to_owned);
            let pbr = material.pbr_metallic_roughness();
            let base_color_factor = pbr.base_color_factor();

            let base_color_texture_index = pbr
                .base_color_texture()
                .map(|texture_info| texture_info.texture().index());

            let normal_texture_index = material
                .normal_texture()
                .map(|texture_info| texture_info.texture().index());

            let material = Material {
                name,
                base_color_factor,
                base_color_texture_index,
                normal_texture_index,
            };

            materials.push(material);
        }

        info!(materials = materials.len(), "Loaded materials");

        let mut meshes = vec![];

        for mesh in gltf.meshes() {
            info!(index = mesh.index(), name = mesh.name(), "Loading mesh");

            let name = mesh.name().map(str::to_owned);

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
                    positions,
                    tex_coords,
                    normals,
                    indices,
                    material_index,
                });
            }

            meshes.push(Mesh { name, primitives });
        }

        info!(meshes = meshes.len(), "Loaded meshes");

        let mut nodes = vec![];

        for node in gltf.nodes() {
            info!(index = node.index(), name = node.name(), "Loading node");

            let name = node.name().map(str::to_owned);

            let mut children = vec![];

            for child in node.children() {
                children.push(child.index());
            }

            let mesh_index = node.mesh().map(|mesh| mesh.index());

            let transform = node.transform().matrix();

            let transform = glam::Mat4::from_cols_array_2d(&transform);

            nodes.push(Node {
                name,
                children,
                mesh_index,
                transform,
            });
        }

        info!(nodes = nodes.len(), "Loaded nodes");

        let mut scenes = vec![];

        for scene in gltf.scenes() {
            info!(index = scene.index(), name = scene.name(), "Loading scene");

            let name = scene.name().map(str::to_owned);

            let mut nodes = vec![];

            for node in scene.nodes() {
                nodes.push(node.index());
            }

            scenes.push(Scene { name, nodes });
        }

        let default_scene_index = gltf.default_scene().map(|scene| scene.index()).unwrap_or(0);

        Ok(Resources {
            scenes,
            nodes,
            meshes,
            materials,
            textures,
            buffers,
            images,
            default_scene_index,
        })
    }
}

pub async fn import_gltf<P>(path: P) -> Result<(gltf::Document, Vec<Buffer>, Vec<Image>)>
where
    P: AsRef<path::Path>,
{
    #[cfg(target_arch = "wasm32")]
    {
        crate::wasm::import_gltf(path).await.map_err(|e| {
            tracing::error!("Failed to fetch gltf: {:?}", e);
            anyhow!("Failed to fetch gltf: {:?}", e)
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let (gltf, buffers, images) = gltf::import(path)?;

        let buffers = buffers
            .into_iter()
            .map(|buffer| buffer.0)
            .collect::<Vec<_>>();

        let images = images
            .into_iter()
            .map(|image| {
                let image = {
                    use crate::ext::RgbaImageExt;
                    image::RgbaImage::from_gltf_image(image)
                        .ok_or(anyhow!("Failed to convert gltf image to rgba"))?
                };
                Ok(image)
            })
            .collect::<Result<Vec<_>>>()?;

        Ok((gltf, buffers, images))
    }
}
