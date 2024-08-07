pub struct RenderImage {
    pub x_size: usize,
    pub y_size: usize,
    pub pixels: Vec<[f32; 4]>,
}

impl RenderImage {
    pub fn new(x_dimension: usize, y_dimension: usize) -> Self {
        let pixels: Vec<[f32; 4]> = vec![[0.0, 0.0, 0.0, 0.0]; x_dimension * y_dimension];

        Self {
            x_size: x_dimension,
            y_size: y_dimension,
            pixels: pixels,
        }
    }
} 