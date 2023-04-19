use anyhow::{anyhow, Result};

use tracing::info;

use crate::entity::Entity;
use crate::ext::DeviceExt;
use crate::material::Material;
use crate::mesh::Mesh;
use crate::model::Model;
use crate::scene::Scene;
use crate::texture::Texture;
use crate::vertex::VertexIn;
use crate::Resources;

pub struct BindGroupLayouts {
    pub scene: wgpu::BindGroupLayout,
    pub model: wgpu::BindGroupLayout,
    pub material: wgpu::BindGroupLayout,
}

pub struct Renderer {
    pub adapter: wgpu::Adapter,
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub shader: wgpu::ShaderModule,
    pub pipeline: wgpu::RenderPipeline,
    pub depth_texture: wgpu::TextureView,
    pub bind_group_layouts: BindGroupLayouts,
    pub scene: Scene,
}

impl Renderer {
    pub async fn new<W>(window: &W, width: u32, height: u32, line: bool) -> Result<Renderer>
    where
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    {
        // Context for all other wgpu objects. Instance of wgpu.
        let instance = wgpu::Instance::default();

        // Surface: handle to a presentable surface.
        let surface = unsafe { instance.create_surface(&window)? };

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

        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let scene_bind_group_layout =
            device.create_bind_group_layout(&Scene::BIND_GROUP_LAYOUT_DESCRIPTOR);

        let model_bind_group_layout =
            device.create_bind_group_layout(&Model::BIND_GROUP_LAYOUT_DESCRIPTOR);

        let material_bind_group_layout =
            device.create_bind_group_layout(&Material::BIND_GROUP_LAYOUT_DESCRIPTOR);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &scene_bind_group_layout,
                &model_bind_group_layout,
                &material_bind_group_layout,
            ],
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

        let scene = Scene::new(
            config.width,
            config.height,
            &device,
            &scene_bind_group_layout,
        );

        let depth_texture = device.create_depth_texture(&config);

        Ok(Renderer {
            adapter,
            surface,
            config,
            device,
            queue,
            shader,
            pipeline,
            depth_texture,
            bind_group_layouts: BindGroupLayouts {
                scene: scene_bind_group_layout,
                model: model_bind_group_layout,
                material: material_bind_group_layout,
            },
            scene,
        })
    }

    pub fn size_changed(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.depth_texture = self.device.create_depth_texture(&self.config);
        self.scene.resize_viewport(width, height, &self.queue);
        self.surface.configure(&self.device, &self.config);
    }

    pub fn rotate_camera(&mut self, x: f32, y: f32) {
        self.scene.rotate_camera(glam::Vec2::new(x, y), &self.queue);
    }

    pub fn render(&self) {
        tracing::debug!("Rendering new frame");
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
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.3,
                            g: 0.3,
                            b: 0.3,
                            a: 1.0,
                        }),
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

            self.scene.render(&mut render_pass);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    pub fn set_model(&mut self, model: Model) {
        self.scene.clear_models();
        self.scene.add_model(model);
    }

    pub fn load_resources(&mut self, resources: Resources) {
        let textures: Vec<Texture> = resources
            .textures
            .into_iter()
            .map(|texture| {
                Texture::new(
                    texture.name.clone(),
                    &resources.images[texture.source_index],
                    &texture.sampler,
                    &self.device,
                    &self.queue,
                )
            })
            .collect();

        let materials: Vec<Material> = resources
            .materials
            .into_iter()
            .map(|material| {
                Material::new(
                    material.name,
                    &material.base_color_factor,
                    material.base_color_texture_index.map(|i| &textures[i]),
                    &self.device,
                    &self.queue,
                    &self.bind_group_layouts.material,
                )
            })
            .collect();

        let meshes: Vec<Mesh> = resources
            .meshes
            .into_iter()
            .map(|mesh| Mesh::new(&mesh, &self.device))
            .collect();

        let entities: Vec<Entity> = resources
            .nodes
            .into_iter()
            .map(|node| node.into())
            .collect();

        let root_entity_indices = resources.scenes[resources.default_scene_index]
            .nodes
            .clone();

        let root_entity = Entity {
            name: Some("Root".to_string()),
            transform: glam::Mat4::from_diagonal(glam::Vec4::new(-1.0, 1.0, 1.0, 1.0)),
            mesh_index: None,
            children: root_entity_indices,
        };

        let model = Model::new(
            meshes,
            materials,
            entities,
            root_entity,
            &self.device,
            &self.queue,
            &self.bind_group_layouts.model,
        );

        self.set_model(model);
    }
}
