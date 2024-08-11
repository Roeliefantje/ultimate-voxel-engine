use rand::Rng;

use super::{chunk::PTObject, cube::Cube, ray::Ray};

pub struct Scene {
    pub cubes: Vec<Cube>,
    pub background_rgba: [f32; 4],
}

impl Scene {

    pub fn new() -> Self {

        let mut cubes: Vec<Cube> = vec![];

        // for _ in 0..10 {
        //     let mut rng = rand::thread_rng();
        //     let random_location: [f32; 3] = [rng.gen_range(0..25) as f32, rng.gen_range(0..25) as f32, rng.gen_range(0..25) as f32];
        //     let random_color: [f32; 4] = [rng.gen_range(0..100) as f32 / 100f32, rng.gen_range(0..100) as f32 / 100f32, rng.gen_range(0..100) as f32 / 100f32, 1.0];
        //     cubes.push(Cube::new_cube_at(&random_location, random_color));
        // }
        for x in -1..1 {
            for y in -1..1 {
                cubes.extend_from_slice(PTObject::new(x, y).get_cubes());
            }
        }
        
        

        Self {
            cubes: cubes,
            background_rgba: [0.4, 0.5, 0.6, 1.0],
        }
    }

    pub fn empty_scene() -> Self {
        Self {
            cubes: vec![],
            background_rgba: [0.4, 0.5, 0.6, 1.0],
        }
    }

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

