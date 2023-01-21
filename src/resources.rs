use anyhow::anyhow;
use std::path;

use crate::Result;

#[cfg(target_arch = "wasm32")]
pub type Image = web_sys::ImageBitmap;

#[cfg(not(target_arch = "wasm32"))]
pub type Image = image::RgbaImage;

pub type Buffer = Vec<u8>;

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
