pub fn normalize_vector(vector: &[f32; 3]) -> [f32; 3] {
    let magnitude = (vector[0].powf(2.0) + vector[1].powf(2.0) + vector[2].powf(2.0)).sqrt();
    [vector[0] / magnitude, vector[1] / magnitude, vector[2] / magnitude]
}

pub fn cross_vector(v: &[f32; 3], u: &[f32; 3]) -> [f32; 3] {
    [
        v[1] * u[2] - v[2] * u[1],
        v[2] * u[0] - v[0] * u[2],
        v[0] * u[1] - v[1] * u[0],
    ]
}
