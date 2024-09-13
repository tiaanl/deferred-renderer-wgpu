use wgpu::util::DeviceExt;

use crate::Renderer;

pub struct UserInterface {
    texture_manager: epaint::TextureManager,
    textures:
        std::collections::HashMap<epaint::TextureId, (Option<wgpu::Texture>, wgpu::BindGroup)>,
    samplers: std::collections::HashMap<epaint::textures::TextureOptions, wgpu::Sampler>,
    fonts: epaint::Fonts,

    screen_size_buffer: wgpu::Buffer,
    screen_size_bind_group: wgpu::BindGroup,

    texture_bind_group_layout: wgpu::BindGroupLayout,

    pipeline: wgpu::RenderPipeline,

    pub shapes: Vec<epaint::ClippedShape>,
}

impl UserInterface {
    pub fn new(renderer: &Renderer) -> Self {
        let mut texture_manager = epaint::TextureManager::default();
        let font_texture_id = texture_manager.alloc(
            "font-texture".into(),
            epaint::FontImage::new([0, 0]).into(),
            Default::default(),
        );
        assert_eq!(font_texture_id, epaint::TextureId::default());

        let fonts = epaint::Fonts::new(1.0, 1024, epaint::text::FontDefinitions::default());

        let module = renderer
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("epaint shader module"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                    "epaint.wgsl"
                ))),
            });

        let screen_size: [f32; 2] = [1600.0, 900.0];

        let screen_size_buffer =
            renderer
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("epaint screen_size buffer"),
                    contents: bytemuck::cast_slice(&screen_size),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let screen_size_bind_group_layout =
            renderer
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("epaint screen_size bind group layout"),
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

        let screen_size_bind_group =
            renderer
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("epaint screen_size bind group"),
                    layout: &screen_size_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: screen_size_buffer.as_entire_binding(),
                    }],
                });

        let texture_bind_group_layout = {
            renderer
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("egui_texture_bind_group_layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
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
                })
        };

        let pipeline_layout =
            renderer
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("epaint pipeline layout"),
                    bind_group_layouts: &[
                        &screen_size_bind_group_layout,
                        &texture_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let pipeline = renderer
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("epaint render pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: "vertex_main",
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<epaint::Vertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![
                            0 => Float32x2,
                            1 => Float32x2,
                            2 => Uint32,
                        ],
                    }],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: "fragment_main",
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8UnormSrgb,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
                cache: None,
            });

        Self {
            texture_manager,
            textures: std::collections::HashMap::new(),
            samplers: std::collections::HashMap::new(),
            fonts,

            screen_size_buffer,
            screen_size_bind_group,

            texture_bind_group_layout,

            pipeline,

            shapes: vec![],
        }
    }

    pub fn render_text(
        &mut self,
        text: impl Into<String>,
        position: [f32; 2],
        size: f32,
        color: epaint::Color32,
    ) {
        let galley = self
            .fonts
            .layout_no_wrap(text.into(), epaint::FontId::monospace(size), color);

        let shape = epaint::TextShape::new(epaint::pos2(position[0], position[1]), galley, color);

        self.shapes.push(epaint::ClippedShape {
            clip_rect: epaint::Rect::EVERYTHING,
            shape: epaint::Shape::Text(shape),
        });
    }

    pub fn resize(&mut self, renderer: &Renderer, size: [f32; 2]) {
        renderer
            .queue
            .write_buffer(&self.screen_size_buffer, 0, bytemuck::cast_slice(&size));
    }

    fn create_sampler(
        renderer: &Renderer,
        options: epaint::textures::TextureOptions,
    ) -> wgpu::Sampler {
        use epaint::textures::{TextureFilter, TextureWrapMode};

        let mag_filter = match options.magnification {
            TextureFilter::Nearest => wgpu::FilterMode::Nearest,
            TextureFilter::Linear => wgpu::FilterMode::Linear,
        };

        let min_filter = match options.minification {
            TextureFilter::Nearest => wgpu::FilterMode::Nearest,
            TextureFilter::Linear => wgpu::FilterMode::Linear,
        };

        let address_mode = match options.wrap_mode {
            TextureWrapMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
            TextureWrapMode::Repeat => wgpu::AddressMode::Repeat,
            TextureWrapMode::MirroredRepeat => wgpu::AddressMode::MirrorRepeat,
        };

        let label = format!("egui sampler (mag: {mag_filter:?}, min: {min_filter:?})");
        println!("creating sampler: {}", label);

        renderer.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(&label),
            mag_filter,
            min_filter,
            address_mode_u: address_mode,
            address_mode_v: address_mode,
            ..Default::default()
        })
    }

    fn update_texture(
        &mut self,
        renderer: &Renderer,
        texture_id: epaint::TextureId,
        image_delta: epaint::ImageDelta,
    ) {
        let width = image_delta.image.width() as u32;
        let height = image_delta.image.height() as u32;

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let data_color32 = match &image_delta.image {
            epaint::ImageData::Color(image) => {
                assert_eq!(
                    width as usize * height as usize,
                    image.pixels.len(),
                    "Mismatch between texture size and texel count"
                );
                std::borrow::Cow::Borrowed(&image.pixels)
            }
            epaint::ImageData::Font(image) => {
                assert_eq!(
                    width as usize * height as usize,
                    image.pixels.len(),
                    "Mismatch between texture size and texel count"
                );
                std::borrow::Cow::Owned(image.srgba_pixels(None).collect::<Vec<epaint::Color32>>())
            }
        };
        let data_bytes: &[u8] = bytemuck::cast_slice(data_color32.as_slice());

        let queue_write_data_to_texture = |texture, origin| {
            renderer.queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture,
                    mip_level: 0,
                    origin,
                    aspect: wgpu::TextureAspect::All,
                },
                data_bytes,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * width),
                    rows_per_image: Some(height),
                },
                size,
            );
        };

        if let Some(pos) = image_delta.pos {
            println!("updating existing texture: {:?}", texture_id);
            // update the existing texture
            let (texture, _) = self
                .textures
                .get(&texture_id)
                .expect("Tried to update a texture that has not been allocated yet.");
            let origin = wgpu::Origin3d {
                x: pos[0] as u32,
                y: pos[1] as u32,
                z: 0,
            };
            queue_write_data_to_texture(
                texture.as_ref().expect("Tried to update user texture."),
                origin,
            );
        } else {
            println!("creating new texture: {:?}", texture_id);

            let label_str = format!("texture_{texture_id:?}");
            let label = Some(label_str.as_str());
            let texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
                label,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
            });

            let sampler = self
                .samplers
                .entry(image_delta.options)
                .or_insert_with(|| Self::create_sampler(renderer, image_delta.options));

            let bind_group = renderer
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label,
                    layout: &self.texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(sampler),
                        },
                    ],
                });

            let origin = wgpu::Origin3d::ZERO;
            queue_write_data_to_texture(&texture, origin);
            self.textures
                .insert(texture_id, (Some(texture), bind_group));
        };
    }

    pub fn render(
        &mut self,
        renderer: &Renderer,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        if let Some(font_image_delta) = self.fonts.font_image_delta() {
            self.texture_manager
                .set(epaint::TextureId::default(), font_image_delta);
        }

        let tessellation_options = epaint::TessellationOptions::default();
        let texture_atlas = self.fonts.texture_atlas();
        let (font_tex_size, prepared_discs) = {
            let atlas = texture_atlas.lock();
            (atlas.size(), atlas.prepared_discs())
        };

        let mut tessellator =
            epaint::Tessellator::new(1.0, tessellation_options, font_tex_size, prepared_discs);

        let shapes = std::mem::take(&mut self.shapes);
        let primitives = tessellator.tessellate_shapes(shapes);

        let texture_deltas = self.texture_manager.take_delta();

        for (texture_id, image_delta) in texture_deltas.set {
            self.update_texture(renderer, texture_id, image_delta);
        }

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("epaint render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
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

        for primitive in primitives.into_iter() {
            match primitive.primitive {
                epaint::Primitive::Mesh(mesh) => {
                    let (_, texture_bind_group) = self
                        .textures
                        .get(&mesh.texture_id)
                        .expect("texture not uploaded");

                    let buffers = crate::mesh::Mesh::from(mesh).upload_to_gpu(renderer);

                    render_pass.set_pipeline(&self.pipeline);
                    render_pass.set_vertex_buffer(0, buffers.vertex_buffer.slice(..));
                    render_pass.set_index_buffer(
                        buffers.index_buffer.slice(..),
                        wgpu::IndexFormat::Uint16,
                    );
                    render_pass.set_bind_group(0, &self.screen_size_bind_group, &[]);
                    render_pass.set_bind_group(1, texture_bind_group, &[]);
                    render_pass.draw_indexed(0..buffers.index_count, 0, 0..1);
                }

                epaint::Primitive::Callback(..) => todo!(),
            }
        }
    }
}
