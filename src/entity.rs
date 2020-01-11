use crate::geometry::Rectangle;
use crate::colour::interpolate_colour;

use piston_window::{context::Context,G2d};

pub trait Entity {
    fn select(&self, _pos: [f64; 2]) -> bool { false }
    fn update(&mut self, _dt: f64) {}
    fn update_selected(&mut self, _dt: f64) {}
    fn grab(&mut self) {}
    fn drop(&mut self) {}
    fn drag(&mut self, _from: [f64; 2], _to: [f64; 2]) {}
    fn draw(&self, context: Context, graphics: &mut G2d);
}

const SAUSAGE_SIZE: [f64; 2] = [10.0, 40.0];
const PINK: [f32; 4] = [239.0 / 255.0, 115.0 / 255.0, 156.0 / 255.0, 1.0];
const BROWN: [f32; 4] = [204.0 / 255.0, 103.0 / 255.0, 26.0 / 255.0, 1.0];
const BLACK: [f32; 4] = [79.0 / 255.0, 48.0 / 255.0, 24.0 / 255.0, 1.0];

pub struct Sausage {
    pos: [f64; 2],
    top_cooked: f64,
    bottom_cooked: f64,
}

impl Sausage {
    pub fn new(pos: [f64; 2]) -> Sausage {
        Sausage{
            pos,
            top_cooked: 0.0,
            bottom_cooked: 0.0,
        }
    }
}

impl Entity for Sausage {
    fn select(&self, pos: [f64; 2]) -> bool {
        let bounds = Rectangle::centered(self.pos, SAUSAGE_SIZE);
        bounds.intersect_point(pos)
    }

    fn drag(&mut self, from: [f64; 2], to: [f64; 2]) {
        for i in 0..2 {
            self.pos[i] += to[i] - from[i];
        }
    }

    fn drop(&mut self) {
        let tmp = self.top_cooked;
        self.top_cooked = self.bottom_cooked;
        self.bottom_cooked = tmp;
    }

    fn draw(&self, context: Context, graphics: &mut G2d) {
        let bounds = Rectangle::centered(self.pos, SAUSAGE_SIZE);
        let color = interpolate_colour(&[(PINK, 0.0), (BROWN, 1.0), (BLACK, 1.2)], self.top_cooked as f32);
        piston_window::rectangle(color,
                                 bounds.as_floats(),
                                 context.transform,
                                 graphics);
    }
}
