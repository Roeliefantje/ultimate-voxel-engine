use winit::{
    window::Window,
    event::*,
};

use crate::{camera::{Camera, CameraController}, objects::*, path_tracing::pt_render::PTRender, texture::*};




// const VERTICES: &[Vertex] = &[
//     Vertex { position: [-0.0868241, 0.49240386, 0.0], color: [0.2, 0.0, 0.2] }, // A
//     Vertex { position: [-0.49513406, 0.06958647, 0.0], color: [0.4, 0.0, 0.4] }, // B
//     Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [0.6, 0.0, 0.6] }, // C
//     Vertex { position: [0.35966998, -0.3473291, 0.0], color: [0.8, 0.0, 0.8] }, // D
//     Vertex { position: [0.44147372, 0.2347359, 0.0], color: [1.0, 0.0, 1.0] }, // E
// ];








//An InstanceGroup shares its shaders and mesh with multiple objects.
// struct InstanceGroup {
//     render_pipeline: wgpu::RenderPipeLine,
//     //    
// }





pub struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: &'a Window,
    clear_color: wgpu::Color,
    #[cfg(feature = "rasterization")] object_groups: Vec<ObjectGroup>,
    camera: Camera,
    camera_controller: CameraController,
    depth_texture: Texture,
    pt_render: PTRender,
    //instance_groups: Vec<InstanceGroup>,
}

impl <'a> State<'a > {
    pub async fn new(window: &'a Window) -> State<'a> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor { 
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface)
            }
        ).await.unwrap();

        
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let clear_color = wgpu::Color {
            r: 0.1, 
            g: 0.1,
            b: 0.1,
            a: 1.0,
        };

        let camera = Camera::new(&device, &config);

        let mut object_groups: Vec<ObjectGroup> = Vec::new();
        
        object_groups.push(ObjectGroup::new(&device, &config, &camera));
        
        let camera_controller = CameraController::new(0.2f32);

        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");

        let pt_render = PTRender::new(&device, &config, &queue);

        println!("Finished creating state");

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            clear_color,
            #[cfg(feature = "rasterization")] object_groups,
            camera,
            camera_controller,
            depth_texture,
            pt_render,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        assert!(new_size.width > 0);
        assert!(new_size.height > 0);
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
        self.depth_texture = Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        // println!("Resizing the screen");
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    pub fn update(&mut self){
        //TODO: I dont think this is very clean, probably want to write to the buffer inside of the camera controller.
        #[cfg(feature = "rasterization")]
        {
            self.camera_controller.update_camera(&mut self.camera.camera_inner);
            self.camera.camera_uniform.update_view_proj(&self.camera.camera_inner);
            self.queue.write_buffer(&self.camera.camera_buffer, 0, bytemuck::cast_slice(&[self.camera.camera_uniform]));
        }
        #[cfg(not(feature = "rasterization"))]
        {
            // self.pt_render.camera.rotate_camera_pitch(0.01);
            // self.pt_render.camera.rotate_camera_yaw(0.01);
            self.pt_render.camera.rotate_camera_roll(0.01);
            self.pt_render.render_texture.update_texture(&self.queue, &self.pt_render.camera.render_scene_cpu(&self.pt_render.scene));
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // use std::time::Instant;
        // let now = Instant::now();

        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });

        //Clear the screen
        #[cfg(feature = "rasterization")]
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations { 
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    }
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations{
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            //TODO: Render all the objects here...
            for object_group in &self.object_groups {
                render_pass.set_pipeline(&object_group.render_pipeline);
                render_pass.set_bind_group(0, &self.camera.camera_bind_group, &[]);
                
                for object in &object_group.objects {
                    render_pass.set_vertex_buffer(0, object.vertex_buffer.slice(..));
                    render_pass.set_index_buffer(object.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..object.num_vertices, 0, 0..1);
                }
            }
        }

        #[cfg(not(feature = "rasterization"))]
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations { 
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    }
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.pt_render.render_pipeline);
            render_pass.set_bind_group(0, &self.pt_render.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.pt_render.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.pt_render.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.pt_render.num_vertices, 0, 0..1);
        }

        
        

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        // let elapsed = now.elapsed();
        // println!("Elapsed: {:.2?}", elapsed);

        Ok(())
    }
}
