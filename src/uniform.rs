use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

pub struct Uniforms {
    pub data: UniformsData,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl Uniforms {
    pub const BIND_GROUP_LAYOUT_DESCRIPTOR: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniforms Bind Group Layout"),
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
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        width: u32,
        height: u32,
    ) -> Self {
        let data = UniformsData::new(width, height);
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniforms Buffer"),
            contents: bytemuck::bytes_of(&data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniforms Bind Group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        Self {
            data,
            buffer,
            bind_group,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&self.data));
    }

    pub fn resize(&mut self, width: u32, height: u32, queue: &wgpu::Queue) {
        self.data.resize(width, height);
        self.update(queue);
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UniformsData {
    pub model_matrix: glam::Mat4,
    pub view_matrix: glam::Mat4,
    pub projection_matrix: glam::Mat4,
}

impl UniformsData {
    pub fn new(width: u32, height: u32) -> Self {
        let translation_matrix = glam::Mat4::from_translation(glam::Vec3::new(0.0, -0.5, 2.0));
        let rotation_matrix = glam::Mat4::from_rotation_y(2.5);
        let scale_matrix = glam::Mat4::from_scale(glam::Vec3 {
            x: 1.5,
            y: 1.5,
            z: 1.5,
        });

        let model_matrix = translation_matrix * rotation_matrix * scale_matrix;
        let view_matrix = glam::Mat4::from_translation(glam::Vec3::new(0.0, 0.0, 0.0)).inverse();
        let projection_matrix =
            glam::Mat4::perspective_lh(1.0, width as f32 / height as f32, 0.1, 100.0);

        Self {
            model_matrix,
            view_matrix,
            projection_matrix,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.projection_matrix =
            glam::Mat4::perspective_lh(1.0, width as f32 / height as f32, 0.1, 100.0);
    }
}

unsafe impl Pod for UniformsData {}
unsafe impl Zeroable for UniformsData {}
