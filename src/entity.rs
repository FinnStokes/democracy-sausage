use std::rc::Rc;
use std::cell::RefCell;

use crate::geometry::Rectangle;
use crate::colour::interpolate_colour;

use piston_window::{context::Context,G2d};
use noise::{Seedable, NoiseFn};

pub trait Entity {
    fn bounds(&self) -> Rectangle;
    fn select(&self, _pos: [f64; 2]) -> bool { false }
    fn update(&mut self, _dt: f64) -> Vec<Rc<RefCell<dyn Entity>>> { vec![] }
    fn update_selected(&mut self, _dt: f64) {}
    fn grab(&mut self) {}
    fn drop(&mut self) {}
    fn drag(&mut self, _from: [f64; 2], _to: [f64; 2]) {}
    fn draw(&self, context: Context, graphics: &mut G2d);
    fn set_pos(&mut self, _pos: [f64; 2]) {}
    fn is_sausage(&self) -> bool { false }
    fn add_sausage(&mut self, _sausage: &Rc<RefCell<dyn Entity>>) -> bool { false }
    fn set_heat(&mut self, _heat: f64) {}
    fn heat(&self, _pos: [f64; 2]) -> f64 { 0.0 }
    fn expired(&self) -> bool { false }
}

const SAUSAGE_SIZE: [f64; 2] = [15.0, 75.0];
const SAUSAGE_OFFSET: f64 = 10.0;
const BREAD_SIZE: [f64; 2] = [60.0, 60.0];
const PINK: [f32; 4] = [239.0 / 255.0, 115.0 / 255.0, 156.0 / 255.0, 1.0];
const BROWN: [f32; 4] = [204.0 / 255.0, 103.0 / 255.0, 26.0 / 255.0, 1.0];
const BLACK: [f32; 4] = [79.0 / 255.0, 48.0 / 255.0, 24.0 / 255.0, 1.0];
const MIN_HEAT: f64 = 0.05;
const PERLIN_HEAT: f64 = 0.1;

pub struct Sausage {
    pos: [f64; 2],
    heat: f64,
    top_cooked: f64,
    bottom_cooked: f64,
}

impl Sausage {
    pub fn new(pos: [f64; 2]) -> Sausage {
        Sausage{
            pos,
            heat: 0.0,
            top_cooked: 0.0,
            bottom_cooked: 0.0,
        }
    }
}

impl Entity for Sausage {
    fn bounds(&self) -> Rectangle {
        Rectangle::centered(self.pos, SAUSAGE_SIZE)
    }

    fn select(&self, pos: [f64; 2]) -> bool {
        self.bounds().intersect_point(pos)
    }

    fn update(&mut self, dt: f64) -> Vec<Rc<RefCell<dyn Entity>>> {
        self.bottom_cooked += dt * self.heat;
        if rand::random::<f64>() < dt * self.heat * 20.0 {
            let bounds = self.bounds().as_floats();
            vec![Rc::new(RefCell::new(Smoke::new([
                bounds[0] + rand::random::<f64>() * bounds[2],
                bounds[1] + rand::random::<f64>() * bounds[3],
            ], 0.4 * (5.5 - 5.0 * self.bottom_cooked as f32).clamp(0.0, 1.0))))]
        } else {
            vec![]
        }
    }

    fn drag(&mut self, from: [f64; 2], to: [f64; 2]) {
        for i in 0..2 {
            self.pos[i] += to[i] - from[i];
        }
    }

    fn set_pos(&mut self, pos: [f64; 2]) {
        self.pos = pos;
    }

    fn grab(&mut self) {
        let tmp = self.top_cooked;
        self.top_cooked = self.bottom_cooked;
        self.bottom_cooked = tmp;
        self.heat = 0.0;
    }

    fn set_heat(&mut self, heat: f64) {
        self.heat = heat;
    }

    fn draw(&self, context: Context, graphics: &mut G2d) {
        let color = interpolate_colour(&[(PINK, 0.0), (BROWN, 1.0), (BLACK, 1.2)], self.top_cooked as f32);
        piston_window::rectangle(color,
                                 self.bounds().as_floats(),
                                 context.transform,
                                 graphics);
    }

    fn is_sausage(&self) -> bool {
        true
    }
}

pub struct Hotplate {
    bounds: Rectangle,
    noise: noise::Perlin,
}

impl Hotplate {
    pub fn new(pos: [f64; 2], size: [f64; 2], seed: u32) -> Hotplate {
        Hotplate{
            bounds: Rectangle::new(pos, size),
            noise: noise::Perlin::new().set_seed(seed),
        }
    }
}

impl Entity for Hotplate {
    fn bounds(&self) -> Rectangle {
        self.bounds.clone()
    }

