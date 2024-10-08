struct Cube {
    min: vec3<f32>,
    max: vec3<f32>,
    color: vec4<f32>,
}

struct Camera {
    origin: vec3<f32>,
    forward_vec: vec3<f32>,
    left_vec: vec3<f32>,
    up_vec: vec3<f32>,
}

struct Ray {
    origin: vec3<f32>,
    velocity: vec3<f32>,
    distance: f32,
    color: vec4<f32>,
}

@group(0) @binding(0) var<uniform> amount_of_cubes: f32;
@group(0) @binding(1) var<uniform> camera: Camera;
@group(0) @binding(2) var<storage, read> cubes: array<Cube>;
@group(0) @binding(3) var<storage, read_write> screen_pixels: array<vec4<f32>>;

const maxfloat = 0x1.fffffep+127f;
const minfloat = -0x1.fffffep+127f;

fn intersect_ray(cube: Cube, ray: Ray) -> Ray {
    //Branchless AABB testing right now, we want to change this to use DDA with a Spare Octree instead.
    //This should help speedup the code and not having to store the aabb should hopefully help reduce memory as well.
    //We would need a way to find out the collission points and then figure the normal out from there.
    //Which we can then use to cast a shadow ray.
    //But A leaf node (cube) should hopefully only contain the color from a cube.
    //We could do a color or even store an index to a texturemap or something in the future.
    //Lets say we have 8 colors we can use in the voxel engine, if we store them in a lookup, we could save memory by giving the index instead.
    //This would also allow for more compelx textures in the future, but I plan on making the voxels so small that they do not need textures.
    //I plan on having a buffer of Octree's, non-leaf nodes would store the index of its children.
    //Im not sure yet how I can store this as sparcely as possible, as in, I do not want to store a lot of 0's if an octree only has 1 node.
    //Maybe we can use a bitflag, this would be 8 bits, but then I'm not entirely sure how I could store the indices.
    //I'd imagine the octree has to have a set size for the buffer, so I cant exactly use a vec for the children.

    var new_ray = ray;

    let inv_velocity = vec3<f32>(1 / ray.velocity.x, 1 / ray.velocity.y, 1 / ray.velocity.z);

    let tx1 = (cube.min.x - ray.origin.x) * inv_velocity.x;
    let tx2 = (cube.max.x - ray.origin.x) * inv_velocity.x;

    var tmin = min(tx1, tx2);
    var tmax = max(tx1, tx2);

    let ty1 = (cube.min.y - ray.origin.y) * inv_velocity.y;
    let ty2 = (cube.max.y - ray.origin.y) * inv_velocity.y;

    tmin = max(tmin, min(ty1, ty2));
    tmax = min(tmax, max(ty1, ty2));

    let tz1 = (cube.min.z - ray.origin.z) * inv_velocity.z;
    let tz2 = (cube.max.z - ray.origin.z) * inv_velocity.z;

    tmin = max(tmin, min(tz1, tz2));
    tmax = min(tmax, max(tz1, tz2));
    
    if (tmax >= max(0.0, tmin) && tmin < ray.distance) {
        new_ray.color = cube.color;
        new_ray.distance = tmin;
    }

    return new_ray;
}

@compute
@workgroup_size(64)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    //Todo! Fix FOV
    let total = arrayLength(&screen_pixels);
    let index = global_invocation_id.x + 1920 * global_invocation_id.y;

    if (index >= total) {
        return;
    }

    let plane_center = camera.origin + camera.forward_vec * 3.0;
    let aspect_ratio = 16.0 / 9.0;

    let top_left = plane_center + aspect_ratio * camera.left_vec + camera.up_vec;

    let v = f32(global_invocation_id.y) / 1079.0;
    let u = f32(global_invocation_id.x) / 1919.0;

    let screen_place = top_left - camera.left_vec * u * 2.0 * aspect_ratio - camera.up_vec * v * 2.0;

    let velocity = screen_place - camera.origin;

    var ray: Ray = Ray(
        camera.origin,
        velocity,
        maxfloat,
        vec4<f32>(v, u, amount_of_cubes, 1.0),
    );

    for (var i: i32 = 0; i < i32(amount_of_cubes); i = i + 1){
        ray = intersect_ray(cubes[i], ray);
    }
    
    

    screen_pixels[index] = ray.color;
}