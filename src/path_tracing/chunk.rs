use noise::NoiseFn;
use rand::Rng;

use super::cube::Cube;

pub struct PTObject {
    pub cubes: Vec<Cube>,
    pub octree: Option<SparseOctree>,
}

pub struct SparseOctreeNode {
    pub is_leaf_node: bool,
    pub children: Option<Vec<SparseOctreeNode>>,
    pub child_mask: Option<u8>,
    pub color: Option<[f32; 4]>,
}

pub struct SparseOctree {
    pub aabb: [[i32; 3]; 2],
    pub max_depth: u32,
    pub root: SparseOctreeNode,
}

const CHUNK_SIZE: i32 = 64;

fn cube_at_loc(cubes: &Vec<Cube>, loc: [i32; 3]) -> bool {
    //todo! Construct a lookup table at chunk generation instead of a vec of cubes.
    
    for cube in cubes {
        if cube.min[0] as i32 == loc[0] &&
           cube.min[1] as i32 == loc[1] &&
           cube.min[2] as i32 == loc[2] {
            return true;
        }
    }


    return false;
}

fn construct_child(cubes: &Vec<Cube>, bounds: [[i32; 3]; 2]) -> Option<SparseOctreeNode> {

    if (bounds[1][0] - bounds[0][0]) as i32 == 1 {
        if cube_at_loc(cubes, bounds[0]){
            println!("Spawning leaf node!");
            Some(SparseOctreeNode {
                is_leaf_node: true,
                children: None,
                child_mask: None,
                color: Some([0.0, 1.0, 0.0, 0.0]),
            })
        } else {
            None
        }
        
    } else {

        let distance = [
            (bounds[1][0] - bounds[0][0]),
            (bounds[1][1] - bounds[0][1]),
            (bounds[1][2] - bounds[0][2]),
        ];

        // let center = [
        //     bounds[0][0] + distance[0],
        //     bounds[0][1] + distance[1],
        //     bounds[0][2] + distance[2],
        // ];

        //000 Bottom left (no increment in Xyz)
        //001 Bottom right (increment in x, not in y,z)
        //010 (Increment in y)
        //011 (increment in y and x)
        //...

        let mut children = vec![];
        let mut child_mask = 0;

        for z in 0..2 {
            for y in 0..2 {
                for x in 0..2 {
                    let child_bounds_aa = [
                        bounds[0][0] + distance[0] * x / 2,
                        bounds[0][1] + distance[1] * y / 2,
                        bounds[0][2] + distance[2] * z / 2,
                    ];

                    let child_bounds_bb = [
                        bounds[1][0] - distance[0] * (1 - x) / 2,
                        bounds[1][1] - distance[1] * (1 - y) / 2,
                        bounds[1][2] - distance[2] * (1 - z) / 2,
                    ];
                    
                    // println!("Bounds:");
                    // println!("{:?}", child_bounds_aa);
                    // println!("{:?}", child_bounds_bb);



                    let child = construct_child(cubes, [child_bounds_aa, child_bounds_bb]);
                    match child {
                        Some(node) => {
                            children.push(node);
                            let child_nr = z * 4 + y * 2 + x;
                            child_mask |= 1 << child_nr;
                        },
                        None => {},
                    } 
                }
            }
        }

        if child_mask > 0 {
            println!("Spawning branch node!");
            Some(SparseOctreeNode{
                is_leaf_node: false,
                children: Some(children),
                child_mask: Some(child_mask),
                color: None,
            })
        } else {
            None
        }
        

    }
}

fn construct_octree(cubes: &Vec<Cube>, bounds: [[i32; 3]; 2]) -> Option<SparseOctree> {
    let root_node = construct_child(cubes, bounds);
    match root_node {
        Some(tree) => {
            Some(SparseOctree {
                aabb: bounds,
                max_depth: 14,
                root: tree,
            })
        }
        None => None
    }
}


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
                z_val = f64::abs(f64::floor(z_val as f64));
                let color: [f32; 4] = [rng.gen_range(0..100) as f32 / 100f32, rng.gen_range(0..100) as f32 / 100f32, rng.gen_range(0..100) as f32 / 100f32, 1.0];
                let cube = Cube::new_cube_at(&[(x_offset + x) as f32, (y_offset + y) as f32, z_val as f32], color);
                // println!("z_val: {:?}", z_val);
                cubes.push(cube);
            }
        }
        
        let bounds = [
            [x_offset, y_offset, 0],
            [x_offset + CHUNK_SIZE, y_offset + CHUNK_SIZE, CHUNK_SIZE ]
        ];

        let octree = construct_octree(&cubes, bounds);

        Self {
            cubes: cubes,
            octree: octree,
        }

    }

    pub fn get_cubes(&self) -> &Vec<Cube> {
        return &self.cubes;
    }
}