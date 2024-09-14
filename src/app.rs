use std::{borrow::Cow, sync::Arc};

use cgmath::Angle;
use winit::keyboard::KeyCode;

use crate::{
    camera::Camera,
    gizmos::Gizmos,
    lights::{Lights, PointLight},
    material::GpuMaterial,
    mesh::{GpuMesh, Mesh, Vertex},
    mesh_render_pipeline::MeshRenderPipeline,
    texture::{create_depth_texture, create_fullscreen_texture, Texture},
    ui, Renderer,
};

enum RenderSource {
    Final,
    Albedo,
    Position,
    Normal,
}

pub struct App {
    depth_texture: Texture,

    mesh_render_pipeline: MeshRenderPipeline,

    mesh: GpuMesh,
    material: crate::material::GpuMaterial,

    albedo_g_texture: Texture,
    position_g_texture: Texture,
    normal_g_texture: Texture,

    fullscreen_render_pipeline: wgpu::RenderPipeline,
    debug_render_pipeline: wgpu::RenderPipeline,
    fullscreen_bind_group_layout: wgpu::BindGroupLayout,

    camera: Camera,

    lights: Lights,

    rotating: Option<(f32, f32)>,
    last_mouse_position: (f32, f32),
    yaw: cgmath::Deg<f32>,
    pitch: cgmath::Deg<f32>,
    distance: f32,

    render_source: RenderSource,

    light_angle: Option<cgmath::Deg<f32>>,

    gizmos: Gizmos,

    last_frame_time: std::time::Instant,

    ui: ui::UserInterface,

    sliders: slotmap::SlotMap<ui::SliderId, ui::Slider>,
    active_slider: Option<ui::SliderId>,
    light_x_id: ui::SliderId,
    light_y_id: ui::SliderId,
    light_z_id: ui::SliderId,
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
        let albedo_g_texture = create_fullscreen_texture(
            device,
            surface_config,
            wgpu::TextureFormat::Bgra8UnormSrgb,
            "albedo texture",
        );
        let position_g_texture = create_fullscreen_texture(
            device,
            surface_config,
            wgpu::TextureFormat::Rgba16Float,
            "position texture",
        );
        let normal_g_texture = create_fullscreen_texture(
            device,
            surface_config,
            wgpu::TextureFormat::Rgba16Float,
            "normal texture",
        );

        let reader =
            std::io::BufReader::new(std::io::Cursor::new(include_bytes!("../res/cube.obj")));
        let mut mesh = Mesh::<Vertex>::from_reader(reader).unwrap();
        mesh.update_tangents();
        let mesh = mesh.upload_to_gpu(renderer);

        let material = GpuMaterial::new(
            renderer,
            include_bytes!("../res/metal/albedo.png"),
            include_bytes!("../res/metal/normal.png"),
        );

        let camera = Camera::new(renderer);

        let lights = Lights::new(renderer, PointLight::new([3.0, 3.0, 3.0], [1.0, 1.0, 1.0]));

        let mesh_render_pipeline = MeshRenderPipeline::new(
            renderer,
            &camera.bind_group_layout,
            &material.bind_group_layout,
        );

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
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let fullscreen_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("fullscreen pipeline layout"),
                bind_group_layouts: &[
                    &fullscreen_bind_group_layout,
                    &camera.bind_group_layout,
                    &lights.bind_group_layout,
                ],
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

        let debug_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("debug render pipeline"),
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
                    entry_point: "fragment_debug",
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

        let gizmos = Gizmos::new(renderer, &camera);

        let ui = ui::UserInterface::new(renderer);

        let light_x = ui::Slider::new(Arc::clone(&ui.fonts), "Light X").with_min_max(-5.0, 5.0);
        let light_y = ui::Slider::new(Arc::clone(&ui.fonts), "Light Y").with_min_max(-5.0, 5.0);
        let light_z = ui::Slider::new(Arc::clone(&ui.fonts), "Light Z").with_min_max(-5.0, 5.0);

