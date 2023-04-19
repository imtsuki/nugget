use crate::{camera::ArcCamera, model::Model};

pub struct Scene {
    pub models: Vec<Model>,
    pub camera: ArcCamera,
}

impl Scene {
    pub const BIND_GROUP_INDEX: u32 = 0;

    pub const BIND_GROUP_LAYOUT_DESCRIPTOR: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            label: Some("Scene Uniforms Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        };

    pub fn new(
        width: u32,
        height: u32,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        Self {
            models: vec![],
            camera: ArcCamera::new(width, height, device, layout),
        }
    }

    pub fn add_model(&mut self, model: Model) {
        self.models.push(model);
    }

    pub fn clear_models(&mut self) {
        self.models.clear();
    }

    pub fn resize_viewport(&mut self, width: u32, height: u32, queue: &wgpu::Queue) {
        self.camera.resize_viewport(width, height, queue);
    }

    pub fn rotate_camera(&mut self, delta: glam::Vec2, queue: &wgpu::Queue) {
        self.camera.rotate(delta, queue);
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_bind_group(
            Scene::BIND_GROUP_INDEX,
            &self.camera.uniforms.bind_group,
            &[],
        );

        for model in &self.models {
            model.render(render_pass);
        }
    }
}
