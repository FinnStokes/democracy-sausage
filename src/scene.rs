use piston_window::{context::Context,G2d};

use crate::entity::{Entity, Sausage};

use std::cell::RefCell;
use std::rc::Rc;

pub struct Scene(Vec<Rc<RefCell<dyn Entity>>>);

impl Scene {
    pub fn new() -> Scene {
        Scene(vec![
              Rc::new(RefCell::new(Sausage::new([100.0, 100.0]))),
              Rc::new(RefCell::new(Sausage::new([120.0, 100.0]))),
              Rc::new(RefCell::new(Sausage::new([140.0, 100.0]))),
              Rc::new(RefCell::new(Sausage::new([160.0, 100.0]))),
        ])
    }

    pub fn draw(&self, context: Context, graphics: &mut G2d) {
        for e in self.0.iter() {
            e.borrow().draw(context, graphics);
        }
    }

    pub fn update(&self, dt: f64) {
        for e in self.0.iter() {
            e.borrow_mut().update(dt);
        }
    }

    pub fn select(&self, pos: [f64; 2]) -> Option<Rc<RefCell<dyn Entity>>> {
        for e in self.0.iter() {
            if e.borrow().select(pos) {
                return Some(e.clone())
            }
        }
        None
    }
}
