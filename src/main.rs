use std::{borrow::Cow, sync::Arc};

use cgmath::{Rotation3, SquareMatrix};
use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::WindowId,
};

struct App {
    window: Arc<winit::window::Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    depth_texture: Texture,

    main_render_pipeline: wgpu::RenderPipeline,
    main_vertex_buffer: wgpu::Buffer,
    main_index_buffer: wgpu::Buffer,
    main_num_indices: u32,

    albedo_texture: Texture,

    fullscreen_render_pipeline: wgpu::RenderPipeline,
    fullscreen_bind_group: wgpu::BindGroup,

    uniforms_buffer: wgpu::Buffer,
    _uniforms_bind_group_layout: wgpu::BindGroupLayout,
    uniforms_bind_group: wgpu::BindGroup,

    rotating: Option<(f32, f32)>,
    last_mouse_position: (f32, f32),
    yaw: f32,
    pitch: f32,
}

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

#[allow(unused)]
struct Texture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}

fn create_depth_texture(device: &wgpu::Device, width: u32, height: u32) -> Texture {
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        lod_min_clamp: 0.0,
        lod_max_clamp: 100.0,
        compare: Some(wgpu::CompareFunction::LessEqual),
        ..Default::default()
    });

    Texture {
        texture,
        view,
        sampler,
    }
}

fn create_fullscreen_texture(
    device: &wgpu::Device,
    surface_config: &wgpu::SurfaceConfiguration,
) -> Texture {
    let size = wgpu::Extent3d {
        width: surface_config.width,
        height: surface_config.height,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("fullscreen_texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        lod_min_clamp: 0.0,
        lod_max_clamp: 100.0,
        compare: None, //Some(wgpu::CompareFunction::LessEqual),
        ..Default::default()
    });

    Texture {
        texture,
        view,
        sampler,
    }
}

impl App {
    fn resize(&mut self, size: PhysicalSize<u32>) {
        let PhysicalSize { width, height } = size;
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
    }

    fn render(&self) {
        let aspect_ratio =
            self.surface_config.width as f32 / (self.surface_config.height as f32).max(0.001);

        let projection_matrix = cgmath::perspective(cgmath::Deg(45.0), aspect_ratio, 0.1, 1000.0);

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
            view_matrix: view_matrix.into(),
            model_matrix: model_matrix.into(),
        };
        self.queue
            .write_buffer(&self.uniforms_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        let output = self
            .surface
            .get_current_texture()
            .expect("get current texture");

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("main command encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
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

            render_pass.set_pipeline(&self.main_render_pipeline);
            render_pass.set_vertex_buffer(0, self.main_vertex_buffer.slice(..));
            render_pass
                .set_index_buffer(self.main_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_bind_group(0, &self.uniforms_bind_group, &[]);
            render_pass.draw_indexed(0..self.main_num_indices, 0, 0..1);

            render_pass.set_pipeline(&self.fullscreen_render_pipeline);
            render_pass.set_bind_group(0, &self.fullscreen_bind_group, &[]);
            render_pass.draw_indexed(0..3, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

enum AppState {
    Uninitialized,
    Initialized(App),
}

#[derive(Clone, Copy, bytemuck::NoUninit)]
#[repr(C)]
struct Uniforms {
    projection_matrix: [[f32; 4]; 4],
    view_matrix: [[f32; 4]; 4],
    model_matrix: [[f32; 4]; 4],
}

impl ApplicationHandler for AppState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    winit::window::WindowAttributes::default()
                        .with_title("Test wGPU")
                        .with_inner_size(LogicalSize::new(900, 600))
                        .with_resizable(false),
                )
                .expect("create window"),
        );

        let PhysicalSize { width, height } = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .expect("request adapter");

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))
                .expect("request device");

        let surface = instance
            .create_surface(Arc::clone(&window))
            .expect("create surface");

        let surface_caps = surface.get_capabilities(&adapter);

        // Find a sRGB surface format or use the first.
        let format = surface_caps
            .formats
            .iter()
            .find(|cap| cap.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let mut surface_config = surface
            .get_default_config(&adapter, width, height)
            .expect("surface get default configuration");
        surface_config.format = format;
        surface_config.present_mode = wgpu::PresentMode::AutoNoVsync;

        surface.configure(&device, &surface_config);

        let albedo_texture = create_fullscreen_texture(&device, &surface_config);

        // Uniforms

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

        // create render pipeline

        let depth_texture = create_depth_texture(&device, width, height);

        let main_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("main bind group layout"),
            bind_group_layouts: &[&uniforms_bind_group_layout],
            push_constant_ranges: &[],
        });

        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("main shader module"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let main_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("main pipeline"),
            layout: Some(&main_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: "vertex_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 32 as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 12,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 24,
                            shader_location: 2,
                        },
                    ],
                }],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Front),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
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

        const MIN: cgmath::Vector3<f32> = cgmath::vec3(-0.5, -0.5, -0.5);
        const MAX: cgmath::Vector3<f32> = cgmath::vec3(0.5, 0.5, 0.5);

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

        const INDICES: &[u16] = &[
            0, 1, 2, 2, 3, 0, // front
            4, 5, 6, 6, 7, 4, // back
            8, 9, 10, 10, 11, 8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // top
            20, 21, 22, 22, 23, 20, // bottom
        ];

        let main_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("main index buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let main_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("main index buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

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

        let fullscreen_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fullscreen bind group"),
            layout: &fullscreen_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&albedo_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&albedo_texture.sampler),
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
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
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

        //
        *self = Self::Initialized(App {
            window,
            device,
            queue,
            surface,
            surface_config,
            depth_texture,
            main_render_pipeline,

            albedo_texture,

            fullscreen_render_pipeline,
            fullscreen_bind_group,

            main_vertex_buffer,
            main_index_buffer,
            main_num_indices: INDICES.len() as u32,

            uniforms_buffer,
            _uniforms_bind_group_layout: uniforms_bind_group_layout,
            uniforms_bind_group,

            rotating: None,
            last_mouse_position: (0.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
        })
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        use winit::event::WindowEvent;

        match event {
            WindowEvent::CloseRequested => _event_loop.exit(),

            WindowEvent::Resized(size) => {
                let Self::Initialized(app) = self else {
                    return;
                };

                app.resize(size);
                app.window.request_redraw();
            }

            WindowEvent::RedrawRequested => {
                let Self::Initialized(app) = self else {
                    return;
                };

                app.render();
                app.window.request_redraw();
            }

            WindowEvent::MouseInput { button, state, .. } => {
                let Self::Initialized(app) = self else {
                    return;
                };

                if matches!(button, winit::event::MouseButton::Left) {
                    if matches!(state, winit::event::ElementState::Pressed) {
                        app.rotating = Some(app.last_mouse_position);
                    } else {
                        app.rotating = None
                    }
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                let Self::Initialized(app) = self else {
                    return;
                };

                let pos = (position.x as f32, position.y as f32);
                app.last_mouse_position = pos;
                if let Some(ref mut start_drag_position) = app.rotating {
                    let delta = (pos.0 - start_drag_position.0, pos.1 - start_drag_position.1);

                    app.yaw += delta.0;
                    app.pitch += delta.1;

                    *start_drag_position = pos;
                }
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if event.physical_key == PhysicalKey::Code(KeyCode::KeyR) {
                    let Self::Initialized(app) = self else {
                        return;
                    };

                    app.pitch = 0.0;
                    app.yaw = 0.0;
                }
            }

            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("create event loop");
    let mut app = AppState::Uninitialized;
    event_loop.run_app(&mut app).expect("run app")
}
