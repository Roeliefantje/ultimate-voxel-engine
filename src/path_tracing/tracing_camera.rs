use super::{quaternion::Quaternion, ray::Ray, render_image::RenderImage, scene::Scene, vector_funcs::{cross_vector, normalize_vector}};


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


    pub fn render_scene(&self, scene: &Scene) -> RenderImage {

        let mut render_image = RenderImage::new(self.screen_size[0], self.screen_size[1]);
        let plane_center = [self.forward_vec[0] * self.focal_distance, self.forward_vec[1] * self.focal_distance, self.forward_vec[2] * self.focal_distance];

        let top_left = [
            plane_center[0] + self.left_vec[0] * self.aspect_ratio + self.up_vec[0], 
            plane_center[1] + self.left_vec[1] * self.aspect_ratio + self.up_vec[1], 
            plane_center[2] + self.left_vec[2] * self.aspect_ratio + self.up_vec[2], 
        ];

        //TODO!: Calculate bottom right and iterate between them based on u/v to fix aspect ratio possibly.
        
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
