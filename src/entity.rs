use std::rc::Rc;
use std::cell::RefCell;

use crate::geometry::Rectangle;
use crate::colour::interpolate_colour;

use piston_window::{context::Context,G2d};

pub trait Entity {
    fn bounds(&self) -> Rectangle;
    fn select(&self, _pos: [f64; 2]) -> bool { false }
    fn update(&mut self, _dt: f64) {}
    fn update_selected(&mut self, _dt: f64) {}
    fn grab(&mut self) {}
    fn drop(&mut self) {}
    fn drag(&mut self, _from: [f64; 2], _to: [f64; 2]) {}
    fn draw(&self, context: Context, graphics: &mut G2d);
    fn set_pos(&mut self, _pos: [f64; 2]) {}
    fn is_sausage(&self) -> bool { false }
    fn add_sausage(&mut self, _sausage: &Rc<RefCell<dyn Entity>>) -> bool { false }
}

const SAUSAGE_SIZE: [f64; 2] = [15.0, 75.0];
const SAUSAGE_OFFSET: f64 = 10.0;
const BREAD_SIZE: [f64; 2] = [60.0, 60.0];
const PINK: [f32; 4] = [239.0 / 255.0, 115.0 / 255.0, 156.0 / 255.0, 1.0];
const BROWN: [f32; 4] = [204.0 / 255.0, 103.0 / 255.0, 26.0 / 255.0, 1.0];
const BLACK: [f32; 4] = [79.0 / 255.0, 48.0 / 255.0, 24.0 / 255.0, 1.0];
const COOK_SPEED: f64 = 0.02;

pub struct Sausage {
    pos: [f64; 2],
    lifted: bool,
    top_cooked: f64,
    bottom_cooked: f64,
    hotplates: Vec<Rc<RefCell<dyn Entity>>>,
}

impl Sausage {
    pub fn new(pos: [f64; 2], hotplates: Vec<Rc<RefCell<dyn Entity>>>) -> Sausage {
        Sausage{
            pos,
            lifted: false,
            top_cooked: 0.0,
            bottom_cooked: 0.0,
            hotplates,
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

    fn update(&mut self, dt: f64) {
        if !self.lifted {
            let bounds = self.bounds();
            for hotplate in &self.hotplates {
                if bounds.intersect_rect(hotplate.borrow().bounds()) {
                    self.bottom_cooked += dt * COOK_SPEED;
                    return;
                }
            }
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
        self.lifted = true;
    }

    fn drop(&mut self) {
        self.lifted = false;
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
}

impl Hotplate {
    pub fn new(pos: [f64; 2], size: [f64; 2]) -> Hotplate {
        Hotplate{
            bounds: Rectangle::new(pos, size),
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

