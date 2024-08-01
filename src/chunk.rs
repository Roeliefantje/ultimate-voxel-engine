use noise::NoiseFn;
use rand::Rng;
use wgpu::util::DeviceExt;

use crate::objects::{Object, Vertex};

const CHUNK_SIZE: i32 = 32;

const INDICES: [u16; 36] = [
        //Top (+z)
        7, 4, 5,
        7, 5, 6,

        //bottom (-z)
        3, 1, 0, 
        3, 2, 1,

        //Front (-y)
        4, 0, 1,
        4, 1, 5,

        //Back (+y)
        6, 2, 3,
        6, 3, 7,

        //Left (-x)
        7, 3, 0,
        7, 0, 4,

        //right (+x)
        5, 2, 6,
        5, 1, 2,
];


fn create_cube_mesh(x: f32,  y: f32, z:f32, index_offset:u16) -> ([Vertex; 8], [u16; 36]) {

    // let color: [f32; 3] = [0.5, 0.5, 0.5];
    let mut rng = rand::thread_rng();
    
    let color: [f32; 3] = [rng.gen_range(0..100) as f32 / 100f32, rng.gen_range(0..100) as f32 / 100f32, rng.gen_range(0..100) as f32 / 100f32];

    let vertices: [Vertex; 8] = [
            Vertex{ position: [x, y, z], color: color}, //0
            Vertex{ position: [x + 1.0, y, z], color: color}, //1
            Vertex{ position: [x + 1.0, y + 1.0, z], color: color}, //2
            Vertex{ position: [x, y + 1.0, z], color: color}, //3
            Vertex{ position: [x, y, z + 1.0], color: color}, //4
            Vertex{ position: [x + 1.0, y, z + 1.0], color: color}, //5
            Vertex{ position: [x + 1.0, y + 1.0, z + 1.0], color: color}, //6
            Vertex{ position: [x, y + 1.0, z + 1.0], color: color}, //7
        ];

    let mut indices = INDICES.clone();

    for index in &mut indices {
        *index += index_offset;
    }

    (vertices, indices)
}

impl Object {
    pub fn new_chunk(device: &wgpu::Device, chunk_x: i32, chunk_y: i32) -> Object {

        let perlin = noise::Perlin::new(1);

        let x_offset: i32 = chunk_x * CHUNK_SIZE;
        let y_offset: i32 = chunk_y * CHUNK_SIZE;
        let mut current_offset_indices = 0;
        //Todo probably change these to vectors when I want to create more proper terrain + cull faces that arent shown. 
        let mut vertices: [Vertex; (8 * CHUNK_SIZE * CHUNK_SIZE) as usize] = [Vertex::default(); (8 * CHUNK_SIZE * CHUNK_SIZE) as usize];
        let mut indices: [u16; (36 * CHUNK_SIZE * CHUNK_SIZE) as usize] = [0; (36 * CHUNK_SIZE * CHUNK_SIZE) as usize];

        
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                // let z_val: f32 = 1.0;
                let z_val = perlin.get([(x_offset + x) as f64 / 10.0, (y_offset + y) as f64 / 10.0]) * 4.0;
                println!("z_val: {:?}", z_val);
                let (cube_vertex, cube_index) = create_cube_mesh((x_offset + x) as f32, (y_offset + y) as f32, z_val as f32, current_offset_indices);
                
                let v_index = ((CHUNK_SIZE * y + x) * 8) as usize;
                vertices[v_index..v_index+8].copy_from_slice(&cube_vertex);
                let i_index = ((CHUNK_SIZE * y + x) * 36) as usize;
                indices[i_index..i_index+36].copy_from_slice(&cube_index);
                current_offset_indices += 8;
            }
        }



        Self {
            vertex_buffer: device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            ),
            index_buffer: device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                }
            ),
            num_vertices: indices.len() as u32
        }
    }
}