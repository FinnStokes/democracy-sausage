use std::rc::Rc;
use std::cell::RefCell;

use crate::geometry::Rectangle;
use crate::colour::interpolate_colour;

use piston_window::{context::Context,G2d};
use noise::{Seedable, NoiseFn};
use rand::Rng;

pub enum Selection {
    None,
    This,
    New(Rc<RefCell<dyn Entity>>),
}

pub trait Entity {
    fn bounds(&self) -> Rectangle;
    fn select(&mut self, _pos: [f64; 2]) -> Selection { Selection::None }
    fn update(&mut self, _dt: f64) -> Vec<Rc<RefCell<dyn Entity>>> { vec![] }
    fn update_selected(&mut self, _dt: f64) {}
    fn grab(&mut self) {}
    fn drop(&mut self) {}
    fn drag(&mut self, _from: [f64; 2], _to: [f64; 2]) {}
    fn draw(&self, context: Context, graphics: &mut G2d);
    fn set_pos(&mut self, _pos: [f64; 2]) {}
    fn topping(&self) -> Option<Topping> { None }
    fn add_topping(&mut self, _topping: &Rc<RefCell<dyn Entity>>) -> bool { false }
    fn add_to(&mut self, _pos: [f64; 2], _others: &[Rc<RefCell<dyn Entity>>]) -> bool { false }
    fn set_heat(&mut self, _heat: f64) {}
    fn heat(&self, _pos: [f64; 2]) -> f64 { 0.0 }
    fn expired(&self) -> bool { false }
}