    fn draw(&self, context: Context, graphics: &mut G2d) {
        piston_window::rectangle([0.2, 0.15, 0.25, 1.0],
                                 self.bounds().as_floats(),
                                 context.transform,
                                 graphics);
        // let bounds = self.bounds().as_floats();
        // for x in 0..100 {
        //     for y in 0..100 {
        //         let rect = [
        //             bounds[0] + (x as f64) / 100.0 * bounds[2],
        //             bounds[1] + (y as f64) / 100.0 * bounds[3],
        //             bounds[2] / 100.0,
        //             bounds[3] / 100.0,
        //         ];
        //         piston_window::rectangle([20.0 * self.heat([rect[0], rect[1]]) as f32, 0.0, 0.2, 1.0],
        //                                  rect, context.transform, graphics);
        //     }
        // }
    }

    fn heat(&self, pos: [f64; 2]) -> f64 {
        if !self.bounds.intersect_point(pos) {
            0.0
        } else {
            let bounds = self.bounds().as_floats();
            let x = (pos[0] - bounds[0]) / bounds[2];
            let y = (pos[1] - bounds[1]) / bounds[3];

            (x * std::f64::consts::PI).sin()
                * (y * std::f64::consts::PI).sin()
                * (MIN_HEAT + PERLIN_HEAT * self.noise.get([x, y]).powi(2))
        }
    }
}

pub struct Bread {
    pos: [f64; 2],
    sausages: Vec<Rc<RefCell<dyn Entity>>>,
}

impl Bread {
    pub fn new(pos: [f64; 2]) -> Bread {
        Bread{
            pos,
            sausages: Vec::new(),
        }
    }
}

impl Entity for Bread {
    fn bounds(&self) -> Rectangle {
        Rectangle::centered(self.pos, BREAD_SIZE)
    }

    fn select(&self, pos: [f64; 2]) -> bool {
        self.bounds().intersect_point(pos)
    }

    fn drag(&mut self, from: [f64; 2], to: [f64; 2]) {
        for i in 0..2 {
            self.pos[i] += to[i] - from[i];
        }
        for sausage in self.sausages.iter() {
            sausage.borrow_mut().drag(from, to);
        }
    }

    fn set_pos(&mut self, pos: [f64; 2]) {
        self.drag(self.pos, pos);
    }

    fn draw(&self, context: Context, graphics: &mut G2d) {
        let inner_size = Rectangle::centered(self.pos, [BREAD_SIZE[0] - 6.0, BREAD_SIZE[1] - 6.0]);
        piston_window::rectangle([194.0 / 255.0, 153.0 / 255.0, 26.0 / 255.0, 1.0],
                                 self.bounds().as_floats(),
                                 context.transform,
                                 graphics);
        piston_window::rectangle([255.0 / 255.0, 246.0 / 255.0, 206.0 / 255.0, 1.0],
                                 inner_size.as_floats(),
                                 context.transform,
                                 graphics);
        for sausage in self.sausages.iter() {
            sausage.borrow().draw(context, graphics);
        }
    }

    fn add_sausage(&mut self, sausage: &Rc<RefCell<dyn Entity>>) -> bool {
        if self.sausages.len() < 2 {
            if self.sausages.len() == 0 {
                sausage.borrow_mut().set_pos(self.pos.clone());
            } else {
                self.sausages[0].borrow_mut().set_pos([self.pos[0] - SAUSAGE_OFFSET, self.pos[1]]); 
                sausage.borrow_mut().set_pos([self.pos[0] + SAUSAGE_OFFSET, self.pos[1]]);
            }
            self.sausages.push(sausage.clone());
            true
        } else {
            false
        }
    }
}

pub struct Smoke {
    pos: [f64; 2],
    age: f64,
    colour: f32,
}

impl Smoke {
    fn new(pos: [f64; 2], colour: f32) -> Smoke {
        Smoke{pos, age: 0.0, colour}
    }
}

impl Entity for Smoke {
    fn bounds(&self) -> Rectangle {
        let r = 10.0 + self.age * 5.0;
        Rectangle::centered(self.pos, [2.0 * r, 2.0 * r])
    }

    fn update(&mut self, dt: f64) -> Vec<Rc<RefCell<dyn Entity>>> {
        self.age += dt;
        self.pos = [self.pos[0] + dt * 20.0, self.pos[1] + dt * 10.0];
        vec![]
    }

    fn expired(&self) -> bool {
        self.age > 5.0
    }

    fn draw(&self, context: Context, graphics: &mut G2d) {
        piston_window::ellipse([self.colour, self.colour, self.colour, 0.5 * (1.0 - (self.age as f32 / 5.0)).max(0.0)],
                               self.bounds().as_floats(),
                               context.transform,
                               graphics);
    }
}
