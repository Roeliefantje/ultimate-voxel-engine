// File made my chatgpt, looks good afaik
//Todo!: Properly learn quaternions.

#[derive(Debug, Clone, Copy)]
pub struct Quaternion {
    w: f32,  // Scalar part
    x: f32,  // X component of the vector part
    y: f32,  // Y component of the vector part
    z: f32,  // Z component of the vector part
}

impl Quaternion {
    // Create a new quaternion
    pub fn new(w: f32, x: f32, y: f32, z: f32) -> Self {
        Self { w, x, y, z }
    }

    // Multiply two quaternions
    pub fn multiply(self, other: Quaternion) -> Quaternion {
        Quaternion {
            w: self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
            x: self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            y: self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            z: self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
        }
    }

    // Conjugate of a quaternion (needed for inversion since unit quaternions have norm 1)
    pub fn conjugate(self) -> Quaternion {
        Quaternion::new(self.w, -self.x, -self.y, -self.z)
    }

    // Rotate a 3D vector by this quaternion
    pub fn rotate_vector(self, vector: [f32; 3]) -> [f32; 3] {
        let q_vec = Quaternion::new(0.0, vector[0], vector[1], vector[2]);
        let q_rotated = self.multiply(q_vec).multiply(self.conjugate());
        [q_rotated.x, q_rotated.y, q_rotated.z]
    }

    // Create a unit quaternion from an axis and an angle (radians)
    pub fn from_axis_angle(axis: [f32; 3], angle: f32) -> Self {
        let half_angle = angle / 2.0;
        let (sin_half_angle, cos_half_angle) = half_angle.sin_cos();
        Quaternion::new(
            cos_half_angle,
            axis[0] * sin_half_angle,
            axis[1] * sin_half_angle,
            axis[2] * sin_half_angle,
        )
    }
}