use wgpu::util::DeviceExt;

use crate::{
    camera::Camera,
    mesh::{GpuMesh, Mesh},
    texture::DEPTH_FORMAT,
    Renderer,
};

#[derive(Clone, Copy, bytemuck::NoUninit)]
#[repr(C)]
struct Vertex {
    position: [f32; 3],
    _padding: f32,
    color: [f32; 4],
}

impl Vertex {
    fn new(position: [f32; 3], color: [f32; 4]) -> Self {
        Self {
            position,
            _padding: 0.0,
            color,
        }
    }
}

pub struct Gizmos {
    pipeline: wgpu::RenderPipeline,

    axis_mesh: GpuMesh,
    axis: Vec<[f32; 3]>,
}

impl Gizmos {
    pub fn new(renderer: &Renderer, camera: &Camera) -> Self {
        let module = renderer
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("gizmos module"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                    "gizmos.wgsl"
                ))),
            });

        let pipeline_layout =
            renderer
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("gizmos pipeline layout"),
                    bind_group_layouts: &[&camera.bind_group_layout],
                    push_constant_ranges: &[],
                });

        let pipeline = renderer
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("gizmos render pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: "vertex_main",
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x3,
                                    offset: 0,
                                    shader_location: 0,
                                },
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x4,
                                    offset: (4 * std::mem::size_of::<f32>()) as wgpu::BufferAddress,
                                    shader_location: 1,
                                },
                            ],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &[wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x3,
                                offset: 0,
                                shader_location: 2,
                            }],
                        },
                    ],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::LineList,
                    ..wgpu::PrimitiveState::default()
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
                        format: wgpu::TextureFormat::Bgra8UnormSrgb,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
                cache: None,
            });

        let axis_mesh = Mesh {
            vertices: vec![
                // X
                Vertex::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0, 1.0]),
                Vertex::new([1.0, 0.0, 0.0], [1.0, 0.0, 0.0, 1.0]),
                // Y
                Vertex::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 1.0]),
                Vertex::new([0.0, 1.0, 0.0], [0.0, 1.0, 0.0, 1.0]),
                // Z
                Vertex::new([0.0, 0.0, 0.0], [0.0, 0.0, 1.0, 1.0]),
                Vertex::new([0.0, 0.0, 1.0], [0.0, 0.0, 1.0, 1.0]),
            ],
            indices: vec![0, 1, 2, 3, 4, 5],
        }
        .upload_to_gpu(renderer);

        Self {
            pipeline,
            axis_mesh,
            axis: vec![],
        }
    }

    pub fn draw_axis(&mut self, position: cgmath::Vector3<f32>) {
        self.axis.push(position.into());
    }

    pub fn render(
        &mut self,
        renderer: &Renderer,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        camera: &Camera,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("gizmos render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        let instance_buffer =
            renderer
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("gizmos axis instances"),
                    contents: bytemuck::cast_slice(self.axis.as_ref()),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.axis_mesh.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        render_pass.set_index_buffer(
            self.axis_mesh.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        render_pass.set_bind_group(0, &camera.bind_group, &[]);
        render_pass.draw_indexed(0..self.axis_mesh.index_count, 0, 0..self.axis.len() as u32);

        self.axis.clear();
    }
}
