use wgpu::util::DeviceExt;

use crate::Renderer;

#[derive(Clone, Copy, bytemuck::NoUninit)]
#[repr(C)]
pub struct PointLight {
    pub position: [f32; 3],
    pub _padding1: f32,
    pub color: [f32; 3],
    pub _padding2: f32,
}

impl PointLight {
    pub fn new(position: [f32; 3], color: [f32; 3]) -> Self {
        Self {
            position,
            _padding1: 0.0,
            color,
            _padding2: 0.0,
        }
    }
}

pub struct Lights {
    pub point_light: PointLight,
    buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl Lights {
    pub fn new(renderer: &Renderer, point_light: PointLight) -> Self {
        let bind_group_layout =
            renderer
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("lights bind group layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("lights buffer"),
                contents: bytemuck::cast_slice(&[point_light]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group = renderer
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("lights bind group"),
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });

        Self {
            point_light,
            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn move_to(&mut self, renderer: &Renderer, position: [f32; 3]) {
        self.point_light.position = position;
        renderer
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.point_light]));
    }
}
