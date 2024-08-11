use noise::NoiseFn;
use rand::Rng;

use super::cube::Cube;

pub struct PTObject {
    pub cubes: Vec<Cube>
}

const CHUNK_SIZE: i32 = 32;

impl PTObject {
    pub fn new(chunk_x: i32, chunk_y: i32) -> Self {
        let perlin = noise::Perlin::new(1);

        let x_offset: i32 = chunk_x * CHUNK_SIZE;
        let y_offset: i32 = chunk_y * CHUNK_SIZE;

        let mut cubes: Vec<Cube> = vec![];

        let mut rng = rand::thread_rng();
    
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                // let z_val: f32 = 1.0;
                let mut z_val = perlin.get([(x_offset + x) as f64 / 10.0, (y_offset + y) as f64 / 10.0]) * 4.0;
                z_val = f64::floor(z_val as f64);
                let color: [f32; 4] = [rng.gen_range(0..100) as f32 / 100f32, rng.gen_range(0..100) as f32 / 100f32, rng.gen_range(0..100) as f32 / 100f32, 1.0];
                let cube = Cube::new_cube_at(&[(x_offset + x) as f32, (y_offset + y) as f32, z_val as f32], color);
                // println!("z_val: {:?}", z_val);
                cubes.push(cube);
            }
        }


        Self {
            cubes: cubes,
        }

    }

    pub fn get_cubes(&self) -> &Vec<Cube> {
        return &self.cubes;
    }
}