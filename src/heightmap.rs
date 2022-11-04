use crate::constants::X_RES;

pub struct Heightmap {
    width: usize,
    height: usize,
    pixels: Vec<f32>,
}

impl Heightmap {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![0.0; width * height],
        }
    }

    pub fn at(&self, x: usize, y: usize) -> f32 {
        self.pixels[x + y * self.width]
    }

    pub fn update<F>(&mut self, fun: F)
    where
        F: Fn(usize, usize) -> f32,
    {
        for y in 0..self.height {
            let row = y * X_RES;
            for x in 0..self.width {
                self.pixels[x + row] = fun(x, y);
            }
        }
    }
}
