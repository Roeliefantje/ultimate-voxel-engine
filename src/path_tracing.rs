//! Path tracing
//! 1. Construct a camera from Point
//! 2. Shoot Ray from camera to screen thats 1920x1080, or whatever resolution we have.
//! 3. Test for collisions.
//! 4. Save the color of the ray for the pixel it shoots through.
//! 5. Render the pixels on the screen.
//! 6. Profit.

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

struct Cube {
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub color: [f32; 4],    
}

struct Scene {
    pub cubes: Vec<Cube>,
    pub background_rgba: [f32; 4],
}

struct RenderImage {
    dimensions: [usize; 2],
    pixels: Vec<Vec<[f32; 4]>>,
}

struct TracingCamera {
    pub origin: [f32; 3],
    pub looking_at: [f32; 3],
    pub focal_distance: f32,
    pub screen_size: [usize; 2],
    pub render_ratio: [u8; 2],
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
            let mut x_vec: Vec<[f32; 4]> = Vec::new();

            for x in 0..self.screen_size[0] {
                let screen_place = [top_left[0] + (x as f32 / self.screen_size[0] as f32) * self.render_ratio[0] as f32,
                                              top_left[1] + (y as f32 / self.screen_size[1] as f32) * self.render_ratio[1] as f32,
                                              top_left[2]];
                
                let velocity = [screen_place[0] - self.origin[0],
                                          screen_place[1] - self.origin[1],
                                          screen_place[2] - self.origin[2],
                                          ];
                
                let mut ray = Ray {
                    origin: self.origin,
                    velocity: velocity,
                    distance: f32::MAX,
                    color: [0.0, 0.0, 0.0, 0.0]
                };

                let color = scene.get_color(ray);
                x_vec.push(color);
            }
            render_image.pixels.push(x_vec);
        }

        render_image

    }
}

impl Scene {
    //Maybe rename to albedo in future, if we have ligthing etc.
    pub fn get_color(&self, mut ray: Ray) -> [f32; 4]{

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
        let pixels: Vec<Vec<[f32; 4]>> = Vec::new();

        Self {
            dimensions: [x_dimension, y_dimension],
            pixels: pixels,
        }
    }
}