const SAUSAGE_SIZE: [f64; 2] = [15.0, 75.0];
const PATTY_SIZE: [f64; 2] = [55.0, 40.0];
const SAUSAGE_OFFSET: f64 = 10.0;
const BREAD_SIZE: [f64; 2] = [60.0, 60.0];
const PINK: [f32; 4] = [239.0 / 255.0, 115.0 / 255.0, 156.0 / 255.0, 1.0];
const YELLOW: [f32; 4] = [252.0 / 255.0, 217.0 / 255.0, 75.0 / 255.0, 1.0];
const ORANGE: [f32; 4] = [247.0 / 255.0, 155.0 / 255.0, 27.0 / 255.0, 1.0];
const BROWN: [f32; 4] = [204.0 / 255.0, 103.0 / 255.0, 26.0 / 255.0, 1.0];
const BLACK: [f32; 4] = [79.0 / 255.0, 48.0 / 255.0, 24.0 / 255.0, 1.0];
const GREEN: [f32; 4] = [53.0 / 255.0, 201.0 / 255.0, 12.0 / 255.0, 1.0];
const BOARD: [f32; 4] = [156.0 / 244.0, 244.0 / 241.0, 243.0 / 255.0, 1.0];
const MIN_HEAT: f64 = 0.03;
const PERLIN_HEAT: f64 = 0.07;
const CHOP_SPEED: f64 = 0.25;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Topping {
    Filling(Filling),
    Onion,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Filling {
    Sausage,
    VeggiePatty,
}

pub struct Cookable {
    pos: [f64; 2],
    heat: f64,
    top_cooked: f64,
    bottom_cooked: f64,
    flipped: bool,
    kind: Filling,
}

impl Cookable {
    pub fn new(kind: Filling, pos: [f64; 2]) -> Cookable {
        Cookable{
            pos,
            heat: 0.0,
            top_cooked: 0.0,
            bottom_cooked: 0.0,
            flipped: false,
            kind,
        }
    }
}

impl Entity for Cookable {
    fn bounds(&self) -> Rectangle {
        match self.kind {
            Filling::Sausage => Rectangle::centered(self.pos, SAUSAGE_SIZE),
            Filling::VeggiePatty => Rectangle::centered(self.pos, PATTY_SIZE),
        }
    }

    fn select(&mut self, pos: [f64; 2]) -> Selection {
        if self.bounds().intersect_point(pos) {
            Selection::This
        } else {
            Selection::None
        }
    }

    fn update(&mut self, dt: f64) -> Vec<Rc<RefCell<dyn Entity>>> {
        self.bottom_cooked += dt * self.heat * match self.kind {
            Filling::Sausage => 1.0,
            Filling::VeggiePatty => 0.5,
        };
        if rand::random::<f64>() < dt * self.heat * 20.0 {
            let bounds = self.bounds().as_floats();
            vec![Rc::new(RefCell::new(Smoke::new([
                bounds[0] + rand::random::<f64>() * bounds[2],
                bounds[1] + rand::random::<f64>() * bounds[3],
            ], 0.4 * (3.8 - 3.0 * self.bottom_cooked as f32).clamp(0.0, 1.0))))]
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
        self.flipped = !self.flipped;
        self.heat = 0.0;
    }

    fn set_heat(&mut self, heat: f64) {
        self.heat = heat;
    }

    fn draw(&self, context: Context, graphics: &mut G2d) {
        match self.kind {
            Filling::Sausage => {
                let color = interpolate_colour(&[(PINK, 0.0), (BROWN, 1.0), (BLACK, 1.4)], self.top_cooked as f32);
                piston_window::rectangle(color,
                                         self.bounds().as_floats(),
                                         context.transform,
                                         graphics);
            },
            Filling::VeggiePatty => {
                let color = interpolate_colour(&[(YELLOW, 0.0), (ORANGE, 1.0), (BLACK, 1.4)], self.top_cooked as f32);
                let bounds = self.bounds().as_floats();
                rounded_rectangle(color,
                                  bounds,
                                  10.0,
                                  context.transform,
                                  graphics);
                let peas = if self.flipped {
                    vec![(0.2, 0.3), (0.85, 0.2), (0.3, 0.7), (0.6, 0.35), (0.7, 0.75)]
                } else {
                    vec![(0.4, 0.3), (0.8, 0.5), (0.15, 0.7), (0.3, 0.45)]
                };
                for offset in &peas {
                    let rect = Rectangle::centered([bounds[0] + offset.0 * bounds[2], bounds[1] + offset.1 * bounds[3]],
                                                   [6.0, 6.0]);
                    piston_window::ellipse(GREEN,
                                           rect.as_floats(),
                                           context.transform,
                                           graphics);
                }
            },
        }
    }

    fn topping(&self) -> Option<Topping> {
        Some(Topping::Filling(self.kind))
    }

    fn add_to(&mut self, pos: [f64; 2], others: &[Rc<RefCell<dyn Entity>>]) -> bool {
        let mut other_fillings = others.iter()
            .filter(|e| if let Some(Topping::Filling(_)) = e.borrow().topping() {
                true
            } else {
                false
            });
        match other_fillings.next() {
            None => {
                self.set_pos(pos);
                true
            },
            Some(f) => if f.borrow().topping().contains(&Topping::Filling(Filling::Sausage)) && self.kind == Filling::Sausage {
                if let None = other_fillings.next() {
                    self.pos = [pos[0] + SAUSAGE_OFFSET, pos[1]];
                    f.borrow_mut().set_pos([pos[0] - SAUSAGE_OFFSET, pos[1]]);
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
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

pub struct Table {
    bounds: Rectangle,
}

impl Table {
    pub fn new(pos: [f64; 2], size: [f64; 2]) -> Table {
        Table{
            bounds: Rectangle::new(pos, size),
        }
    }
}

fn rounded_rectangle(colour: [f32; 4], bounds: [f64; 4], r: f64, transform: [[f64; 3]; 2], graphics: &mut G2d) {
    piston_window::rectangle(colour,
                             [bounds[0] + r, bounds[1], bounds[2] - 2.0 * r, bounds[3]],
                             transform,
                             graphics);
    piston_window::rectangle(colour,
                             [bounds[0], bounds[1] + r, bounds[2], bounds[3] - 2.0 * r],
                             transform,
                             graphics);
    piston_window::ellipse(colour,
                           [bounds[0], bounds[1], 2.0 * r, 2.0 * r],
                           transform,
                           graphics);
    piston_window::ellipse(colour,
                           [bounds[0], bounds[1] + bounds[3] - 2.0 * r, 2.0 * r, 2.0 * r],
                           transform,
                           graphics);
    piston_window::ellipse(colour,
                           [bounds[0] + bounds[2] - 2.0 * r, bounds[1], 2.0 * r, 2.0 * r],
                           transform,
                           graphics);
    piston_window::ellipse(colour,
                           [bounds[0] + bounds[2] - 2.0 * r, bounds[1] + bounds[3] - 2.0 * r, 2.0 * r, 2.0 * r],
                           transform,
                           graphics);
}

impl Entity for Table {
    fn bounds(&self) -> Rectangle {
        self.bounds.clone()
    }

    fn draw(&self, context: Context, graphics: &mut G2d) {
        rounded_rectangle([0.95, 1.0, 1.0, 1.0],
                          self.bounds().as_floats(),
                          25.0,
                          context.transform,
                          graphics);
    }
}

pub struct Bread {
    pos: [f64; 2],
    toppings: Vec<Rc<RefCell<dyn Entity>>>,
}

impl Bread {
    pub fn new(pos: [f64; 2]) -> Bread {
        Bread{
            pos,
            toppings: Vec::new(),
        }
    }
}

impl Entity for Bread {
    fn bounds(&self) -> Rectangle {
        Rectangle::centered(self.pos, BREAD_SIZE)
    }

    fn select(&mut self, pos: [f64; 2]) -> Selection {
        if self.bounds().intersect_point(pos) {
            Selection::This
        } else {
            Selection::None
        }
    }

    fn drag(&mut self, from: [f64; 2], to: [f64; 2]) {
        for i in 0..2 {
            self.pos[i] += to[i] - from[i];
        }
        for topping in &self.toppings {
            topping.borrow_mut().drag(from, to);
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
        for topping in &self.toppings {
            topping.borrow().draw(context, graphics);
        }
    }

    fn add_topping(&mut self, topping: &Rc<RefCell<dyn Entity>>) -> bool {
        if topping.borrow_mut().add_to(self.pos.clone(), &self.toppings) {
            self.toppings.push(topping.clone());
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

pub struct ChoppingBoard {
    pos: [f64; 2],
    progress: f64,
    onions: Vec<Onion>,
}

impl ChoppingBoard {
    pub fn new(pos: [f64; 2]) -> ChoppingBoard {
        ChoppingBoard{
            pos,
            progress: 0.0,
            onions: vec![Onion::new(pos), Onion::new(pos), Onion::new(pos), Onion::new(pos)],
        }
    }
}

fn knife(centre: [f64; 2], transform: [[f64; 3]; 2], graphics: &mut G2d) {
    piston_window::polygon([145.0 / 255.0, 145.0 / 255.0, 145.0 / 255.0, 1.0],
                           &[[centre[0] - 60.0, centre[1] - 48.0],
                             [centre[0] - 30.0, centre[1] - 50.0],
                             [centre[0] + 36.0, centre[1] - 50.0],
                             [centre[0] + 36.0, centre[1] - 36.0],
                             [centre[0], centre[1] - 37.0],
                             [centre[0] - 24.0, centre[1] - 38.0],
                             [centre[0] - 48.0, centre[1] - 43.0]],
                           transform,
                           graphics);
    piston_window::rectangle([0.1, 0.0, 0.0, 1.0],
                             [centre[0] + 36.0, centre[1] - 49.0, 36.0, 10.0],
                             transform,
                             graphics);
}

pub fn interpolate_path(points: &[([f64; 2], f64)], point: f64) -> [f64; 2] {
    let lower = points.iter().filter(|(_, x)| *x < point).last();
    let upper = points.iter().filter(|(_, x)| *x >= point).next();

    match (lower, upper) {
        (None, None) => panic!("Gradient missing reference points"),
        (Some((colour, _)), None) => *colour,
        (None, Some((colour, _))) => *colour,
        (Some((lc, l)), Some((uc, u))) => {
            let s = (point - l) / (u - l);
            [
                lc[0] * (1.0 - s) + uc[0] * s,
                lc[1] * (1.0 - s) + uc[1] * s,
            ]
        },
    }
}

impl Entity for ChoppingBoard {
    fn bounds(&self) -> Rectangle {
        Rectangle::centered(self.pos, [90.0, 120.0])
    }

    fn select(&mut self, pos: [f64; 2]) -> Selection {
        if self.bounds().intersect_point(pos) {
            if self.progress < 1.0 {
                Selection::This
            } else if let Some(onion) = self.onions.pop() {
                if self.onions.len() == 0 {
                    self.progress = 0.0;
                    self.onions = vec![Onion::new(pos), Onion::new(pos), Onion::new(pos), Onion::new(pos)];
                }
                Selection::New(Rc::new(RefCell::new(onion)))
            } else {
                Selection::None
            }
        } else {
            Selection::None
        }
    }

    fn update_selected(&mut self, dt: f64) {
        self.progress = (self.progress + dt * CHOP_SPEED).min(1.0);
    }

    fn draw(&self, context: Context, graphics: &mut G2d) {
        rounded_rectangle(BOARD,
                          self.bounds().as_floats(),
                          2.0,
                          context.transform,
                          graphics);
        let centre = self.bounds().centre();
        if self.progress < 1.0 {
            let size = [50.0, 45.0];
            let onion = Rectangle::centered(centre, size);
            let colour = if self.progress < 0.2 {
                [191.0 / 255.0, 145.0 / 255.0, 59.0 / 255.0, 1.0]
            } else {
                [RAW_ONION[0], RAW_ONION[1], RAW_ONION[2], 1.0]
            };
            piston_window::ellipse(colour,
                                   onion.as_floats(),
                                   context.transform,
                                   graphics);
            piston_window::polygon(colour,
                                   &[[centre[0] + 0.5f64.sqrt() * size[0] / 2.0, centre[1] - 0.5f64.sqrt() * size[1] / 2.0],
                                     [centre[0], centre[1] - 1.2 * size[1] / 2.0],
                                     [centre[0] - 0.5f64.sqrt() * size[0] / 2.0, centre[1] - 0.5f64.sqrt() * size[1] / 2.0]],
                                   context.transform,
                                   graphics);
        } else {
            for onion in &self.onions {
                onion.draw(context, graphics);
            }
        }

        knife(interpolate_path(
            &[(centre, 0.0),
              ([centre[0] - 30.0, centre[1] + 30.0], 0.1),
              ([centre[0] + 30.0, centre[1] + 30.0], 0.2),
              ([centre[0] - 30.0, centre[1] + 35.0], 0.35),
              ([centre[0] + 30.0, centre[1] + 35.0], 0.45),
              ([centre[0] - 30.0, centre[1] + 40.0], 0.6),
              ([centre[0] + 30.0, centre[1] + 40.0], 0.7),
              ([centre[0] - 30.0, centre[1] + 45.0], 0.85),
              ([centre[0] + 30.0, centre[1] + 45.0], 0.95),
              (centre, 1.0)],
            self.progress,
        ), context.transform, graphics);
    }
}

const ONION_LAYERS: usize = 4;
const ONION_PIECES: usize = 3;
const RAW_ONION: [f32; 4] = [1.0, 0.95, 0.9, 0.8];
const COOKED_ONION: [f32; 4] = [214.0 / 255.0, 141.0 / 255.0, 38.0 / 255.0, 0.8];

pub struct Onion {
    pos: [f64; 2],
    heat: f64,
    cooked: [f64; ONION_LAYERS],
    layers: [[OnionPiece; ONION_PIECES]; ONION_LAYERS],
    bounds: Rectangle,
}

pub struct OnionPiece {
    rect: [f64; 4],
    start: f64,
    end: f64,
    thickness: f64,
}

impl Onion {
    pub fn new(pos: [f64; 2]) -> Onion {
        let layers: [[OnionPiece; ONION_PIECES]; ONION_LAYERS] = array_init::array_init(
            |_| array_init::array_init(
                |_| OnionPiece::new()
            )
        );
        let bounds = layers.iter()
            .flatten()
            .map(|p| p.bounds(pos))
            .fold(Rectangle::centered(pos, [0.0, 0.0]), |r, s| r.union(&s));
        Onion{
            pos,
            heat: 0.0,
            cooked: [0.0; ONION_LAYERS],
            layers,
            bounds,
        }
    }

    pub fn scramble(&mut self) {
        let mut rng = rand::thread_rng();
        for i in (1..self.layers.len()).rev() {
            // invariant: elements with index > i have been locked in place.
            let j = rng.gen_range(0, (i + 1) as u32) as usize;
            self.layers.swap(i, j);
            self.cooked.swap(i, j);
        }
        for layer in &mut self.layers {
            for piece in layer {
                piece.scramble();
            }
        }
        self.bounds = self.layers.iter()
            .flatten()
            .map(|p| p.bounds(self.pos))
            .fold(Rectangle::centered(self.pos, [0.0, 0.0]), |r, s| r.union(&s));
    }
}

impl Entity for Onion {
    fn bounds(&self) -> Rectangle {
        self.bounds.clone()
    }

    fn select(&mut self, pos: [f64; 2]) -> Selection {
        if self.bounds().intersect_point(pos) {
            Selection::This
        } else {
            Selection::None
        }
    }

    fn update(&mut self, dt: f64) -> Vec<Rc<RefCell<dyn Entity>>> {
        for i in 0..ONION_LAYERS {
            self.cooked[i] += dt * self.heat * [1.0, 0.6, 0.3, 0.1][i];
        }
        if rand::random::<f64>() < dt * self.heat * 20.0 {
            let bounds = self.bounds().as_floats();
            vec![Rc::new(RefCell::new(Smoke::new([
                bounds[0] + rand::random::<f64>() * bounds[2],
                bounds[1] + rand::random::<f64>() * bounds[3],
            ], 0.4 * (3.8 - 3.0 * self.cooked[0] as f32).clamp(0.0, 1.0))))]
        } else {
            vec![]
        }
    }

    fn drag(&mut self, from: [f64; 2], to: [f64; 2]) {
        let mut bounds = self.bounds().as_floats();
        for i in 0..2 {
            self.pos[i] += to[i] - from[i];
            bounds[i] += to[i] - from[i];
        }
        self.bounds = Rectangle::new([bounds[0], bounds[1]], [bounds[2], bounds[3]]);
    }

    fn set_pos(&mut self, pos: [f64; 2]) {
        self.pos = pos;
    }

    fn drop(&mut self) {
        self.scramble();
        self.heat = 0.0;
    }

    fn set_heat(&mut self, heat: f64) {
        self.heat = heat;
    }

    fn topping(&self) -> Option<Topping> {
        Some(Topping::Onion)
    }

    fn add_to(&mut self, pos: [f64; 2], others: &[Rc<RefCell<dyn Entity>>]) -> bool {
        if others.iter()
                 .filter(|e| e.borrow().topping().contains(&Topping::Onion))
                 .next()
                 .is_none() {
            self.set_pos(pos);
            true
        } else {
            false
        }
    }

    fn draw(&self, context: Context, graphics: &mut G2d) {
        for (layer, cooked) in self.layers.iter().zip(&self.cooked) {
            let colour = interpolate_colour(&[(RAW_ONION, 0.0), (COOKED_ONION, 1.0), (BLACK, 1.4)], *cooked as f32);
            for piece in layer {
                piece.draw(self.pos, colour, context, graphics);
            }
        }
    }
}

impl OnionPiece {
    fn new() -> OnionPiece {
        let start = rand::random::<f64>() * std::f64::consts::PI * 2.0;
        let end = start + (0.4 + 0.6 * rand::random::<f64>()) * std::f64::consts::PI;
        let r = 5.0 + 20.0 * rand::random::<f64>();
        let x = 40.0 * rand::random::<f64>() - 20.0 - (1.0 + (end.sin() - start.sin()) / (end - start)) * r;
        let y = 40.0 * rand::random::<f64>() - 20.0 - (1.0 + (start.cos() - end.cos()) / (end - start)) * r;
        OnionPiece{
            rect: [x, y, 2.0 * r, 2.0* r],
            start,
            end,
            thickness: 2.0 + rand::random::<f64>() * 2.0,
        }
    }

    fn scramble(&mut self) {
        let len = self.end - self.start;
        self.start = rand::random::<f64>() * std::f64::consts::PI * 2.0;
        self.end = self.start + len;
        let r = self.rect[2] / 2.0;
        self.rect[0] = 40.0 * rand::random::<f64>() - 20.0 - (1.0 + (self.end.sin() - self.start.sin()) / (self.end - self.start)) * r;
        self.rect[1] = 40.0 * rand::random::<f64>() - 20.0 - (1.0 + (self.start.cos() - self.end.cos()) / (self.end - self.start)) * r;
    }

    fn bounds(&self, pos: [f64; 2]) -> Rectangle {
        let r = self.rect[2] / 2.0;
        let sc = self.start.cos();
        let ec = self.end.cos();
        let xmin = if self.end - std::f64::consts::PI - (self.start - std::f64::consts::PI).div_euclid(2.0 * std::f64::consts::PI) * 2.0 * std::f64::consts::PI > 2.0 * std::f64::consts::PI {
            -1.0
        } else {
            sc.min(ec)
        } * r;
        let xmax = if self.end - self.start.div_euclid(2.0 * std::f64::consts::PI) * 2.0 * std::f64::consts::PI > 2.0 * std::f64::consts::PI {
            1.0
        } else {
            sc.max(ec)
        } * r;
        let ss = self.start.sin();
        let es = self.end.sin();
        let ymin = if self.end - 3.0 / 2.0 * std::f64::consts::PI - (self.start - 3.0 / 2.0 * std::f64::consts::PI).div_euclid(2.0 * std::f64::consts::PI) * 2.0 * std::f64::consts::PI > 2.0 * std::f64::consts::PI {
            -1.0
        } else {
            ss.min(es)
        } * r;
        let ymax = if self.end - 1.0 / 2.0 * std::f64::consts::PI - (self.start - 1.0 / 2.0 * std::f64::consts::PI).div_euclid(2.0 * std::f64::consts::PI) * 2.0 * std::f64::consts::PI > 2.0 * std::f64::consts::PI {
            1.0
        } else {
            ss.max(es)
        } * r;
        Rectangle::new([pos[0] + self.rect[0] + self.rect[2] / 2.0 + xmin, pos[1] + self.rect[1] + self.rect[3] / 2.0 + ymin], [xmax - xmin, ymax - ymin])
    }

    fn draw(&self, pos: [f64; 2], colour: [f32; 4], context: Context, graphics: &mut G2d) {
        piston_window::circle_arc(
            colour,
            self.thickness,
            self.start,
            self.end,
            [self.rect[0] + pos[0], self.rect[1] + pos[1], self.rect[2], self.rect[3]],
            context.transform,
            graphics,
        )
    }
}
