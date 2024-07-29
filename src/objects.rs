use rand::Rng;
use wgpu::util::DeviceExt;

use crate::camera::Camera;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3]
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
    // Front face
    0, 1, 2,
    2, 3, 0,
    // Back face
    4, 5, 6,
    6, 7, 4,
    // Left face
    4, 0, 3,
    3, 7, 4,
    // Right face
    1, 5, 6,
    6, 2, 1,
    // Top face
    3, 2, 6,
    6, 7, 3,
    // Bottom face
    4, 5, 1,
    1, 0, 4,
];

pub struct Object {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_vertices: u32,
}

impl Object {
    pub fn new(device: &wgpu::Device) -> Object {
        
        let mut rng = rand::thread_rng();
        //cube kek
        let random_x: f32 = rng.gen_range(0..10) as f32;
        let random_y: f32 = rng.gen_range(0..10) as f32;
        let random_z: f32 = rng.gen_range(0..10) as f32;

        let random_color: [f32; 3] = [rng.gen_range(0..100) as f32 / 100f32, rng.gen_range(0..100) as f32 / 100f32, rng.gen_range(0..100) as f32 / 100f32];

        let vertices: &[Vertex] = &[
            Vertex{ position: [random_x, random_y, random_z], color: random_color}, //A
            Vertex{ position: [random_x + 1f32, random_y, random_z], color: random_color}, //B
            Vertex{ position: [random_x + 1f32, random_y + 1f32, random_z], color: random_color}, //C
            Vertex{ position: [random_x, random_y + 1f32, random_z], color: random_color}, //D
            Vertex{ position: [random_x, random_y, random_z + 1f32], color: random_color}, //E
            Vertex{ position: [random_x + 1f32, random_y, random_z + 1f32], color: random_color}, //F
            Vertex{ position: [random_x + 1f32, random_y + 1f32, random_z + 1f32], color: random_color}, //G
            Vertex{ position: [random_x, random_y + 1f32, random_z + 1f32], color: random_color}, //H
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
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState { count: 1, mask: !0, alpha_to_coverage_enabled: false },
            multiview: None,
        });

        let mut objects: Vec<Object> = Vec::new();

        for _ in 0..100 {
            objects.push(Object::new(device)); 
        }

        // objects.push(Object::new(device));
        // objects.push(Object::new(device));
        // objects.push(Object::new(device));
        // objects.push(Object::new(device));


        Self {
            render_pipeline,
            objects
        }
    }
}