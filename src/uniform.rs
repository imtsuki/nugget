use std::mem;

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

#[derive(Debug)]

pub struct UniformsArray<T: bytemuck::NoUninit> {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    alignment: wgpu::BufferAddress,
    _marker: std::marker::PhantomData<T>,
}

impl<T: bytemuck::NoUninit> UniformsArray<T> {
    pub fn new(len: usize, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> Self {
        let element_size = mem::size_of::<T>() as wgpu::BufferAddress;
        let alignment = wgpu::util::align_to(
            element_size,
            device.limits().min_uniform_buffer_offset_alignment as u64,
        );

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("UniformsArrayBuffer"),
            size: alignment * len as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("UniformsArrayBindGroup"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(element_size),
                }),
            }],
        });

        Self {
            buffer,
            bind_group,
            alignment,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn update(&self, data: T, index: usize, queue: &wgpu::Queue) {
        let offset = index as u64 * self.alignment;
        queue.write_buffer(&self.buffer, offset, bytemuck::bytes_of(&data));
    }

    pub fn offset(&self, index: usize) -> wgpu::BufferAddress {
        index as u64 * self.alignment
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

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EntityBinding {
    pub transform: glam::Mat4,
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug)]
pub struct MaterialFactorsBinding {
    pub base_color_factor: [f32; 4],
    pub metallic_factor: f32,
    pub roughness_factor: f32,
}

unsafe impl Pod for CameraBinding {}
unsafe impl Zeroable for CameraBinding {}
unsafe impl Pod for ModelBinding {}
unsafe impl Zeroable for ModelBinding {}
unsafe impl Pod for EntityBinding {}
unsafe impl Zeroable for EntityBinding {}
unsafe impl Pod for MaterialFactorsBinding {}
unsafe impl Zeroable for MaterialFactorsBinding {}
