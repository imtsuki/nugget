/// TODO: remove this once `wgpu 0.15.0` is released
pub trait SurfaceExt {
    fn get_default_config(
        &self,
        adapter: &wgpu::Adapter,
        width: u32,
        height: u32,
    ) -> Option<wgpu::SurfaceConfiguration>;
}

impl SurfaceExt for wgpu::Surface {
    fn get_default_config(
        &self,
        adapter: &wgpu::Adapter,
        width: u32,
        height: u32,
    ) -> Option<wgpu::SurfaceConfiguration> {
        let format = *self.get_supported_formats(adapter).get(0)?;
        let present_mode = *self.get_supported_present_modes(adapter).get(0)?;

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };

        Some(config)
    }
}

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
        });

        depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
    }
}
