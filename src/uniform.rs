use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[derive(Debug)]

pub struct Uniforms<T: bytemuck::NoUninit> {
    pub data: T,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl<T: bytemuck::NoUninit> Uniforms<T> {
    pub fn new(data: T, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> Self {
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

    pub fn update(&mut self, data: T, queue: &wgpu::Queue) {
        self.data = data;
        self.write_buffer(queue);
    }

    pub fn write_buffer(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&self.data));
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CameraBinding {
    pub view_matrix: glam::Mat4,
    pub projection_matrix: glam::Mat4,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ModelBinding {
    pub model_matrix: glam::Mat4,
}

impl ModelBinding {
    pub fn new() -> Self {
        let translation_matrix = glam::Mat4::from_translation(glam::Vec3::new(0.0, -0.5, 0.0));
        let rotation_matrix = glam::Mat4::from_rotation_y(0.0);
        let scale_matrix = glam::Mat4::from_scale(glam::Vec3 {
            x: 1.5,
            y: 1.5,
            z: 1.5,
        });

        let model_matrix = translation_matrix * rotation_matrix * scale_matrix;

        Self { model_matrix }
    }
}

unsafe impl Pod for CameraBinding {}
unsafe impl Zeroable for CameraBinding {}
unsafe impl Pod for ModelBinding {}
unsafe impl Zeroable for ModelBinding {}
