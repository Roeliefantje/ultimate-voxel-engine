use rand::Rng;
use wgpu::util::DeviceExt;

use crate::{camera::Camera, texture};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3]
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            color: [0.0, 0.0, 0.0]
        }
    }
}

impl Vertex {

    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0=> Float32x3, 1 => Float32x3];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

const INDICES: &[u16] = &[
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

pub struct Object {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_vertices: u32,
}

impl Object {
    pub fn new_cube(device: &wgpu::Device) -> Object {
        
        let mut rng = rand::thread_rng();
        //cube kek
        let random_x: f32 = rng.gen_range(-15..15) as f32;
        let random_y: f32 = rng.gen_range(-15..15) as f32;
        let random_z: f32 = rng.gen_range(-15..15) as f32;

        let random_color: [f32; 3] = [rng.gen_range(0..100) as f32 / 100f32, rng.gen_range(0..100) as f32 / 100f32, rng.gen_range(0..100) as f32 / 100f32];

        let vertices: &[Vertex] = &[
            Vertex{ position: [random_x, random_y, random_z], color: random_color}, //0
            Vertex{ position: [random_x + 1.0, random_y, random_z], color: random_color}, //1
            Vertex{ position: [random_x + 1.0, random_y + 1.0, random_z], color: random_color}, //2
            Vertex{ position: [random_x, random_y + 1.0, random_z], color: random_color}, //3
            Vertex{ position: [random_x, random_y, random_z + 1.0], color: random_color}, //4
            Vertex{ position: [random_x + 1.0, random_y, random_z + 1.0], color: random_color}, //5
            Vertex{ position: [random_x + 1.0, random_y + 1.0, random_z + 1.0], color: random_color}, //6
            Vertex{ position: [random_x, random_y + 1.0, random_z + 1.0], color: random_color}, //7

        ];

        Self {

            

            vertex_buffer: device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            ),
            index_buffer: device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(INDICES),
                    usage: wgpu::BufferUsages::INDEX,
                }
            ),
            num_vertices: INDICES.len() as u32
        }
    }
}


//An Object Group shares its shaders with other objects.
//These objects can have their own meshes and will have seperated draw calls.
pub struct ObjectGroup {
    pub render_pipeline: wgpu::RenderPipeline,
    pub objects: Vec<Object>
}

impl ObjectGroup {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, camera: &Camera) -> ObjectGroup {

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Object Group Render Pipeline Layout"),
            bind_group_layouts: &[
                &camera.camera_bind_group_layout
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[
                    Vertex::desc(),
                ]
            },
            fragment: Some(wgpu::FragmentState { module: &shader, entry_point: "fs_main", 
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })]
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState { count: 1, mask: !0, alpha_to_coverage_enabled: false },
            multiview: None,
        });

        let mut objects: Vec<Object> = Vec::new();

        // for _ in 0..1000 {
        //     objects.push(Object::new_cube(device)); 
        // }
        objects.push(Object::new_chunk(device, 0, 0));


        Self {
            render_pipeline,
            objects
        }
    }
}