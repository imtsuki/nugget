use std::fmt;

use crate::resources;

pub struct Texture {
    pub name: Option<String>,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl fmt::Debug for Texture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Texture")
            .field("name", &self.name)
            .field("view", &self.view)
            .field("sampler", &self.sampler)
            .finish()
    }
}

impl Texture {
    pub fn new(
        name: Option<String>,
        image: &resources::Image,
        _sampler: &resources::Sampler,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Texture {
        let size = wgpu::Extent3d {
            width: image.width(),
            height: image.height(),
            depth_or_array_layers: 1,
        };

        let texture = Self::create_device_texture(size, device);

        #[cfg(not(target_arch = "wasm32"))]
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            image.as_raw(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * size.width),
                rows_per_image: std::num::NonZeroU32::new(size.height),
            },
            size,
        );

        #[cfg(target_arch = "wasm32")]
        {
            let image_copy_external_image = wgpu::ImageCopyExternalImage {
                source: wgpu::ExternalImageSource::ImageBitmap(image.clone()),
                origin: wgpu::Origin2d::ZERO,
                flip_y: false,
            };
            queue.copy_external_image_to_texture(
                &image_copy_external_image,
                wgpu::ImageCopyTextureTagged {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                    color_space: wgpu::PredefinedColorSpace::Srgb,
                    premultiplied_alpha: false,
                },
                size,
            );
        }

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

        Texture {
            name,
            view: texture_view,
            sampler,
        }
    }

    pub fn white(device: &wgpu::Device, queue: &wgpu::Queue) -> Texture {
        let size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };

        let texture = Self::create_device_texture(size, device);

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[0xff, 0xff, 0xff, 0xff],
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * size.width),
                rows_per_image: std::num::NonZeroU32::new(size.height),
            },
            size,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

        Texture {
            name: Some("white".to_string()),
            view: texture_view,
            sampler,
        }
    }

    fn create_device_texture(size: wgpu::Extent3d, device: &wgpu::Device) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: if cfg!(target_arch = "wasm32") {
                // Chrome performs color space conversion,
                wgpu::TextureFormat::Rgba8Unorm
            } else {
                // while the image crate does not
                wgpu::TextureFormat::Rgba8Unorm
            },
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        })
    }
}
