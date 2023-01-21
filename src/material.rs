use wgpu::util::DeviceExt;

use crate::texture::Texture;

#[derive(Debug)]
pub struct Material {
    pub name: Option<String>,
    pub base_color_factor: ([f32; 4], Option<wgpu::Buffer>),
    pub base_color_texture: Texture,
    pub bind_group: Option<wgpu::BindGroup>,
}

impl Material {
    pub const BIND_GROUP_INDEX: u32 = 2;

    pub const BIND_GROUP_LAYOUT_DESCRIPTOR: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            label: Some("Material Bind Group Layout"),
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

    pub fn load(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) {
        let base_color_factor = self.base_color_factor.0;
        let base_color_factor_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Base Color Factor"),
                contents: bytemuck::cast_slice(&[base_color_factor]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        self.base_color_factor.1 = Some(base_color_factor_buffer);
        self.base_color_texture.load(device, queue);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Material Bind Group"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        self.base_color_factor
                            .1
                            .as_ref()
                            .unwrap()
                            .as_entire_buffer_binding(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        self.base_color_texture.view.as_ref().unwrap(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(
                        self.base_color_texture.sampler.as_ref().unwrap(),
                    ),
                },
            ],
        });

        self.bind_group = Some(bind_group);
    }
}
