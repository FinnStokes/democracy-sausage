use piston_window::{context::Context,G2d};

use crate::entity::{Entity, Selection, Cookable, Filling, Bread, Hotplate, Table, Bottle, Condiment, ChoppingBoard, Queue};

use std::cell::RefCell;
use std::rc::Rc;

pub struct Scene(Vec<Rc<RefCell<dyn Entity>>>);

impl Scene {
    pub fn new() -> Scene {
        let hotplates: Vec<Rc<RefCell<dyn Entity>>> = vec![
            Rc::new(RefCell::new(Hotplate::new([200.0, 200.0], [420.0, 200.0], rand::random()))),
        ];
        Scene(vec![
              Rc::new(RefCell::new(Queue::new([180.0, 50.0], [720.0, 50.0], 4))),
              Rc::new(RefCell::new(Table::new([-40.0, 200.0], [220.0, 440.0]))),
              Rc::new(RefCell::new(ChoppingBoard::new([100.0, 400.0]))),
              hotplates[0].clone(),
              Rc::new(RefCell::new(Bread::new([102.5, 305.0]))),
              Rc::new(RefCell::new(Bread::new([120.0, 305.0]))),
              Rc::new(RefCell::new(Bread::new([137.5, 305.0]))),
              Rc::new(RefCell::new(Cookable::new(Filling::Sausage, [80.0, 240.0]))),
              Rc::new(RefCell::new(Cookable::new(Filling::Sausage, [100.0, 240.0]))),
              Rc::new(RefCell::new(Cookable::new(Filling::Sausage, [120.0, 240.0]))),
              Rc::new(RefCell::new(Cookable::new(Filling::Sausage, [140.0, 240.0]))),
              Rc::new(RefCell::new(Cookable::new(Filling::Sausage, [160.0, 240.0]))),
              Rc::new(RefCell::new(Cookable::new(Filling::VeggiePatty, [30.0, 260.0]))),
              Rc::new(RefCell::new(Cookable::new(Filling::VeggiePatty, [30.0, 305.0]))),
              Rc::new(RefCell::new(Bottle::new(Condiment::Sauce, [15.0, 180.0]))),
              Rc::new(RefCell::new(Bottle::new(Condiment::Mustard, [45.0, 180.0]))),
        ])
    }

    pub fn draw(&self, context: Context, graphics: &mut G2d) {
        for e in self.0.iter() {
            e.borrow().draw(context, graphics);
        }
    }

    pub fn update(&mut self, dt: f64) {
        let mut new = vec![];
        for e in self.0.iter() {
            new.append(&mut e.borrow_mut().update(dt));
        }
        self.0.drain_filter(|e| e.borrow().expired()).count();
        self.0.append(&mut new);
    }

    pub fn select(&mut self, pos: [f64; 2]) -> Option<Rc<RefCell<dyn Entity>>> {
        enum Action {
            Return(Rc<RefCell<dyn Entity>>),
            Append(Rc<RefCell<dyn Entity>>),
        }

        match self.0.iter().rev().find_map(|e| {
            match e.borrow_mut().select(pos) {
                Selection::None => None,
                Selection::This => Some(Action::Return(e.clone())),
                Selection::New(entity) => Some(Action::Append(entity)),
            }
        }) {
            Some(Action::Return(e)) => Some(e),
            Some(Action::Append(e)) => {
                self.0.push(e.clone());
                Some(e)
            },
            None => None,
        }
    }

    pub fn grabbed(&mut self, entity: &Rc<RefCell<dyn Entity>>) {
        let n = self.0.iter().enumerate().filter(|(_, e)| Rc::ptr_eq(e, entity)).next().unwrap().0;
        let e = self.0.remove(n);
        self.0.push(e);
    }

    pub fn dropped(&mut self, entity: &Rc<RefCell<dyn Entity>>) {
        if !entity.borrow().topping().is_none() {
            for e in self.0.iter().rev().filter(|e| !Rc::ptr_eq(e, entity) && e.borrow().bounds().intersect_rect(&entity.borrow().bounds())) {
                let res = e.borrow_mut().add_topping(entity);
                match res {
                    Selection::This => {
                        let n = self.0.iter().enumerate().filter(|(_, e)| Rc::ptr_eq(e, entity)).next().unwrap().0;
                        self.0.remove(n);
                        return;
                    },
                    Selection::New(_) => {
                        return;
                    },
                    Selection::None => {},
                }
            }
        }

        if let Some(order) = entity.borrow().order() {
            let mut mood = None;
            for e in self.0.iter().rev().filter(|e| !Rc::ptr_eq(e, entity) && e.borrow().bounds().intersect_rect(&entity.borrow().bounds())) {
                mood = e.borrow_mut().deliver_order(order);
                if mood.is_some() {
                    break;
                }
            }
            if let Some(mood) = mood {
                let n = self.0.iter().enumerate().filter(|(_, e)| Rc::ptr_eq(e, entity)).next().unwrap().0;
                self.0.remove(n);
                return;
            }
        }

        let pos = entity.borrow().bounds().centre();
        entity.borrow_mut().set_heat(self.0.iter().filter(|e| !Rc::ptr_eq(e, entity)).map(|e| e.borrow().heat(pos)).sum());
    }
}
