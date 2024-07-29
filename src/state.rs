use wgpu::{core::{command::ClearError, instance}, hal::gles::Adapter, rwh::AppKitDisplayHandle, util::DeviceExt};
use winit::{
    window::Window,
    event::*,
};

use crate::{camera::Camera, objects::*};




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
    object_groups: Vec<ObjectGroup>,
    camera: Camera,
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
        
        

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            clear_color,
            object_groups,
            camera
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
        // println!("Resizing the screen");
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self){

    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {

        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });

        //Clear the screen
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

        
        

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
