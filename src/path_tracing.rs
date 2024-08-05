//! Path tracing
//! 1. Construct a camera from Point
//! 2. Shoot Ray from camera to screen thats 1920x1080, or whatever resolution we have.
//! 3. Test for collisions.
//! 4. Save the color of the ray for the pixel it shoots through.
//! 5. Render the pixels on the screen.
//! 6. Profit.

use wgpu::{util::DeviceExt, Sampler};

use crate::texture::Texture;

struct Ray {
    pub origin: [f32; 3],
    pub velocity: [f32; 3],
    pub distance: f32,
    pub color: [f32; 4],
}

struct Face {
    pub vertices: [[f32; 3]; 4],
    pub normal: [f32; 3],
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

struct Cube {
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub color: [f32; 4],    
}

pub struct Scene {
    pub cubes: Vec<Cube>,
    pub background_rgba: [f32; 4],
}

pub struct RenderImage {
    pub x_size: usize,
    pub y_size: usize,
    pub pixels: Vec<[f32; 4]>,
}

pub struct TracingCamera {
    pub origin: [f32; 3],
    pub looking_at: [f32; 3],
    pub focal_distance: f32,
    pub screen_size: [usize; 2],
    pub render_ratio: [u8; 2],

}

pub struct PTRender {
    pub camera: TracingCamera,
    pub scene: Scene,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub render_texture: Texture,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_vertices: u32,
}

impl PTRender {
    pub fn new(
        device : &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        queue: &wgpu::Queue,
    ) -> Self {

        let camera = TracingCamera::new();
        let scene = Scene::new();

        let render_texture = Texture::create_buffer_from_pixel_vec(device, queue, &camera.render_scene(&scene), "PTRender Texture");

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

        let shader = device.create_shader_module(wgpu::include_wgsl!("PT_texture_shader.wgsl"));

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

        Self {
            camera,
            scene,
            bind_group_layout,
            bind_group,
            render_texture,
            pipeline_layout,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_vertices,
        }

    }
}

impl TracingCamera {
    pub fn new() -> Self {       
        Self {
            origin: [0.0, 0.0, 0.0],
            focal_distance: 0.5,
            screen_size: [1920, 1080],
            looking_at: [10.0, 10.0, 5.0],
            render_ratio: [16, 9],
        }
    }

    pub fn render_scene(&self, scene: &Scene) -> RenderImage {

        let mut render_image = RenderImage::new(self.screen_size[0], self.screen_size[1]);

        let plane_center_z = self.focal_distance;
        let plane_center_x = self.looking_at[0] * (self.focal_distance / self.looking_at[2]);
        let plane_center_y = self.looking_at[1] * (self.focal_distance / self.looking_at[2]);

        let center = [plane_center_x, plane_center_y, plane_center_z];
        let top_left = [center[0] - (self.render_ratio[0] as f32 / 2.0), center[1] - (self.render_ratio[1] as f32 / 2.0), center[2]];
        // let bottom_right = [center[0] + (self.render_ratio[0] as f32 / 2.0), center[1] + (self.render_ratio[1] as f32 / 2.0), center[2]];
        
        for y in 0..self.screen_size[1]{
            for x in 0..self.screen_size[0] {
                let screen_place = [top_left[0] + (x as f32 / self.screen_size[0] as f32) * self.render_ratio[0] as f32,
                                              top_left[1] + (y as f32 / self.screen_size[1] as f32) * self.render_ratio[1] as f32,
                                              top_left[2]];
                
                let velocity = [screen_place[0] - self.origin[0],
                                          screen_place[1] - self.origin[1],
                                          screen_place[2] - self.origin[2],
                                          ];
                
                let ray = Ray {
                    origin: self.origin,
                    velocity: velocity,
                    distance: f32::MAX,
                    color: [0.0, 0.0, 0.0, 0.0]
                };

                let color = scene.get_color(ray);
                render_image.pixels[y * render_image.x_size + x] = color; 
            }
        }

        render_image

    }
}

impl Scene {

    pub fn new() -> Self {
        Self {
            cubes: vec![],
            background_rgba: [0.4, 0.5, 0.6, 1.0],
        }
    }

    //Maybe rename to albedo in future, if we have ligthing etc.
    fn get_color(&self, mut ray: Ray) -> [f32; 4]{

        let mut rgba = self.background_rgba;

        for cube in &self.cubes {
            cube.intersect_ray(&mut ray);
        }

        if ray.distance < f32::MAX {
            rgba = ray.color;
        }

        rgba
    }
}

impl Cube {
    pub fn intersect_ray(&self, ray: &mut Ray){
        //https://tavianator.com/cgit/dimension.git/tree/libdimension/bvh/bvh.c#n196
        //https://education.siggraph.org/static/HyperGraph/raytrace/rtinter3.htm
        //Not the best way yet, but more intuitive.


        let mut tnear = f32::MIN;
        let mut tfar = f32::MAX;

        for d in 0..3 as usize {

            if (ray.velocity[d] == 0.0) {
                if (ray.origin[d] >= self.min[d] && ray.origin[d] <= self.max[d]){
                    continue
                } else {
                    return;
                }
            }

            let t1 = (self.min[d] - ray.origin[d]) / ray.velocity[d];
            let t2 = (self.max[d] - ray.origin[d]) / ray.velocity[d];
            let tmin = t1.min(t2);
            let tmax = t1.max(t2);

            tnear = tnear.max(tmin);
            tfar = tfar.min(tmax);
        }

        if tnear > tfar || tfar < 0.0 {
            return;
        } else {
            ray.distance = tnear;
            ray.color = self.color;
        }

    }
}

impl RenderImage {
    pub fn new(x_dimension: usize, y_dimension: usize) -> Self {
        let pixels: Vec<[f32; 4]> = vec![[0.0, 0.0, 0.0, 0.0]; x_dimension * y_dimension];

        Self {
            x_size: x_dimension,
            y_size: y_dimension,
            pixels: pixels,
        }
    }
} 