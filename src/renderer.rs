use std::{ffi, fmt, mem, path::Path};

use anyhow::{anyhow, bail, Result};

use tracing::info;

use crate::model::Model;

pub struct Renderer {
    pub adapter: wgpu::Adapter,
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub shader: wgpu::ShaderModule,
    pub render_pipeline: wgpu::RenderPipeline,
    pub model: Model,
}

/// TODO: remove this once `wgpu 0.15.0` is released
trait SurfaceExt {
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

#[repr(C)]
struct VertexIn {
    position: [f32; 4],
}

impl Renderer {
    pub async fn new<W>(window: &W, width: u32, height: u32) -> Result<Renderer>
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
            .ok_or(anyhow!("Failed to find an appropriate GPU adapter"))?;

        info!("Supported features: {:?}", adapter.features());

        let config = surface
            .get_default_config(&adapter, width, height)
            .ok_or(anyhow!("Failed to get default surface configuration"))?;

        // A device is the logical instantiation of an adapter.
        let device_descriptor = wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::empty(),
            // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
            limits: wgpu::Limits::default().using_resolution(adapter.limits()),
        };
        let (device, queue) = adapter.request_device(&device_descriptor, None).await?;

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let model = Model::new(&device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let vertex_buffer_layouts = [wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<VertexIn>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x4],
        }];

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Ok(Renderer {
            adapter,
            surface,
            config,
            device,
            queue,
            shader,
            render_pipeline,
            model,
        })
    }

    pub fn size_changed(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
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
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);

            self.model.render(&mut render_pass);

            render_pass.set_pipeline(&self.render_pipeline);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    pub fn load_model<P: AsRef<Path> + fmt::Debug>(&self, path: P) -> Result<()> {
        use std::{fs, io};

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
            Some("gltf") => {
                let file = fs::File::open(path)?;
                let reader = io::BufReader::new(file);
                let model = gltf::Gltf::from_reader(reader)?;
                dbg!(model);
            }
            _ => {
                bail!("Unsupported file extension");
            }
        }

        Ok(())
    }
}
