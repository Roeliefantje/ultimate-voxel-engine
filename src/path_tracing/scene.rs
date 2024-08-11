use rand::Rng;

use super::{chunk::PTObject, cube::Cube, ray::Ray};

pub struct Scene {
    pub cubes: Vec<Cube>,
    pub background_rgba: [f32; 4],
    pub chunk_grid: Vec<bool>, //The way I have made it now makes it kind of unnessecary for this grid to exist, as it will always be true if teh chunks are loaded in.
    pub grid_size: usize, //The amount of Chunks in a direction. (Note the render distance is this value / 2, as we support negative values as well)
}

fn chunk_xy_to_grid_location(grid_size: &usize, chunk_x: &i32, chunk_y: &i32) -> usize {

    let grid_y = (grid_size / 2) as i32 + chunk_y;
    let grid_x = (grid_size / 2) as i32 + chunk_x;
    grid_y as usize * grid_size + grid_x as usize
}

impl Scene {

    pub fn new() -> Self {

        let mut cubes: Vec<Cube> = vec![];

        let grid_size = 16;

        let mut chunk_grid: Vec<bool> = vec![false; grid_size * grid_size];

        // for _ in 0..10 {
        //     let mut rng = rand::thread_rng();
        //     let random_location: [f32; 3] = [rng.gen_range(0..25) as f32, rng.gen_range(0..25) as f32, rng.gen_range(0..25) as f32];
        //     let random_color: [f32; 4] = [rng.gen_range(0..100) as f32 / 100f32, rng.gen_range(0..100) as f32 / 100f32, rng.gen_range(0..100) as f32 / 100f32, 1.0];
        //     cubes.push(Cube::new_cube_at(&random_location, random_color));
        // }
        for x in -1..1 {
            for y in -1..1 {
                cubes.extend_from_slice(PTObject::new(x, y).get_cubes());
                let index = chunk_xy_to_grid_location(&grid_size, &x, &y);
                chunk_grid[index] = true;
            }
        }
        
        Self {
            cubes: cubes,
            background_rgba: [0.4, 0.5, 0.6, 1.0],
            chunk_grid: chunk_grid,
            grid_size: grid_size,
        }
    }

    pub fn empty_scene() -> Self {
        let grid_size = 16;

        let chunk_grid: Vec<bool> = vec![false; grid_size * grid_size];

        Self {
            cubes: vec![],
            background_rgba: [0.4, 0.5, 0.6, 1.0],
            chunk_grid: chunk_grid,
            grid_size: grid_size,
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

