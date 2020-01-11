use piston_window::{context::Context,G2d};

use crate::entity::{Entity, Sausage, Bread, Hotplate};

use std::cell::RefCell;
use std::rc::Rc;

pub struct Scene(Vec<Rc<RefCell<dyn Entity>>>);

impl Scene {
    pub fn new() -> Scene {
        let hotplates: Vec<Rc<RefCell<dyn Entity>>> = vec![
            Rc::new(RefCell::new(Hotplate::new([200.0, 50.0], [200.0, 200.0]))),
            Rc::new(RefCell::new(Hotplate::new([420.0, 50.0], [200.0, 200.0]))),
        ];
        Scene(vec![
              hotplates[0].clone(),
              hotplates[1].clone(),
              Rc::new(RefCell::new(Bread::new([102.5, 200.0]))),
              Rc::new(RefCell::new(Bread::new([120.0, 200.0]))),
              Rc::new(RefCell::new(Bread::new([137.5, 200.0]))),
              Rc::new(RefCell::new(Sausage::new([80.0, 100.0], hotplates.clone()))),
              Rc::new(RefCell::new(Sausage::new([100.0, 100.0], hotplates.clone()))),
              Rc::new(RefCell::new(Sausage::new([120.0, 100.0], hotplates.clone()))),
              Rc::new(RefCell::new(Sausage::new([140.0, 100.0], hotplates.clone()))),
              Rc::new(RefCell::new(Sausage::new([160.0, 100.0], hotplates.clone()))),
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
        for e in self.0.iter().rev() {
            if e.borrow().select(pos) {
                return Some(e.clone())
            }
        }
        None
    }

    pub fn grabbed(&mut self, entity: &Rc<RefCell<dyn Entity>>) {
        let n = self.0.iter().enumerate().filter(|(_, e)| Rc::ptr_eq(e, entity)).next().unwrap().0;
        let e = self.0.remove(n);
        self.0.push(e);
    }

    pub fn dropped(&mut self, entity: &Rc<RefCell<dyn Entity>>) {
        if entity.borrow().is_sausage() {
            for e in self.0.iter().rev().filter(|e| !Rc::ptr_eq(e, entity) && e.borrow().bounds().intersect_rect(entity.borrow().bounds())) {
                if e.borrow_mut().add_sausage(entity) {
                    let n = self.0.iter().enumerate().filter(|(_, e)| Rc::ptr_eq(e, entity)).next().unwrap().0;
                    self.0.remove(n);
                    return;
                }
            }
        }
    }
}
