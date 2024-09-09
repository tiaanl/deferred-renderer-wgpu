use obj::TexturedVertex;
use wgpu::util::DeviceExt;

use crate::Renderer;

#[derive(Clone, Copy, bytemuck::NoUninit)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coord: [f32; 2],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
}

impl Vertex {
    #[allow(clippy::too_many_arguments)]
    pub fn raw(x: f32, y: f32, z: f32, n_x: f32, n_y: f32, n_z: f32, u: f32, v: f32) -> Self {
        Self {
            position: [x, y, z],
            normal: [n_x, n_y, n_z],
            tex_coord: [u, v],
            tangent: [0.0, 0.0, 0.0],
            bitangent: [0.0, 0.0, 0.0],
        }
    }
}

#[derive(Default)]
pub struct Mesh<V> {
    pub vertices: Vec<V>,
    pub indices: Vec<u16>,
}

impl Mesh<Vertex> {
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

    pub fn update_tangents(&mut self) {
        let mut triangles_included = vec![0; self.vertices.len()];

        for c in self.indices.chunks(3) {
            let v0 = self.vertices[c[0] as usize];
            let v1 = self.vertices[c[1] as usize];
            let v2 = self.vertices[c[2] as usize];

            let pos0: cgmath::Vector3<_> = v0.position.into();
            let pos1: cgmath::Vector3<_> = v1.position.into();
            let pos2: cgmath::Vector3<_> = v2.position.into();

            let uv0: cgmath::Vector2<_> = v0.tex_coord.into();
            let uv1: cgmath::Vector2<_> = v1.tex_coord.into();
            let uv2: cgmath::Vector2<_> = v2.tex_coord.into();

            // Calculate the edges of the triangle
            let delta_pos1 = pos1 - pos0;
            let delta_pos2 = pos2 - pos0;

            // This will give us a direction to calculate the
            // tangent and bitangent
            let delta_uv1 = uv1 - uv0;
            let delta_uv2 = uv2 - uv0;

            // Solving the following system of equations will
            // give us the tangent and bitangent.
            //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
            //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
            // Luckily, the place I found this equation provided
            // the solution!
            let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
            let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
            // We flip the bitangent to enable right-handed normal
            // maps with wgpu texture coordinate system
            let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

            // We'll use the same tangent/bitangent for each vertex in the triangle
            self.vertices[c[0] as usize].tangent =
                (tangent + cgmath::Vector3::from(self.vertices[c[0] as usize].tangent)).into();
            self.vertices[c[1] as usize].tangent =
                (tangent + cgmath::Vector3::from(self.vertices[c[1] as usize].tangent)).into();
            self.vertices[c[2] as usize].tangent =
                (tangent + cgmath::Vector3::from(self.vertices[c[2] as usize].tangent)).into();
            self.vertices[c[0] as usize].bitangent =
                (bitangent + cgmath::Vector3::from(self.vertices[c[0] as usize].bitangent)).into();
            self.vertices[c[1] as usize].bitangent =
                (bitangent + cgmath::Vector3::from(self.vertices[c[1] as usize].bitangent)).into();
            self.vertices[c[2] as usize].bitangent =
                (bitangent + cgmath::Vector3::from(self.vertices[c[2] as usize].bitangent)).into();

            // Used to average the tangents/bitangents
            triangles_included[c[0] as usize] += 1;
            triangles_included[c[1] as usize] += 1;
            triangles_included[c[2] as usize] += 1;
        }

        // Average the tangents/bitangents
        for (i, n) in triangles_included.into_iter().enumerate() {
            let denom = 1.0 / n as f32;
            let v = &mut self.vertices[i];
            v.tangent = (cgmath::Vector3::from(v.tangent) * denom).into();
            v.bitangent = (cgmath::Vector3::from(v.bitangent) * denom).into();
        }
    }
}

impl<V: bytemuck::NoUninit> Mesh<V> {
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
