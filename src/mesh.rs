use obj::TexturedVertex;
use wgpu::util::DeviceExt;

use crate::Renderer;

#[derive(Clone, Copy, bytemuck::NoUninit)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coord: [f32; 2],
}

impl Vertex {
    #[allow(clippy::too_many_arguments)]
    pub fn raw(x: f32, y: f32, z: f32, n_x: f32, n_y: f32, n_z: f32, u: f32, v: f32) -> Self {
        Self {
            position: [x, y, z],
            normal: [n_x, n_y, n_z],
            tex_coord: [u, v],
        }
    }
}

#[derive(Default)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

impl Mesh {
    pub fn from_reader(reader: impl std::io::BufRead) -> Result<Self, ()> {
        // let r = BufReader::new(std::fs::File::open(path).unwrap());
        // let obj: obj::Obj<TexturedVertex, u16> = obj::load_obj(r).unwrap();
        let obj: obj::Obj<TexturedVertex, u16> = obj::load_obj(reader).map_err(|_| ())?;

        Ok(Self {
            vertices: obj
                .vertices
                .iter()
                .map(|v| {
                    Vertex::raw(
                        v.position[0],
                        v.position[1],
                        v.position[2],
                        v.normal[0],
                        v.normal[1],
                        v.normal[2],
                        v.texture[0],
                        v.texture[1],
                    )
                })
                .collect(),
            indices: obj.indices,
        })
    }

    pub fn upload_to_gpu(&self, renderer: &Renderer) -> GpuMesh {
        let vertex_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("vertex buffer"),
                contents: bytemuck::cast_slice(self.vertices.as_ref()),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("index buffer"),
                contents: bytemuck::cast_slice(self.indices.as_ref()),
                usage: wgpu::BufferUsages::INDEX,
            });

        GpuMesh {
            vertex_buffer,
            index_buffer,
            index_count: self.indices.len() as u32,
        }
    }
}

pub struct GpuMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}
