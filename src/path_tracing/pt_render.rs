use std::mem;

use wgpu::util::DeviceExt;

use crate::texture::Texture;

use super::{cube::Cube, render_image::RenderImage, scene::Scene, tracing_camera::{TracingCamera, TracingCameraController}};

pub struct PTRender {
    pub camera: TracingCamera,
    pub camera_controller: TracingCameraController,
    pub scene: Scene,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub render_texture: Texture,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_vertices: u32,

    pub cube_bind_group: wgpu::BindGroup,
    pub cube_buffer: wgpu::Buffer,
    pub compute_pipeline: wgpu::ComputePipeline,
    pub compute_param_buffer: wgpu::Buffer,
    pub compute_camera_buffer: wgpu::Buffer,
    pub compute_texture_output_buffer: wgpu::Buffer,
}

const MAX_CUBES: u32 = 200000;

impl PTRender {
    pub fn new(
        device : &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        queue: &wgpu::Queue,
    ) -> Self {

        let camera = TracingCamera::new(
            [0.0, 0.0, 0.0],
            3.0,
            [1920, 1080],
            [10.0, 10.0, 10.0]
        );
        let camera_controller = TracingCameraController::new();

        let scene = Scene::new();
        //TODO!: Make it so I do not need to render the scene on cpu for the original Render_image...
        let render_texture = Texture::create_buffer_from_pixel_vec(device, queue, &camera.render_scene_cpu(&Scene::empty_scene()), "PTRender Texture");

        //Texture render stuffs

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { 
                        sample_type: wgpu::TextureSampleType::Float {filterable: false},
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
            label: Some("PT Camera Layout")
        });

        

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor{
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&render_texture.view)
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(render_texture.sampler.as_ref().unwrap()),
                }
            ],
            label: Some("PTRender texture bind group"),
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("../PT_texture_shader.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("PTRender pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("PTRender render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    RenderVertex::desc(),
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState { 
                module: &shader, entry_point: "fs_main", 
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState { count: 1, mask: !0, alpha_to_coverage_enabled: false },
            multiview: None,
        });



        // Define the vertex data for a fullscreen quad
        let vertex_data = [
            RenderVertex {
                position: [-1.0, -1.0],
                tex_coords: [0.0, 0.0],
            },
            RenderVertex {
                position: [1.0, -1.0],
                tex_coords: [1.0, 0.0],
            },
            RenderVertex {
                position: [-1.0, 1.0],
                tex_coords: [0.0, 1.0],
            },
            RenderVertex {
                position: [1.0, 1.0],
                tex_coords: [1.0, 1.0],
            },
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });
    
        let index_data: &[u16] = &[0, 1, 2, 1, 3, 2];
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(index_data),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_vertices = index_data.len() as u32;


        //Compute Shader setup
        let compute_shader = device.create_shader_module(wgpu::include_wgsl!("path_tracer.wgsl"));

        let compute_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry { //Amount of cubes
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(mem::size_of::<f32>() as _),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry { //Camera values
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(16 * (mem::size_of::<f32>() as u64)),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry { //Cube in
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry { //Texture out
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("PT Compute bind group layout")
        });

        let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("PT Compute pipeline layout"),
            bind_group_layouts: &[&compute_bind_group_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("PT Compute pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "main",
            compilation_options: Default::default(),
        });

        let mut initial_cube_data = vec![
                Cube{
                    min: [0.0, 0.0, 0.0, 0.0],
                    max: [0.0, 0.0, 0.0, 0.0],
                    color: [0.0, 0.0, 0.0, 0.0],
                };
                MAX_CUBES as usize
            ];
            
        
        for i in 0..scene.cubes.len() {
            initial_cube_data[i] = scene.cubes[i];
        }

        
        let cube_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube buffer"),
            contents: bytemuck::cast_slice(&initial_cube_data),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let amount_of_cubes = scene.cubes.len() as f32;

        let compute_param_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Compute Amount of Cubes buffer"),
            contents: bytemuck::cast_slice(&[amount_of_cubes]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_vectors = [
            camera.origin[0],
            camera.origin[1],
            camera.origin[2],
            0.0, //Padding
            camera.forward_vec[0],
            camera.forward_vec[1],
            camera.forward_vec[2],
            0.0, //Padding
            camera.left_vec[0],
            camera.left_vec[1],
            camera.left_vec[2],
            0.0, //Padding
            camera.up_vec[0],
            camera.up_vec[1],
            camera.up_vec[2],
            0.0 //Padding
        ];

        let compute_camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Compute Camera Buffer"),
            contents: bytemuck::cast_slice(&camera_vectors),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let initial_output_values = vec![0.0; 1920 * 1080 * 4];

        let compute_texture_output_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Compute Texture output Buffer"),
            contents: bytemuck::cast_slice(&initial_output_values),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });


        let cube_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: compute_param_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: compute_camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: cube_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: compute_texture_output_buffer.as_entire_binding(),
                },

            ],
            label: Some("Compute Bind group"),
        });

        Self {
            camera,
            camera_controller,
            scene,
            bind_group_layout,
            bind_group,
            render_texture,
            pipeline_layout,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_vertices,
            cube_bind_group,
            cube_buffer,
            compute_pipeline,
            compute_param_buffer,
            compute_camera_buffer,
            compute_texture_output_buffer
        }

    }

    pub fn update_camera_uniform(
        &self,
        queue: &wgpu::Queue,
    ) {
        let camera_vectors = [
            self.camera.origin[0],
            self.camera.origin[1],
            self.camera.origin[2],
            0.0, //Padding
            self.camera.forward_vec[0],
            self.camera.forward_vec[1],
            self.camera.forward_vec[2],
            0.0, //Padding
            self.camera.left_vec[0],
            self.camera.left_vec[1],
            self.camera.left_vec[2],
            0.0, //Padding
            self.camera.up_vec[0],
            self.camera.up_vec[1],
            self.camera.up_vec[2],
            0.0 //Padding
        ];

        queue.write_buffer(&self.compute_camera_buffer, 0, bytemuck::cast_slice(&[camera_vectors]));
    }

    pub fn render_scene_gpu(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue
    ) {
        
        let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Compute Encoder")}); 

        {
            let mut cpass = command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(0, &self.cube_bind_group, &[]);
            cpass.dispatch_workgroups(1920 / 64, 1080, 1);
        }

        let texture_copy_view = wgpu::ImageCopyTexture {
            texture: &self.render_texture.texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        };

        let buffer_copy_view = wgpu::ImageCopyBuffer {
            buffer: &self.compute_texture_output_buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some((1920 * 4 * 4) as u32), //TODO!: I need to pad I guess for some reason.
                rows_per_image: Some(1080 as u32),
            }
        };

        let size = wgpu::Extent3d {
            width: 1920,
            height: 1080,
            depth_or_array_layers: 1,
        };

        command_encoder.copy_buffer_to_texture(buffer_copy_view, texture_copy_view, size);

        queue.submit(Some(command_encoder.finish()));
    }
}


#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct RenderVertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

impl RenderVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0=>Float32x2, 1=>Float32x2];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<RenderVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
