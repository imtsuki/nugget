use std::{fmt, sync::OnceLock};

use crate::resources;

pub struct Texture {
    pub name: Option<String>,
    pub texture: wgpu::Texture,
    pub sampler: wgpu::Sampler,
}

static DEFAULT_BASE_COLOR_TEXTURE: OnceLock<Texture> = OnceLock::new();
static DEFAULT_NORMAL_TEXTURE: OnceLock<Texture> = OnceLock::new();

impl fmt::Debug for Texture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Texture")
            .field("name", &self.name)
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

        tracing::debug!("width: {}, height: {}", size.width, size.height);

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
                bytes_per_row: Some(4 * size.width),
                rows_per_image: Some(size.height),
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

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

        Texture {
            name,
            texture,
            sampler,
        }
    }

    pub fn default_base_color_texture<'a>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> &'a Texture {
        DEFAULT_BASE_COLOR_TEXTURE.get_or_init(|| {
            Self::create_solid_color_texture(
                Some("default_base_color".to_string()),
                [0xff, 0xff, 0xff, 0xff],
                device,
                queue,
            )
        })
    }

    pub fn default_normal_texture<'a>(device: &wgpu::Device, queue: &wgpu::Queue) -> &'a Texture {
        DEFAULT_NORMAL_TEXTURE.get_or_init(|| {
            Self::create_solid_color_texture(
                Some("default_normal".to_string()),
                [0x80, 0x80, 0xff, 0xff],
                device,
                queue,
            )
        })
    }

    fn create_solid_color_texture(
        name: Option<String>,
        color: [u8; 4],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Texture {
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
            &color,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * size.width),
                rows_per_image: Some(size.height),
            },
            size,
        );

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

        Texture {
            name,
            texture,
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
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[
                wgpu::TextureFormat::Rgba8Unorm,
                wgpu::TextureFormat::Rgba8UnormSrgb,
            ],
        })
    }

    pub fn create_view(&self, format: wgpu::TextureFormat) -> wgpu::TextureView {
        self.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(format),
            ..Default::default()
        })
    }
}
