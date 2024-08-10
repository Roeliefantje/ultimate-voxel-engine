use bytemuck::{Pod, Zeroable};

use super::ray::Ray;
#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Cube {
    pub min: [f32; 4],
    pub max: [f32; 4],
    pub color: [f32; 4],    
}

impl Cube {
    pub fn new_cube_at(loc: &[f32; 3], color: [f32; 4]) -> Self {
        Self {
            min: [loc[0], loc[1], loc[2], 0.0],
            max: [loc[0] + 1.0, loc[1] + 1.0, loc[2] + 1.0, 0.0],
            color: color,
        }
    }

    pub fn intersect_ray(&self, ray: &mut Ray){
        //https://tavianator.com/cgit/dimension.git/tree/libdimension/bvh/bvh.c#n196
        //https://education.siggraph.org/static/HyperGraph/raytrace/rtinter3.htm
        //Not the best way yet, but more intuitive.
        let mut tnear = f32::MIN;
        let mut tfar = f32::MAX;

        for d in 0..3 as usize {

            if ray.velocity[d] == 0.0 {
                if ray.origin[d] >= self.min[d] && ray.origin[d] <= self.max[d]{
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