        let mut sliders = slotmap::SlotMap::with_key();
        let light_x_key = sliders.insert(light_x);
        let light_y_key = sliders.insert(light_y);
        let light_z_key = sliders.insert(light_z);

        Self {
            depth_texture,
            mesh_render_pipeline,

            mesh,
            material,

            albedo_g_texture,
            position_g_texture,
            normal_g_texture,

            fullscreen_render_pipeline,
            debug_render_pipeline,
            fullscreen_bind_group_layout,

            camera,
            lights,

            rotating: None,
            last_mouse_position: (0.0, 0.0),
            yaw: cgmath::Deg(90.0),
            pitch: cgmath::Deg(0.0),
            distance: 10.0,

            render_source: RenderSource::Final,

            light_angle: None,

            gizmos,

            last_frame_time: std::time::Instant::now(),

            ui,
            sliders,
            active_slider: None,
            light_x_id: light_x_key,
            light_y_id: light_y_key,
            light_z_id: light_z_key,
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
        self.albedo_g_texture = create_fullscreen_texture(
            device,
            surface_config,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            "albedo texture",
        );
        self.position_g_texture = create_fullscreen_texture(
            device,
            surface_config,
            wgpu::TextureFormat::Rgba16Float,
            "position texture",
        );
        self.normal_g_texture = create_fullscreen_texture(
            device,
            surface_config,
            wgpu::TextureFormat::Rgba16Float,
            "normal texture",
        );

        self.ui.resize(
            renderer,
            [surface_config.width as f32, surface_config.height as f32],
        );

        const SLIDER_SIZE: epaint::Vec2 = epaint::Vec2 { x: 300.0, y: 40.0 };
        let mut top = 0.0;
        for (_, slider) in self.sliders.iter_mut() {
            slider.bounds = epaint::Rect {
                min: epaint::pos2(surface_config.width as f32 - 10.0 - SLIDER_SIZE.x, top),
                max: epaint::pos2(surface_config.width as f32 - 10.0, top + SLIDER_SIZE.y),
            };
            top += SLIDER_SIZE.y;
        }
    }

    pub fn on_mouse_down(&mut self, button: winit::event::MouseButton) {
        for (id, slider) in self.sliders.iter_mut() {
            let x = self.last_mouse_position.0;
            let y = self.last_mouse_position.1;
            if slider.bounds.contains(epaint::pos2(x, y)) && slider.on_mouse_down(x, y) {
                // If the widget returns true from an `on_mouse_down` event, we
                // lock that widget as the active one and start sending `on_mouse_dragged`
                // events instead of `on_mouse_moved` events when the mouse moves.
                self.active_slider = Some(id);
                return;
            }
        }

        if matches!(button, winit::event::MouseButton::Left) {
            self.rotating = Some(self.last_mouse_position);
        }
    }

    pub fn on_mouse_up(&mut self, button: winit::event::MouseButton) {
        if let Some(id) = self.active_slider {
            if let Some(ref mut slider) = self.sliders.get_mut(id) {
                slider.on_mouse_up();
                self.active_slider = None;
                return;
            }
        }

        self.active_slider = None;

        if matches!(button, winit::event::MouseButton::Left) {
            self.rotating = None;
        }
    }

    pub fn on_mouse_wheel(&mut self, delta: f32) {
        self.distance -= delta * (self.distance * 0.1);
    }

    pub fn on_mouse_moved(&mut self, x: f32, y: f32) {
        self.last_mouse_position = (x, y);

        if let Some(id) = self.active_slider {
            if let Some(ref mut slider) = self.sliders.get_mut(id) {
                slider.on_mouse_dragged(x, y);
                return;
            }
        } else {
            for (_, slider) in self.sliders.iter_mut() {
                if slider.bounds.contains(epaint::pos2(x, y)) {
                    slider.on_mouse_moved(x, y);
                    return;
                }
            }
        }

        if let Some(ref mut start_drag_position) = self.rotating {
            let delta = (x - start_drag_position.0, y - start_drag_position.1);

            self.yaw += cgmath::Deg(delta.0);
            self.pitch += cgmath::Deg(delta.1);

            *start_drag_position = (x, y);
        }
    }

