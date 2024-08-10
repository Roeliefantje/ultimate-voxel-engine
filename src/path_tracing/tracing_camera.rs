use winit::{event::{ElementState, KeyEvent, WindowEvent}, keyboard::{KeyCode, PhysicalKey}};

use super::{pt_render::{self, PTRender}, quaternion::Quaternion, ray::Ray, render_image::RenderImage, scene::Scene, vector_funcs::{cross_vector, normalize_vector}};


pub struct TracingCamera {
    pub origin: [f32; 3],
    pub forward_vec: [f32; 3],
    pub left_vec: [f32; 3],
    pub up_vec: [f32; 3],
    pub aspect_ratio: f32,
    pub focal_distance: f32,
    pub screen_size: [usize; 2],
}

impl TracingCamera {
    pub fn new(
        origin: [f32; 3],
        focal_distance: f32,
        screen_size: [usize; 2],
        looking_at: [f32; 3]
    ) -> Self {
        let looking_at_n = normalize_vector(&[looking_at[0] - origin[0], looking_at[1] - origin[1], looking_at[2] - origin[2]]);
        // let plane_center = [looking_at_n[0] * focal_distance, looking_at_n[1] * focal_distance, looking_at_n[2] * focal_distance];
        let mut up_vector = [0.0, 0.0, 1.0];
        let left_vec = cross_vector(&looking_at_n, &up_vector);
        up_vector = cross_vector(&looking_at_n, &left_vec);

        let aspect_ratio = screen_size[0] as f32 / screen_size[1] as f32;

        Self {
            origin,
            forward_vec: looking_at_n,
            left_vec: left_vec,
            up_vec: up_vector,
            aspect_ratio: aspect_ratio,
            focal_distance: focal_distance,
            screen_size: screen_size,
        }

    }

    pub fn rotate_camera_yaw(&mut self, rad: f32) {
        let q = Quaternion::from_axis_angle(self.up_vec, rad);
        self.left_vec = q.rotate_vector(self.left_vec);
        self.forward_vec = q.rotate_vector(self.forward_vec);
    }

    pub fn rotate_camera_pitch(&mut self, rad: f32) {
        let q = Quaternion::from_axis_angle(self.left_vec, rad);
        self.up_vec = q.rotate_vector(self.up_vec);
        self.forward_vec = q.rotate_vector(self.forward_vec);
    }

    pub fn rotate_camera_roll(&mut self, rad: f32) {
        let q = Quaternion::from_axis_angle(self.forward_vec, rad);
        self.up_vec = q.rotate_vector(self.up_vec);
        self.left_vec = q.rotate_vector(self.left_vec);
    }


    pub fn render_scene_cpu(&self, scene: &Scene) -> RenderImage {

        let mut render_image = RenderImage::new(self.screen_size[0], self.screen_size[1]);
        let plane_center = [self.forward_vec[0] * self.focal_distance, self.forward_vec[1] * self.focal_distance, self.forward_vec[2] * self.focal_distance];

        let top_left = [
            plane_center[0] + self.left_vec[0] * self.aspect_ratio + self.up_vec[0], 
            plane_center[1] + self.left_vec[1] * self.aspect_ratio + self.up_vec[1], 
            plane_center[2] + self.left_vec[2] * self.aspect_ratio + self.up_vec[2], 
        ];

        //TODO!: Calculate bottom right and iterate between them based on u/v to fix aspect ratio possibly.
        //TODO: Move this to the gpu
        
        for y in 0..self.screen_size[1]{
            let v = y as f32 / self.screen_size[1] as f32;
            for x in 0..self.screen_size[0] {
                let u = x as f32 / self.screen_size[0] as f32;

                let screen_place = [
                    top_left[0] - self.left_vec[0] * u * 2.0 * self.aspect_ratio - self.up_vec[0] * v * 2.0,
                    top_left[1] - self.left_vec[1] * u * 2.0 * self.aspect_ratio - self.up_vec[1] * v * 2.0,
                    top_left[2] - self.left_vec[2] * u * 2.0 * self.aspect_ratio - self.up_vec[2] * v * 2.0,
                ];
                
                let velocity = [
                    screen_place[0] - self.origin[0],
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


pub struct TracingCameraController {
    pub sensitivity: f32,
    pub speed: f32,
    pub is_forward_pressed: bool,
    pub is_backward_pressed: bool,
    pub is_left_pressed: bool,
    pub is_right_pressed: bool,
    pub mouse_x_movement: f32,
    pub mouse_y_movement: f32,
}

impl TracingCameraController {
    pub fn new() -> Self {
        Self {
            sensitivity: 0.01,
            speed: 0.03,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            mouse_x_movement: 0.0,
            mouse_y_movement: 0.0,
        }
    }

    pub fn update_camera(
        &mut self,
        queue: &wgpu::Queue,
        pt_render: &mut PTRender,
    ) {
        let mut changed = false;
        if self.is_forward_pressed && !self.is_backward_pressed {
            pt_render.camera.origin = [
                pt_render.camera.origin[0] + pt_render.camera.forward_vec[0] * self.speed,
                pt_render.camera.origin[1] + pt_render.camera.forward_vec[1] * self.speed,
                pt_render.camera.origin[2] + pt_render.camera.forward_vec[2] * self.speed,
            ];
            changed = true;
        } else if self.is_backward_pressed && !self.is_forward_pressed {
            pt_render.camera.origin = [
                pt_render.camera.origin[0] - pt_render.camera.forward_vec[0] * self.speed,
                pt_render.camera.origin[1] - pt_render.camera.forward_vec[1] * self.speed,
                pt_render.camera.origin[2] - pt_render.camera.forward_vec[2] * self.speed,
            ];
            changed = true;
        }

        if self.is_left_pressed && !self.is_right_pressed {
            pt_render.camera.origin = [
                pt_render.camera.origin[0] + pt_render.camera.left_vec[0] * self.speed,
                pt_render.camera.origin[1] + pt_render.camera.left_vec[1] * self.speed,
                pt_render.camera.origin[2] + pt_render.camera.left_vec[2] * self.speed,
            ];
            changed = true;
        } else if self.is_right_pressed && !self.is_left_pressed {
            pt_render.camera.origin = [
                pt_render.camera.origin[0] - pt_render.camera.left_vec[0] * self.speed,
                pt_render.camera.origin[1] - pt_render.camera.left_vec[1] * self.speed,
                pt_render.camera.origin[2] - pt_render.camera.left_vec[2] * self.speed,
            ];
            changed = true;
        }

        if self.mouse_x_movement != 0.0 {
            pt_render.camera.rotate_camera_yaw(self.mouse_x_movement * self.sensitivity * -1.0);
            self.mouse_x_movement = 0.0;
            changed = true;
        }

        if self.mouse_y_movement != 0.0 {
            pt_render.camera.rotate_camera_pitch(self.mouse_y_movement * self.sensitivity * -1.0);
            self.mouse_y_movement = 0.0;
            changed = true;
        }


        if changed {
            pt_render.update_camera_uniform(queue)
        }
    }


    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {KeyCode::KeyW | KeyCode::ArrowUp => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyA | KeyCode::ArrowLeft => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyS | KeyCode::ArrowDown => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyD | KeyCode::ArrowRight => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
}