use std::borrow::Cow;

use cgmath::{Matrix, Rotation3, SquareMatrix};
use winit::keyboard::KeyCode;

use crate::{
    mesh::{GpuMesh, Mesh, Vertex},
    mesh_render_pipeline::MeshRenderPipeline,
    texture::{create_depth_texture, create_fullscreen_texture, Texture},
    Renderer,
};

enum RenderSource {
    Albedo,
    Position,
    Normal,
}

pub struct App {
    depth_texture: Texture,

    mesh_render_pipeline: MeshRenderPipeline,

    mesh: GpuMesh,

    albedo_texture: Texture,
    position_texture: Texture,
    normal_texture: Texture,

    fullscreen_render_pipeline: wgpu::RenderPipeline,
    fullscreen_bind_group_layout: wgpu::BindGroupLayout,

    uniforms_buffer: wgpu::Buffer,
    _uniforms_bind_group_layout: wgpu::BindGroupLayout,
    uniforms_bind_group: wgpu::BindGroup,

    rotating: Option<(f32, f32)>,
    last_mouse_position: (f32, f32),
    yaw: f32,
    pitch: f32,

    render_source: RenderSource,
}

#[derive(Clone, Copy, bytemuck::NoUninit)]
#[repr(C)]
struct Uniforms {
    projection_matrix: [[f32; 4]; 4],
    projection_inv_matrix: [[f32; 4]; 4],
    view_matrix: [[f32; 4]; 4],
    model_matrix: [[f32; 4]; 4],
}

