use std::time::{Duration, Instant};

use image::imageops::FilterType::CatmullRom;
use image::*;
use rand::prelude::*;

use crate::constants::*;
use crate::heightmap::*;

struct Position {
    x: f64,
    y: f64,
}

pub struct Game {
    // game state
    position: Position,

    // implementation
    pub heightmap: Heightmap,
    pub pattern: DynamicImage,

    depthmap_base: DynamicImage,
    elapsed: Duration,
    last_tick: Instant,
}

impl Game {
    pub fn new() -> Self {
        let depth_bytes = include_bytes!("../assets/teddy.jpg");
        let depthmap_base = image::load_from_memory(depth_bytes).unwrap();
        let depthmap_base =
            depthmap_base
                .rotate180()
                .resize(X_RES as u32, Y_RES as u32 - 40, CatmullRom);

        let pattern = DynamicImage::ImageRgba8(RgbaImage::new(PATTERN_WIDTH as u32, Y_RES as u32));

        Self {
            heightmap: Heightmap::new(X_RES, Y_RES),
            elapsed: Duration::default(),
            last_tick: Instant::now(),
            position: Position { x: 0.0, y: 0.0 },
            pattern,
            depthmap_base,
        }
    }

    /// Update game state once per simulation frame.
    pub fn update(self: &mut Self) {
        // update timer
        let now = Instant::now();
        let delta = now.duration_since(self.last_tick);
        self.elapsed += delta;
        self.last_tick = now;

        // update heightmap
        self.heightmap.update(|x: usize, y: usize| -> f32 {
            let x = (x as i32 - self.position.x as i32) as u32;
            let y = (y as i32 - self.position.y as i32) as u32;

            if self.depthmap_base.in_bounds(x, y) {
                unsafe {
                    let rgba = self.depthmap_base.unsafe_get_pixel(x, y).to_luma().0;
                    rgba[0] as f32 / 255.0
                }
            } else {
                0.0
            }
        });

        let mut rng = rand::thread_rng();

        for y in 0..self.pattern.height() {
            for x in 0..self.pattern.width() {
                let noise: u8 = rng.gen_range(0..=255);
                let pixel = Rgba([noise, noise, noise, 255]);
                self.pattern.put_pixel(x, y, pixel);
            }
        }

        // markers for easier eye adjustment
        const BLUE: Rgba<u8> = Rgba([0, 0, 255, 255]);
        for y in 0..8 {
            for x in 0..4 {
                self.pattern.put_pixel(x, y, BLUE);
            }
        }
    }

    pub fn on_mouse_move(self: &mut Self, dx: f64, dy: f64) {
        self.position.x += dx;
        self.position.y -= dy;
    }
}
