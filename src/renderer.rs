use std::{ffi, fmt, path::Path};

use anyhow::{anyhow, bail, Result};

use tracing::info;

use crate::ext::{DeviceExt, SurfaceExt};
use crate::model::Model;
use crate::texture::Texture;
use crate::vertex::VertexIn;

pub struct Renderer {
    pub adapter: wgpu::Adapter,
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub shader: wgpu::ShaderModule,
    pub pipeline: wgpu::RenderPipeline,
    pub depth_texture: wgpu::TextureView,
    pub model: Model,
}

impl Renderer {
    pub async fn new<W>(
        window: &W,
        width: u32,
        height: u32,
        mut model: Model,
        line: bool,
    ) -> Result<Renderer>
    where
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    {
        // Context for all other wgpu objects. Instance of wgpu.
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        // Surface: handle to a presentable surface.
        let surface = unsafe { instance.create_surface(&window) };

        // An adapter identifies an implementation of WebGPU on the system.
        let adapter_options = wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        };
        let adapter = instance
            .request_adapter(&adapter_options)
            .await
            .ok_or_else(|| anyhow!("Failed to find an appropriate GPU adapter"))?;

        info!("Supported features: {:?}", adapter.features());

        let config = surface
            .get_default_config(&adapter, width, height)
            .ok_or_else(|| anyhow!("Failed to get default surface configuration"))?;

        // A device is the logical instantiation of an adapter.
        let device_descriptor = wgpu::DeviceDescriptor {
            label: None,
            features: if line {
                wgpu::Features::POLYGON_MODE_LINE
            } else {
                wgpu::Features::empty()
            },
            // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
            limits: wgpu::Limits::default().using_resolution(adapter.limits()),
        };
        let (device, queue) = adapter.request_device(&device_descriptor, None).await?;

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let texture_bind_group_layout =
            device.create_bind_group_layout(&Texture::BIND_GROUP_LAYOUT_DESCRIPTOR);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let vertex_buffer_layouts = VertexIn::BUFFER_LAYOUTS;

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vertex_main",
                buffers: &vertex_buffer_layouts,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fragment_main",
                targets: &[Some(config.format.into())],
            }),
            primitive: wgpu::PrimitiveState {
                polygon_mode: if line {
                    wgpu::PolygonMode::Line
                } else {
                    wgpu::PolygonMode::Fill
                },
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        model.allocate_buffers(&device);
        model.load_textures(&device, &queue, &texture_bind_group_layout);

        let depth_texture = device.create_depth_texture(&config);

        Ok(Renderer {
            adapter,
            surface,
            config,
            device,
            queue,
            shader,
            pipeline,
            model,
            depth_texture,
        })
    }

    pub fn size_changed(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.depth_texture = self.device.create_depth_texture(&self.config);
        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(&self) {
        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            render_pass.set_pipeline(&self.pipeline);

            self.model.render(&mut render_pass);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    pub fn load_model<P: AsRef<Path> + fmt::Debug>(&self, path: P) -> Result<()> {
        match path.as_ref().extension().and_then(ffi::OsStr::to_str) {
            Some("obj") => {
                let options = tobj::LoadOptions::default();
                let (models, materials) = tobj::load_obj(path, &options)?;
                let materials = materials?;
                for model in models {
                    dbg!(model.name);
                    dbg!(model.mesh.material_id);
                }
                for material in materials {
                    dbg!(material.name);
                }
            }
            _ => {
                bail!("Unsupported file extension");
            }
        }

        Ok(())
    }
}