    pub fn on_key_pressed(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::KeyR => {
                self.pitch = cgmath::Deg(0.0);
                self.yaw = cgmath::Deg(0.0);
            }

            KeyCode::Digit1 => {
                self.render_source = RenderSource::Final;
            }

            KeyCode::Digit2 => {
                self.render_source = RenderSource::Albedo;
            }

            KeyCode::Digit3 => {
                self.render_source = RenderSource::Position;
            }

            KeyCode::Digit4 => {
                self.render_source = RenderSource::Normal;
            }

            KeyCode::KeyL => {
                if self.light_angle.is_none() {
                    self.light_angle = Some(cgmath::Deg(0.0));
                } else {
                    self.light_angle = None;
                }
            }

            KeyCode::ArrowLeft => {
                self.lights.point_light.position[0] -= 0.5;
            }
            KeyCode::ArrowRight => {
                self.lights.point_light.position[0] += 0.5;
            }

            KeyCode::ArrowUp => {
                self.lights.point_light.position[2] += 0.5;
            }
            KeyCode::ArrowDown => {
                self.lights.point_light.position[2] -= 0.5;
            }

            KeyCode::PageUp => {
                self.lights.point_light.position[1] += 0.5;
            }
            KeyCode::PageDown => {
                self.lights.point_light.position[1] -= 0.5;
            }

            _ => {}
        }
    }

    pub fn on_key_released(&mut self, _key_code: KeyCode) {}

    pub fn render(&mut self, renderer: &Renderer) {
        let Renderer {
            device,
            queue,
            surface,
            surface_config,
        } = renderer;

        let now = std::time::Instant::now();
        let last_frame_duration = now - self.last_frame_time;
        self.last_frame_time = now;

        let fps = 1.0 / last_frame_duration.as_secs_f32();
        self.ui.push_shape(epaint::ClippedShape {
            clip_rect: epaint::Rect::EVERYTHING,
            shape: epaint::Shape::Text(epaint::TextShape::new(
                epaint::pos2(10.0, 10.0),
                self.ui.fonts.layout_no_wrap(
                    format!("fps: {:0.2}", fps),
                    epaint::FontId::monospace(16.0),
                    epaint::Color32::GREEN,
                ),
                epaint::Color32::default(),
            )),
        });

        let time_delta = 1.0 / ((1.0 / 60.0) / last_frame_duration.as_secs_f32());

        if let Some(ref mut light_angle) = self.light_angle {
            *light_angle += cgmath::Deg(1.0 * time_delta);
            let x = light_angle.cos() * 3.0;
            let y = light_angle.sin() * 3.0;
            self.lights.move_to(renderer, [x, 1.0, y]);
        } else {
            let x = self.sliders.get(self.light_x_id).unwrap().value();
            let y = self.sliders.get(self.light_y_id).unwrap().value();
            let z = self.sliders.get(self.light_z_id).unwrap().value();

            // self.lights.move_to(renderer, self.lights.point_light.position);
            self.lights.move_to(renderer, [x, y, z]);
        }

        let aspect_ratio = surface_config.width as f32 / (surface_config.height as f32).max(0.001);

        let projection_matrix = cgmath::perspective(cgmath::Deg(45.0), aspect_ratio, 0.01, 100.0);

        let (camera_position, view_matrix) = {
            // Calculate the camera position
            let camera_x = self.distance * self.yaw.cos() * self.pitch.cos();
            let camera_y = self.distance * self.pitch.sin();
            let camera_z = self.distance * self.yaw.sin() * self.pitch.cos();

            let camera_position = cgmath::Point3::new(camera_x, camera_y, camera_z);

            let target = cgmath::Point3::new(0.0, 0.0, 0.0);
            let up = cgmath::Vector3::unit_y();
            (
                camera_position,
                cgmath::Matrix4::look_at_rh(camera_position, target, up),
            )
        };

        self.camera
            .set_matrices(renderer, projection_matrix, view_matrix, camera_position);

        self.gizmos
            .draw_axis(self.lights.point_light.position.into());

        let output = surface.get_current_texture().expect("get current texture");

        let surface_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("main command encoder"),
        });

        encoder.clear_texture(&output.texture, &wgpu::ImageSubresourceRange::default());

        encoder.clear_texture(
            &self.albedo_g_texture.texture,
            &wgpu::ImageSubresourceRange::default(),
        );
        encoder.clear_texture(
            &self.position_g_texture.texture,
            &wgpu::ImageSubresourceRange::default(),
        );
        encoder.clear_texture(
            &self.normal_g_texture.texture,
            &wgpu::ImageSubresourceRange::default(),
        );

        if true {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("gbuffer render pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.albedo_g_texture.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.position_g_texture.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.normal_g_texture.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
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
            render_pass.set_bind_group(0, &self.camera.bind_group, &[]);
            render_pass.set_bind_group(1, &self.material.bind_group, &[]);
            render_pass.set_bind_group(2, &self.lights.bind_group, &[]);
            render_pass.draw_indexed(0..self.mesh.index_count, 0, 0..1);
        }

        if true {
            let fullscreen_bind_group = if matches!(self.render_source, RenderSource::Final) {
                // let fullscreen_texture = match self.render_source {
                //     RenderSource::Albedo => &self.albedo_g_texture,
                //     RenderSource::Position => &self.position_g_texture,
                //     RenderSource::Normal => &self.normal_g_texture,
                // };

                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("fullscreen bind group"),
                    layout: &self.fullscreen_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&self.depth_texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(
                                &self.albedo_g_texture.view,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(
                                &self.albedo_g_texture.sampler,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::TextureView(
                                &self.position_g_texture.view,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: wgpu::BindingResource::Sampler(
                                &self.position_g_texture.sampler,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: wgpu::BindingResource::TextureView(
                                &self.normal_g_texture.view,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: wgpu::BindingResource::Sampler(
                                &self.normal_g_texture.sampler,
                            ),
                        },
                    ],
                })
            } else {
                let fullscreen_texture = match self.render_source {
                    RenderSource::Albedo => &self.albedo_g_texture,
                    RenderSource::Position => &self.position_g_texture,
                    RenderSource::Normal => &self.normal_g_texture,
                    RenderSource::Final => unreachable!("handled above"),
                };

                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("fullscreen bind group"),
                    layout: &self.fullscreen_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&self.depth_texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&fullscreen_texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(&fullscreen_texture.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::TextureView(&fullscreen_texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: wgpu::BindingResource::Sampler(&fullscreen_texture.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: wgpu::BindingResource::TextureView(&fullscreen_texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: wgpu::BindingResource::Sampler(&fullscreen_texture.sampler),
                        },
                    ],
                })
            };

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fullscreen render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
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
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if matches!(self.render_source, RenderSource::Final) {
                render_pass.set_pipeline(&self.fullscreen_render_pipeline);
            } else {
                render_pass.set_pipeline(&self.debug_render_pipeline);
            }
            render_pass.set_bind_group(0, &fullscreen_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera.bind_group, &[]);
            render_pass.set_bind_group(2, &self.lights.bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        if true {
            self.gizmos.render(
                renderer,
                &mut encoder,
                &surface_view,
                &self.depth_texture.view,
                &self.camera,
            );
        }

        for (_, slider) in self.sliders.iter_mut() {
            let shapes = slider.shapes();
            self.ui.push_shapes(shapes);
        }

        self.ui.render(renderer, &mut encoder, &surface_view);

        queue.submit(std::iter::once(encoder.finish()));

        output.present();
    }
}
