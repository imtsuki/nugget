pub trait DeviceExt {
    fn create_depth_texture(&self, config: &wgpu::SurfaceConfiguration) -> wgpu::TextureView;
}

impl DeviceExt for wgpu::Device {
    fn create_depth_texture(&self, config: &wgpu::SurfaceConfiguration) -> wgpu::TextureView {
        let depth_texture = self.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[wgpu::TextureFormat::Depth32Float],
        });

        depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
    }
}

pub trait RgbaImageExt {
    fn from_gltf_image(image: gltf::image::Data) -> Option<image::RgbaImage>;
}

impl RgbaImageExt for image::RgbaImage {
    fn from_gltf_image(image: gltf::image::Data) -> Option<image::RgbaImage> {
        use gltf::image::Format;
        use image::buffer::ConvertBuffer;
        Some(match image.format {
            Format::R8G8B8A8 => {
                image::RgbaImage::from_raw(image.width, image.height, image.pixels)?
            }
            Format::R8G8B8 => {
                image::RgbImage::from_raw(image.width, image.height, image.pixels)?.convert()
            }
            _ => unimplemented!("Image format not yet implemented: {:?}", image.format),
        })
    }
}
