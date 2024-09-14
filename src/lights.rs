use wgpu::util::DeviceExt;

use crate::Renderer;

#[derive(Clone, Copy, bytemuck::NoUninit)]
#[repr(C)]
pub struct PointLight {
    pub position: [f32; 3],
    pub intensity: f32,
    pub color: [f32; 3],
    pub shininess: f32,
    pub ambient: f32,
    _dummy: [f32; 3],
}

impl PointLight {
    pub fn new(
        position: [f32; 3],
        intensity: f32,
        color: [f32; 3],
        shininess: f32,
        ambient: f32,
    ) -> Self {
        Self {
            position,
            intensity,
            color,
            shininess,
            ambient,
            _dummy: [0.0; 3],
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

    pub fn move_to(
        &mut self,
        renderer: &Renderer,
        position: [f32; 3],
        intensity: f32,
        color: [f32; 3],
        shininess: f32,
        ambient: f32,
    ) {
        self.point_light.position = position;
        self.point_light.intensity = intensity;
        self.point_light.color = color;
        self.point_light.shininess = shininess;
        self.point_light.ambient = ambient;
        renderer
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.point_light]));
    }
}
