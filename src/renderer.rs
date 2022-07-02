use std::{borrow::Cow, ffi, fmt, path::Path};

use anyhow::{anyhow, Result};

pub struct Renderer {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub shader: wgpu::ShaderModule,
    pub swapchain_format: wgpu::TextureFormat,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub async fn new<W>(window: &W) -> Result<Renderer>
    where
        W: raw_window_handle::HasRawWindowHandle,
    {
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&window) };
        let adapter_options = wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        };
        let adapter = instance
            .request_adapter(&adapter_options)
            .await
            .ok_or(anyhow!("Failed to find an appropriate adapter"))?;
        let device_descriptor = wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::empty(),
            // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
            limits: wgpu::Limits::default().using_resolution(adapter.limits()),
        };
        let (device, queue) = adapter.request_device(&device_descriptor, None).await?;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        // The first format in the vector is preferred
        let swapchain_format = surface.get_supported_formats(&adapter)[0];

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vertex_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fragment_main",
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Ok(Renderer {
            surface,
            device,
            queue,
            shader,
            swapchain_format,
            pipeline_layout,
            render_pipeline,
        })
    }

    pub fn size_changed(&self, width: u32, height: u32) {
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.swapchain_format,
            width,
            height,
            present_mode: wgpu::PresentMode::Mailbox,
        };
        self.surface.configure(&self.device, &config);
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
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
            rpass.set_pipeline(&self.render_pipeline);
            rpass.draw(0..3, 0..1);
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
                return Err(anyhow!("Unsupported file extension"));
            }
        }

        Ok(())
    }
}
