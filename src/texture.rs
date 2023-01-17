use wgpu::util::DeviceExt;

pub struct Texture {
    pub name: Option<String>,
    pub factor: ([f32; 4], Option<wgpu::Buffer>),
    pub image: Option<image::RgbaImage>,
    pub view: Option<wgpu::TextureView>,
    pub sampler: Option<wgpu::Sampler>,
}

impl Texture {
    pub const BIND_GROUP_LAYOUT_DESCRIPTOR: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                // base color factor
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // base color texture
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // base color sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        };

    pub fn load(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let factor = self.factor.0;
        let factor_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Base Color Factor"),
            contents: bytemuck::cast_slice(&[factor]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        self.factor.1 = Some(factor_buffer);

        let size = if let Some(image) = &self.image {
            wgpu::Extent3d {
                width: image.width(),
                height: image.height(),
                depth_or_array_layers: 1,
            }
        } else {
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            }
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Base Color Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            if let Some(image) = &self.image {
                image.as_raw()
            } else {
                &[0xff, 0xff, 0xff, 0xff]
            },
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * size.width),
                rows_per_image: std::num::NonZeroU32::new(size.height),
            },
            size,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

        self.view = Some(texture_view);
        self.sampler = Some(sampler);
    }
}