impl App {
    pub fn new(renderer: &Renderer) -> Self {
        let Renderer {
            device,
            surface_config,
            ..
        } = renderer;

        let depth_texture =
            create_depth_texture(device, surface_config.width, surface_config.height);
        let albedo_texture = create_fullscreen_texture(device, surface_config, "albedo texture");
        let position_texture =
            create_fullscreen_texture(device, surface_config, "position texture");
        let normal_texture = create_fullscreen_texture(device, surface_config, "normal texture");

        let uniforms_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("uniforms bind group layout"),
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
            });

        let uniforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniforms buffer"),
            size: std::mem::size_of::<Uniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniforms_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniforms bind group"),
            layout: &uniforms_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms_buffer.as_entire_binding(),
            }],
        });

        let mesh_render_pipeline = MeshRenderPipeline::new(renderer, &uniforms_bind_group_layout);

        let mut mesh = Mesh::default();

        const MIN: cgmath::Vector3<f32> = cgmath::vec3(-0.5, -0.5, -0.5);
        const MAX: cgmath::Vector3<f32> = cgmath::vec3(0.5, 0.5, 0.5);

        mesh.vertices
            .push(Vertex::raw(MIN.x, MIN.y, MAX.z, 0.0, 0.0, 1.0, 0.0, 0.0));
        mesh.vertices
            .push(Vertex::raw(MAX.x, MIN.y, MAX.z, 0.0, 0.0, 1.0, 1.0, 0.0));
        mesh.vertices
            .push(Vertex::raw(MAX.x, MAX.y, MAX.z, 0.0, 0.0, 1.0, 1.0, 1.0));
        mesh.vertices
            .push(Vertex::raw(MIN.x, MAX.y, MAX.z, 0.0, 0.0, 1.0, 0.0, 1.0));
        mesh.vertices
            .push(Vertex::raw(MIN.x, MAX.y, MIN.z, 0.0, 0.0, -1.0, 1.0, 0.0));
        mesh.vertices
            .push(Vertex::raw(MAX.x, MAX.y, MIN.z, 0.0, 0.0, -1.0, 0.0, 0.0));
        mesh.vertices
            .push(Vertex::raw(MAX.x, MIN.y, MIN.z, 0.0, 0.0, -1.0, 0.0, 1.0));
        mesh.vertices
            .push(Vertex::raw(MIN.x, MIN.y, MIN.z, 0.0, 0.0, -1.0, 1.0, 1.0));
        mesh.vertices
            .push(Vertex::raw(MAX.x, MIN.y, MIN.z, 1.0, 0.0, 0.0, 0.0, 0.0));
        mesh.vertices
            .push(Vertex::raw(MAX.x, MAX.y, MIN.z, 1.0, 0.0, 0.0, 1.0, 0.0));
        mesh.vertices
            .push(Vertex::raw(MAX.x, MAX.y, MAX.z, 1.0, 0.0, 0.0, 1.0, 1.0));
        mesh.vertices
            .push(Vertex::raw(MAX.x, MIN.y, MAX.z, 1.0, 0.0, 0.0, 0.0, 1.0));
        mesh.vertices
            .push(Vertex::raw(MIN.x, MIN.y, MAX.z, -1.0, 0.0, 0.0, 1.0, 0.0));
        mesh.vertices
            .push(Vertex::raw(MIN.x, MAX.y, MAX.z, -1.0, 0.0, 0.0, 0.0, 0.0));
        mesh.vertices
            .push(Vertex::raw(MIN.x, MAX.y, MIN.z, -1.0, 0.0, 0.0, 0.0, 1.0));
        mesh.vertices
            .push(Vertex::raw(MIN.x, MIN.y, MIN.z, -1.0, 0.0, 0.0, 1.0, 1.0));
        mesh.vertices
            .push(Vertex::raw(MAX.x, MAX.y, MIN.z, 0.0, 1.0, 0.0, 1.0, 0.0));
        mesh.vertices
            .push(Vertex::raw(MIN.x, MAX.y, MIN.z, 0.0, 1.0, 0.0, 0.0, 0.0));
        mesh.vertices
            .push(Vertex::raw(MIN.x, MAX.y, MAX.z, 0.0, 1.0, 0.0, 0.0, 1.0));
        mesh.vertices
            .push(Vertex::raw(MAX.x, MAX.y, MAX.z, 0.0, 1.0, 0.0, 1.0, 1.0));
        mesh.vertices
            .push(Vertex::raw(MAX.x, MIN.y, MAX.z, 0.0, -1.0, 0.0, 0.0, 0.0));
        mesh.vertices
            .push(Vertex::raw(MIN.x, MIN.y, MAX.z, 0.0, -1.0, 0.0, 1.0, 0.0));
        mesh.vertices
            .push(Vertex::raw(MIN.x, MIN.y, MIN.z, 0.0, -1.0, 0.0, 1.0, 1.0));
        mesh.vertices
            .push(Vertex::raw(MAX.x, MIN.y, MIN.z, 0.0, -1.0, 0.0, 0.0, 1.0));

        /*
        const VERTICES: &[[f32; 8]] = &[
            // Front
            [MIN.x, MIN.y, MAX.z, 0.0, 0.0, 1.0, 0.0, 0.0], //
            [MAX.x, MIN.y, MAX.z, 0.0, 0.0, 1.0, 1.0, 0.0], //
            [MAX.x, MAX.y, MAX.z, 0.0, 0.0, 1.0, 1.0, 1.0], //
            [MIN.x, MAX.y, MAX.z, 0.0, 0.0, 1.0, 0.0, 1.0], //
            // Back
            [MIN.x, MAX.y, MIN.z, 0.0, 0.0, -1.0, 1.0, 0.0], //
            [MAX.x, MAX.y, MIN.z, 0.0, 0.0, -1.0, 0.0, 0.0], //
            [MAX.x, MIN.y, MIN.z, 0.0, 0.0, -1.0, 0.0, 1.0], //
            [MIN.x, MIN.y, MIN.z, 0.0, 0.0, -1.0, 1.0, 1.0], //
            // Right
            [MAX.x, MIN.y, MIN.z, 1.0, 0.0, 0.0, 0.0, 0.0], //
            [MAX.x, MAX.y, MIN.z, 1.0, 0.0, 0.0, 1.0, 0.0], //
            [MAX.x, MAX.y, MAX.z, 1.0, 0.0, 0.0, 1.0, 1.0], //
            [MAX.x, MIN.y, MAX.z, 1.0, 0.0, 0.0, 0.0, 1.0], //
            // Left
            [MIN.x, MIN.y, MAX.z, -1.0, 0.0, 0.0, 1.0, 0.0], //
            [MIN.x, MAX.y, MAX.z, -1.0, 0.0, 0.0, 0.0, 0.0], //
            [MIN.x, MAX.y, MIN.z, -1.0, 0.0, 0.0, 0.0, 1.0], //
            [MIN.x, MIN.y, MIN.z, -1.0, 0.0, 0.0, 1.0, 1.0], //
            // Top
            [MAX.x, MAX.y, MIN.z, 0.0, 1.0, 0.0, 1.0, 0.0], //
            [MIN.x, MAX.y, MIN.z, 0.0, 1.0, 0.0, 0.0, 0.0], //
            [MIN.x, MAX.y, MAX.z, 0.0, 1.0, 0.0, 0.0, 1.0], //
            [MAX.x, MAX.y, MAX.z, 0.0, 1.0, 0.0, 1.0, 1.0], //
            // Bottom
            [MAX.x, MIN.y, MAX.z, 0.0, -1.0, 0.0, 0.0, 0.0], //
            [MIN.x, MIN.y, MAX.z, 0.0, -1.0, 0.0, 1.0, 0.0], //
            [MIN.x, MIN.y, MIN.z, 0.0, -1.0, 0.0, 1.0, 1.0], //
            [MAX.x, MIN.y, MIN.z, 0.0, -1.0, 0.0, 0.0, 1.0], //
        ];
        */

        mesh.indices.append(&mut vec![
            0, 1, 2, 2, 3, 0, // front
            4, 5, 6, 6, 7, 4, // back
            8, 9, 10, 10, 11, 8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // top
            20, 21, 22, 22, 23, 20, // bottom
        ]);

        let mesh = mesh.upload_to_gpu(renderer);

        // let main_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("main index buffer"),
        //     contents: bytemuck::cast_slice(VERTICES),
        //     usage: wgpu::BufferUsages::VERTEX,
        // });

        // let main_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("main index buffer"),
        //     contents: bytemuck::cast_slice(INDICES),
        //     usage: wgpu::BufferUsages::INDEX,
        // });

        let fullscreen_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fullscreen shader module"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("fullscreen.wgsl"))),
        });

        let fullscreen_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("fullscreen bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let fullscreen_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("fullscreen pipeline layout"),
                bind_group_layouts: &[&fullscreen_bind_group_layout],
                push_constant_ranges: &[],
            });

        let fullscreen_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("fullscreen render pipeline"),
                layout: Some(&fullscreen_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &fullscreen_module,
                    entry_point: "vertex_main",
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &fullscreen_module,
                    entry_point: "fragment_main",
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
                cache: None,
            });

        Self {
            depth_texture,
            mesh_render_pipeline,
            mesh,
            albedo_texture,
            position_texture,
            normal_texture,
            fullscreen_render_pipeline,
            fullscreen_bind_group_layout,
            uniforms_buffer,
            _uniforms_bind_group_layout: uniforms_bind_group_layout,
            uniforms_bind_group,

            rotating: None,
            last_mouse_position: (0.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,

            render_source: RenderSource::Albedo,
        }
    }

    pub fn resize(&mut self, renderer: &Renderer) {
        let Renderer {
            device,
            surface_config,
            ..
        } = renderer;

        self.depth_texture =
            create_depth_texture(device, surface_config.width, surface_config.height);
        self.albedo_texture = create_fullscreen_texture(device, surface_config, "albedo texture");
        self.position_texture =
            create_fullscreen_texture(device, surface_config, "position texture");
        self.normal_texture = create_fullscreen_texture(device, surface_config, "normal texture");
    }

    pub fn on_mouse_down(&mut self, button: winit::event::MouseButton) {
        if matches!(button, winit::event::MouseButton::Left) {
            self.rotating = Some(self.last_mouse_position);
        }
    }

    pub fn on_mouse_up(&mut self, button: winit::event::MouseButton) {
        if matches!(button, winit::event::MouseButton::Left) {
            self.rotating = None;
        }
    }

    pub fn on_mouse_moved(&mut self, x: f32, y: f32) {
        self.last_mouse_position = (x, y);

        if let Some(ref mut start_drag_position) = self.rotating {
            let delta = (x - start_drag_position.0, y - start_drag_position.1);

            self.yaw += delta.0;
            self.pitch += delta.1;

            *start_drag_position = (x, y);
        }
    }

    pub fn on_key_pressed(&mut self, key_code: winit::keyboard::KeyCode) {
        match key_code {
            KeyCode::KeyR => {
                self.pitch = 0.0;
                self.yaw = 0.0;
            }

            KeyCode::Digit1 => {
                self.render_source = RenderSource::Albedo;
            }

            KeyCode::Digit2 => {
                self.render_source = RenderSource::Position;
            }

            KeyCode::Digit3 => {
                self.render_source = RenderSource::Normal;
            }

            _ => {}
        }
    }

    pub fn render(&self, renderer: &Renderer) {
        let Renderer {
            device,
            queue,
            surface,
            surface_config,
        } = renderer;

        let aspect_ratio = surface_config.width as f32 / (surface_config.height as f32).max(0.001);

        let projection_matrix = cgmath::perspective(cgmath::Deg(45.0), aspect_ratio, 0.1, 1000.0);
        let projection_inv_matrix = projection_matrix.invert().unwrap().transpose();

        let distance = 4.0;
        let view_matrix = {
            let yaw_quat = cgmath::Quaternion::from_angle_y(cgmath::Deg(self.yaw));
            let pitch_quat = cgmath::Quaternion::from_angle_x(cgmath::Deg(self.pitch));
            let rotation_quat = yaw_quat * pitch_quat;
            let rotation_matrix = cgmath::Matrix4::from(rotation_quat);
            let translation_matrix =
                cgmath::Matrix4::from_translation(cgmath::Vector3::new(0.0, 0.0, -distance));
            translation_matrix * rotation_matrix
        };

        let model_matrix = cgmath::Matrix4::identity();

        let uniforms = Uniforms {
            projection_matrix: projection_matrix.into(),
            projection_inv_matrix: projection_inv_matrix.into(),
            view_matrix: view_matrix.into(),
            model_matrix: model_matrix.into(),
        };

        queue.write_buffer(&self.uniforms_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("main command encoder"),
        });

        let output = surface.get_current_texture().expect("get current texture");

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("albedo render pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.albedo_texture.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.position_texture.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.normal_texture.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.mesh_render_pipeline.pipeline);
            render_pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
            render_pass
                .set_index_buffer(self.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_bind_group(0, &self.uniforms_bind_group, &[]);
            render_pass.draw_indexed(0..self.mesh.index_count, 0, 0..1);
        }

        {
            let fullscreen_texture = match self.render_source {
                RenderSource::Albedo => &self.albedo_texture,
                RenderSource::Position => &self.position_texture,
                RenderSource::Normal => &self.normal_texture,
            };

            let fullscreen_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("fullscreen bind group"),
                layout: &self.fullscreen_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&fullscreen_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&fullscreen_texture.sampler),
                    },
                ],
            });

            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fullscreen render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.fullscreen_render_pipeline);
            render_pass.set_bind_group(0, &fullscreen_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
