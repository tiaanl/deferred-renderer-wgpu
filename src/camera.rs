use cgmath::SquareMatrix;
use wgpu::util::DeviceExt;

use crate::Renderer;

pub struct Camera {
    buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

#[derive(Clone, Copy, bytemuck::NoUninit)]
#[repr(C)]
struct GpuCamera {
    projection_matrix: [[f32; 4]; 4],
    view_matrix: [[f32; 4]; 4],
    position: [f32; 3],
    _padding: f32,
}

impl Camera {
    pub fn new(renderer: &Renderer) -> Self {
        let projection_matrix = cgmath::Matrix4::identity().into();
        let view_matrix = cgmath::Matrix4::identity().into();

        let data = GpuCamera {
            projection_matrix,
            view_matrix,
            position: [0.0, 0.0, 0.0],
            _padding: 0.0,
        };

        let buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("camera bind group layout"),
                contents: bytemuck::cast_slice(&[data]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group_layout =
            renderer
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("camera bind group layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let bind_group = renderer
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("camera bind group"),
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });

        Self {
            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn set_matrices(
        &mut self,
        renderer: &Renderer,
        projection_matrix: cgmath::Matrix4<f32>,
        view_matrix: cgmath::Matrix4<f32>,
        camera_position: cgmath::Point3<f32>,
    ) {
        let gpu_camera = GpuCamera {
            projection_matrix: projection_matrix.into(),
            view_matrix: view_matrix.into(),
            position: camera_position.into(),
            _padding: 0.0,
        };

        renderer
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[gpu_camera]));
    }
}
