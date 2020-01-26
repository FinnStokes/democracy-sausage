use std::rc::Rc;
use std::cell::RefCell;

use crate::geometry::Rectangle;
use crate::colour::interpolate_colour;

use piston_window::{context::Context,Graphics};
use noise::{Seedable, NoiseFn};
use rand::{Rng,distributions::Bernoulli};
use rand_distr::Beta;

pub enum Selection<G: Graphics> {
    None,
    This,
    New(Rc<RefCell<dyn Entity<G>>>),
}

pub trait Entity<G: Graphics> {
    fn bounds(&self) -> Rectangle;
    fn select(&mut self, _pos: [f64; 2]) -> Selection<G> { Selection::None }
    fn update(&mut self, _dt: f64) -> Vec<Rc<RefCell<dyn Entity<G>>>> { vec![] }
    fn update_selected(&mut self, _dt: f64) {}
    fn grab(&mut self) {}
    fn drop(&mut self) {}
    fn drag(&mut self, _from: [f64; 2], _to: [f64; 2]) {}
    fn draw(&self, context: Context, graphics: &mut G);
    fn set_pos(&mut self, _pos: [f64; 2]) {}
    fn topping(&self) -> Option<Topping> { None }
    fn add_topping(&mut self, _topping: &Rc<RefCell<dyn Entity<G>>>) -> Selection<G> { Selection::None }
    fn add_to(&mut self, _pos: [f64; 2], _others: &[Rc<RefCell<dyn Entity<G>>>]) -> Selection<G> { Selection::None }
    fn set_heat(&mut self, _heat: f64) {}
    fn heat(&self, _pos: [f64; 2]) -> f64 { 0.0 }
    fn cooked(&self) -> [f64; 2] { [0.0, 0.0] }
    fn expired(&self) -> bool { false }
    fn order(&self) -> Option<&Bread<G>> { None }
    fn deliver_order(&mut self, _order: &Bread<G>) -> Option<Mood> { None }
}

const SAUSAGE_SIZE: [f64; 2] = [13.0, 65.0];
const PATTY_SIZE: [f64; 2] = [50.0, 36.0];
const PLATE_SIZE: [f64; 2] = [78.0, 78.0];
const SAUSAGE_OFFSET: f64 = 10.0;
const BREAD_SIZE: [f64; 2] = [53.0, 53.0];
const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const HAPPY: [f32; 4] = [0.3, 1.0, 0.4, 1.0];
const NEUTRAL: [f32; 4] = [0.9, 0.9, 0.1, 1.0];
const SAD: [f32; 4] = [1.0, 0.1, 0.1, 1.0];
const LIGHT_GREY: [f32; 4] = [0.95, 0.95, 0.95, 1.0];
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
const CUSTOMERS_PER_SECOND: f64 = 0.1;
const ORDER_OFFSET: [f64; 2] = [0.0, 90.0];
const QUEUE_SPACING: f64 = 130.0;
const QUEUE_SPEED: f64 = 100.0;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Topping {
    Filling(Filling),
    Onion,
    Condiment(Condiment),
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Filling {
    Sausage,
    VeggiePatty,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Condiment {
    Sauce,
    Mustard,
}

impl Condiment {
    fn colour(&self) -> [f32; 4] {
        match self {
            Condiment::Sauce => [0.95, 0.1, 0.0, 1.0],
            Condiment::Mustard => [0.9, 0.85, 0.0, 1.0],
        }
    }
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
        Cookable::with_cooked(kind, pos, 0.0)
    }

    pub fn with_cooked(kind: Filling, pos: [f64; 2], cooked:f64) -> Cookable {
        Cookable{
            pos,
            heat: 0.0,
            top_cooked: cooked,
            bottom_cooked: cooked,
            flipped: false,
            kind,
        }
    }
}

impl<G: Graphics> Entity<G> for Cookable {
    fn bounds(&self) -> Rectangle {
        match self.kind {
            Filling::Sausage => Rectangle::centered(self.pos, SAUSAGE_SIZE),
            Filling::VeggiePatty => Rectangle::centered(self.pos, PATTY_SIZE),
        }
    }

    fn select(&mut self, pos: [f64; 2]) -> Selection<G> {
        if Entity::<G>::bounds(self).intersect_point(pos) {
            Selection::This
        } else {
            Selection::None
        }
    }

    fn update(&mut self, dt: f64) -> Vec<Rc<RefCell<dyn Entity<G>>>> {
        self.bottom_cooked += dt * self.heat * match self.kind {
            Filling::Sausage => 1.0,
            Filling::VeggiePatty => 0.5,
        };
        if rand::random::<f64>() < dt * self.heat * 20.0 {
            let bounds = Entity::<G>::bounds(self).as_floats();
            vec![Rc::new(RefCell::new(Smoke::new([
                bounds[0] + rand::random::<f64>() * bounds[2],
                bounds[1] + rand::random::<f64>() * bounds[3],
            ], 0.4 * (3.8 - 3.0 * self.bottom_cooked as f32).max(0.0).min(1.0))))]
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

    fn cooked(&self) -> [f64; 2] {
        [
            self.top_cooked.min(self.bottom_cooked),
            self.top_cooked.max(self.bottom_cooked),
        ]
    }

    fn set_heat(&mut self, heat: f64) {
        self.heat = heat;
    }

    fn draw(&self, context: Context, graphics: &mut G) {
        match self.kind {
            Filling::Sausage => {
                let color = interpolate_colour(&[(PINK, 0.0), (BROWN, 1.0), (BLACK, 1.4)], self.top_cooked as f32);
                piston_window::rectangle(color,
                                         Entity::<G>::bounds(self).as_floats(),
                                         context.transform,
                                         graphics);
            },
            Filling::VeggiePatty => {
                let color = interpolate_colour(&[(YELLOW, 0.0), (ORANGE, 1.0), (BLACK, 1.4)], self.top_cooked as f32);
                let bounds = Entity::<G>::bounds(self).as_floats();
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

    fn add_to(&mut self, pos: [f64; 2], others: &[Rc<RefCell<dyn Entity<G>>>]) -> Selection<G> {
        let mut other_fillings = others.iter()
            .filter(|e| if let Some(Topping::Filling(_)) = e.borrow().topping() {
                true
            } else {
                false
            });
        match other_fillings.next() {
            None => {
                Entity::<G>::set_pos(self, pos);
                Selection::This
            },
            Some(f) => if Some(Topping::Filling(Filling::Sausage)) == f.borrow().topping() {
                if self.kind == Filling::Sausage {
                    if let None = other_fillings.next() {
                        self.pos = [pos[0] + SAUSAGE_OFFSET, pos[1]];
                        f.borrow_mut().set_pos([pos[0] - SAUSAGE_OFFSET, pos[1]]);
                        Selection::This
                    } else {
                        Selection::None
                    }
                } else {
                    Selection::None
                }
            } else {
                Selection::None
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

impl<G: Graphics> Entity<G> for Hotplate {
    fn bounds(&self) -> Rectangle {
        self.bounds
    }

    fn draw(&self, context: Context, graphics: &mut G) {
        piston_window::rectangle([0.2, 0.15, 0.25, 1.0],
                                 Entity::<G>::bounds(self).as_floats(),
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
            let bounds = Entity::<G>::bounds(self).as_floats();
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

fn rounded_rectangle<G: Graphics>(colour: [f32; 4], bounds: [f64; 4], r: f64, transform: [[f64; 3]; 2], graphics: &mut G) {
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

impl<G: Graphics> Entity<G> for Table {
    fn bounds(&self) -> Rectangle {
        self.bounds
    }

    fn draw(&self, context: Context, graphics: &mut G) {
        rounded_rectangle([0.95, 1.0, 1.0, 1.0],
                          Entity::<G>::bounds(self).as_floats(),
                          25.0,
                          context.transform,
                          graphics);
    }
}

pub struct Bread<G: Graphics> {
    pos: [f64; 2],
    toppings: Vec<Rc<RefCell<dyn Entity<G>>>>,
}

impl<G: Graphics> Bread<G> {
    pub fn new(pos: [f64; 2]) -> Bread<G> {
        Bread{
            pos,
            toppings: Vec::new(),
        }
    }
}

impl<G: Graphics> Clone for Bread<G> {
    fn clone(&self) -> Bread<G> {
        Bread{
            pos: self.pos,
            toppings: self.toppings.clone(),
        }
    }
}

impl<G: Graphics> Entity<G> for Bread<G> {
    fn bounds(&self) -> Rectangle {
        Rectangle::centered(self.pos, BREAD_SIZE)
    }

    fn select(&mut self, pos: [f64; 2]) -> Selection<G> {
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

    fn draw(&self, context: Context, graphics: &mut G) {
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

    fn add_topping(&mut self, topping: &Rc<RefCell<dyn Entity<G>>>) -> Selection<G> {
        let res = topping.borrow_mut().add_to(self.pos, &self.toppings);
        match &res {
            Selection::This => {
                self.toppings.push(topping.clone());
            },
            Selection::New(new) => {
                self.toppings.push(new.clone());
            },
            _ => {},
        }
        res
    }

    fn order(&self) -> Option<&Bread<G>> {
        Some(self)
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

impl<G: Graphics> Entity<G> for Smoke {
    fn bounds(&self) -> Rectangle {
        let r = 10.0 + self.age * 5.0;
        Rectangle::centered(self.pos, [2.0 * r, 2.0 * r])
    }

    fn update(&mut self, dt: f64) -> Vec<Rc<RefCell<dyn Entity<G>>>> {
        self.age += dt;
        self.pos = [self.pos[0] + dt * 20.0, self.pos[1] + dt * 10.0];
        vec![]
    }

    fn expired(&self) -> bool {
        self.age > 5.0
    }

    fn draw(&self, context: Context, graphics: &mut G) {
        piston_window::ellipse([self.colour, self.colour, self.colour, 0.5 * (1.0 - (self.age as f32 / 5.0)).max(0.0)],
                               Entity::<G>::bounds(self).as_floats(),
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

fn knife<G: Graphics>(centre: [f64; 2], transform: [[f64; 3]; 2], graphics: &mut G) {
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

impl<G: Graphics> Entity<G> for ChoppingBoard {
    fn bounds(&self) -> Rectangle {
        Rectangle::centered(self.pos, [90.0, 120.0])
    }

    fn select(&mut self, pos: [f64; 2]) -> Selection<G> {
        if Entity::<G>::bounds(self).intersect_point(pos) {
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

    fn draw(&self, context: Context, graphics: &mut G) {
        rounded_rectangle(BOARD,
                          Entity::<G>::bounds(self).as_floats(),
                          2.0,
                          context.transform,
                          graphics);
        let centre = Entity::<G>::bounds(self).centre();
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
        Onion::with_cooked(pos, 0.0)
    }

    pub fn with_cooked(pos: [f64; 2], cooked: f64) -> Onion {
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
            cooked: [cooked; ONION_LAYERS],
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

impl<G: Graphics> Entity<G> for Onion {
    fn bounds(&self) -> Rectangle {
        self.bounds
    }

    fn select(&mut self, pos: [f64; 2]) -> Selection<G> {
        if Entity::<G>::bounds(self).intersect_point(pos) {
            Selection::This
        } else {
            Selection::None
        }
    }

    fn update(&mut self, dt: f64) -> Vec<Rc<RefCell<dyn Entity<G>>>> {
        for i in 0..ONION_LAYERS {
            self.cooked[i] += dt * self.heat * [1.0, 0.6, 0.3, 0.1][i];
        }
        if rand::random::<f64>() < dt * self.heat * 20.0 {
            let bounds = Entity::<G>::bounds(self).as_floats();
            vec![Rc::new(RefCell::new(Smoke::new([
                bounds[0] + rand::random::<f64>() * bounds[2],
                bounds[1] + rand::random::<f64>() * bounds[3],
            ], 0.4 * (3.8 - 3.0 * self.cooked[0] as f32).max(0.0).min(1.0))))]
        } else {
            vec![]
        }
    }

    fn cooked(&self) -> [f64; 2] {
        [
            self.cooked.iter().cloned().fold(0./0., f64::min),
            self.cooked.iter().cloned().fold(0./0., f64::max),
        ]
    }

    fn drag(&mut self, from: [f64; 2], to: [f64; 2]) {
        let mut bounds = Entity::<G>::bounds(self).as_floats();
        for i in 0..2 {
            self.pos[i] += to[i] - from[i];
            bounds[i] += to[i] - from[i];
        }
        self.bounds = Rectangle::new([bounds[0], bounds[1]], [bounds[2], bounds[3]]);
    }

    fn set_pos(&mut self, pos: [f64; 2]) {
        Entity::<G>::drag(self, self.pos, pos);
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

    fn add_to(&mut self, pos: [f64; 2], others: &[Rc<RefCell<dyn Entity<G>>>]) -> Selection<G> {
        if others.iter()
                 .filter(|e| e.borrow().topping() == Some(Topping::Onion))
                 .next()
                 .is_none() {
            Entity::<G>::set_pos(self, pos);
            Selection::This
        } else {
            Selection::None
        }
    }

    fn draw(&self, context: Context, graphics: &mut G) {
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
        let r = 5.0 + 15.0 * rand::random::<f64>();
        let x = 30.0 * rand::random::<f64>() - 15.0 - (1.0 + (end.sin() - start.sin()) / (end - start)) * r;
        let y = 30.0 * rand::random::<f64>() - 15.0 - (1.0 + (start.cos() - end.cos()) / (end - start)) * r;
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

    fn draw<G: Graphics>(&self, pos: [f64; 2], colour: [f32; 4], context: Context, graphics: &mut G) {
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

pub struct Squirt {
    pos: [f64; 2],
    blobs: Vec<Blob>,
    bounds: Rectangle,
    condiment: Condiment,
}

pub struct Blob {
    offset: [f64; 2],
    radius: f64,
}

impl Squirt {
    pub fn new(condiment: Condiment, pos: [f64; 2]) -> Squirt {
        let scale = BREAD_SIZE[0] / 2.0;
        let mut rng = rand::thread_rng();
        let n_blob = rng.gen_range(4, 6);
        let mut blobs = Vec::with_capacity(n_blob);
        let offset = match condiment {
            Condiment::Sauce => [-0.2, -0.2],
            Condiment::Mustard => [0.2, 0.2],
        };
        blobs.push(Blob{offset: [offset[0] * scale, offset[1] * scale], radius: 0.2 * scale});
        for _ in 1..n_blob {
            blobs.push(Blob{
                offset: [(offset[0] - 0.2 + 0.4 * rng.gen::<f64>()) * scale,
                         (offset[1] - 0.2 + 0.4 * rng.gen::<f64>()) * scale],
                radius: (0.2 + 0.2 * rng.gen::<f64>()) * scale,
            });
        }
        let mut it = blobs.iter().map(|p| p.bounds(pos));
        let first = it.next();
        let bounds = it.fold(first.unwrap(), |r, s| r.union(&s));
        Squirt{
            pos,
            blobs,
            bounds,
            condiment,
        }
    }
}

impl<G: Graphics> Entity<G> for Squirt {
    fn bounds(&self) -> Rectangle {
        self.bounds
    }

    fn select(&mut self, pos: [f64; 2]) -> Selection<G> {
        if Entity::<G>::bounds(self).intersect_point(pos) {
            Selection::This
        } else {
            Selection::None
        }
    }

    fn drag(&mut self, from: [f64; 2], to: [f64; 2]) {
        let mut bounds = Entity::<G>::bounds(self).as_floats();
        for i in 0..2 {
            self.pos[i] += to[i] - from[i];
            bounds[i] += to[i] - from[i];
        }
        self.bounds = Rectangle::new([bounds[0], bounds[1]], [bounds[2], bounds[3]]);
    }

    fn set_pos(&mut self, pos: [f64; 2]) {
        Entity::<G>::drag(self, self.pos, pos);
    }

    fn topping(&self) -> Option<Topping> {
        Some(Topping::Condiment(self.condiment))
    }

    fn add_to(&mut self, pos: [f64; 2], others: &[Rc<RefCell<dyn Entity<G>>>]) -> Selection<G> {
        if others.iter()
                 .filter(|e| e.borrow().topping() == Some(Topping::Condiment(self.condiment)))
                 .next()
                 .is_none() {
            Entity::<G>::set_pos(self, pos);
            Selection::This
        } else {
            Selection::None
        }
    }

    fn draw(&self, context: Context, graphics: &mut G) {
        for blob in &self.blobs {
            piston_window::ellipse(self.condiment.colour(),
                                   blob.bounds(self.pos).as_floats(),
                                   context.transform,
                                   graphics);
        }
    }
}

impl Blob {
    fn bounds(&self, pos: [f64; 2]) -> Rectangle {
        Rectangle::centered([pos[0] + self.offset[0], pos[1] + self.offset[1]], [self.radius * 2.0, self.radius * 2.0])
    }
}

pub struct Bottle {
    pos: [f64; 2],
    condiment: Condiment,
}

impl Bottle {
    pub fn new(condiment: Condiment, pos: [f64; 2]) -> Bottle {
        Bottle{pos, condiment}
    }
}

impl<G: Graphics> Entity<G> for Bottle {
    fn bounds(&self) -> Rectangle {
        Rectangle::centered(self.pos, [20.0, 80.0])
    }

    fn select(&mut self, pos: [f64; 2]) -> Selection<G> {
        if Entity::<G>::bounds(self).intersect_point(pos) {
            Selection::This
        } else {
            Selection::None
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

    fn draw(&self, context: Context, graphics: &mut G) {
        piston_window::rectangle(self.condiment.colour(),
                                 [self.pos[0] - 10.0, self.pos[1] - 25.0, 20.0, 65.0],
                                 context.transform,
                                 graphics);
        piston_window::polygon(self.condiment.colour(),
                               &[[self.pos[0] - 8.0, self.pos[1] - 25.0],
                                 [self.pos[0] - 1.0, self.pos[1] - 40.0],
                                 [self.pos[0] + 1.0, self.pos[1] - 40.0],
                                 [self.pos[0] + 8.0, self.pos[1] - 25.0]],
                               context.transform,
                               graphics);
    }

    fn topping(&self) -> Option<Topping> {
        Some(Topping::Condiment(self.condiment))
    }

    fn add_to(&mut self, pos: [f64; 2], others: &[Rc<RefCell<dyn Entity<G>>>]) -> Selection<G> {
        if others.iter()
                 .filter(|e| e.borrow().topping().contains(&Topping::Condiment(self.condiment)))
                 .next()
                 .is_none() {
            Selection::New(Rc::new(RefCell::new(Squirt::new(self.condiment, pos))))
        } else {
            Selection::None
        }
    }
}

#[derive(Clone, Copy)]
pub enum Mood {
    Happy,
    Neutral,
    Sad,
    Sick,
}

pub struct Customer<G: Graphics> {
    pos: [f64; 2],
    order: Bread<G>,
    meal: Option<Bread<G>>,
    mood: Option<Mood>,
}

impl<G: Graphics> Customer<G> {
    fn new(pos: [f64; 2]) -> Customer<G> {
        let mut order = Bread{
            pos: [pos[0] + ORDER_OFFSET[0], pos[1] + ORDER_OFFSET[1]],
            toppings: Vec::with_capacity(5),
        };
        let mut rng = rand::thread_rng();
        
        // Filling cooked between 0.8 and 1.2 with peak at 1.0
        let filling_cooked: f64 = 0.8 + 0.4 * rng.sample(Beta::new(2.0, 2.0).unwrap());
        // Onion cooked between 0.8 and 1.2 with peak at 1.0
        let onion_cooked: f64 = 0.8 + 0.4 * rng.sample(Beta::new(2.0, 2.0).unwrap());
        
        let filling: f64 = rng.gen();
        if filling < 0.5 {
            // 50% chance of one sausage
            order.add_topping(&(Rc::new(RefCell::new(Cookable::with_cooked(
                Filling::Sausage,
                pos,
                filling_cooked,
            ))) as Rc<RefCell<dyn Entity<G>>>));
        } else if filling < 0.75 {
            // 25% chance of two sausages
            order.add_topping(&(Rc::new(RefCell::new(Cookable::with_cooked(
                Filling::Sausage,
                pos,
                filling_cooked,
            ))) as Rc<RefCell<dyn Entity<G>>>));
            order.add_topping(&(Rc::new(RefCell::new(Cookable::with_cooked(
                Filling::Sausage,
                pos,
                filling_cooked,
            ))) as Rc<RefCell<dyn Entity<G>>>));
        } else {
            // 25% chance of patty
            order.add_topping(&(Rc::new(RefCell::new(Cookable::with_cooked(
                Filling::VeggiePatty,
                pos,
                filling_cooked,
            ))) as Rc<RefCell<dyn Entity<G>>>));
        }

        // 40% chance the customer wants onion
        if rng.sample(Bernoulli::new(0.4).unwrap()) {
            order.add_topping(&(Rc::new(RefCell::new(Onion::with_cooked(
                pos,
                onion_cooked,
            ))) as Rc<RefCell<dyn Entity<G>>>));
        }

        let condiment: f64 = rng.gen();
        if condiment < 0.5 {
            // 50% chance of tomato sauce
            order.add_topping(&(Rc::new(RefCell::new(Squirt::new(
                Condiment::Sauce,
                pos,
            ))) as Rc<RefCell<dyn Entity<G>>>));
        } else if condiment < 0.7 {
            // 20% chance of mustard
            order.add_topping(&(Rc::new(RefCell::new(Squirt::new(
                Condiment::Mustard,
                pos,
            ))) as Rc<RefCell<dyn Entity<G>>>));
        } else if condiment < 0.9 {
            // 20% chance of tomato sauce and mustard
            order.add_topping(&(Rc::new(RefCell::new(Squirt::new(
                Condiment::Sauce,
                pos,
            ))) as Rc<RefCell<dyn Entity<G>>>));
            order.add_topping(&(Rc::new(RefCell::new(Squirt::new(
                Condiment::Mustard,
                pos,
            ))) as Rc<RefCell<dyn Entity<G>>>));
        } else {
            // 10% chance of no condiment
        }

        Customer{
            pos,
            order,
            meal: None,
            mood: None,
        }
    }
}

impl<G: Graphics> Entity<G> for Customer<G> {
    fn bounds(&self) -> Rectangle {
        Rectangle::centered(self.pos, PLATE_SIZE)
    }

    fn set_pos(&mut self, pos: [f64; 2]) {
        self.pos = pos;
        self.order.set_pos([pos[0] + ORDER_OFFSET[0], pos[1] + ORDER_OFFSET[1]]);
        if let Some(meal) = &mut self.meal {
            meal.set_pos(self.pos);
        }
    }

    fn draw(&self, context: Context, graphics: &mut G) {
        piston_window::ellipse(WHITE,
                               self.bounds().as_floats(),
                               context.transform,
                               graphics);
        piston_window::ellipse(LIGHT_GREY,
                               Rectangle::centered(self.pos, [PLATE_SIZE[0] * 0.65, PLATE_SIZE[1] * 0.65]).as_floats(),
                               context.transform,
                               graphics);

        let thought_colour = match self.mood {
            None => WHITE,
            Some(Mood::Happy) => HAPPY,
            Some(Mood::Neutral) => NEUTRAL,
            Some(Mood::Sad) => SAD,
            Some(Mood::Sick) => SAD,
        };

        piston_window::ellipse(thought_colour,
                               [self.pos[0] + ORDER_OFFSET[0] - 54.0, self.pos[1] + ORDER_OFFSET[1] - 30.0, 60.0, 50.0],
                               context.transform,
                               graphics);
        piston_window::ellipse(thought_colour,
                               [self.pos[0] + ORDER_OFFSET[0] - 44.0, self.pos[1] + ORDER_OFFSET[1] - 10.0, 55.0, 55.0],
                               context.transform,
                               graphics);
        piston_window::ellipse(thought_colour,
                               [self.pos[0] + ORDER_OFFSET[0] - 34.0, self.pos[1] + ORDER_OFFSET[1] - 45.0, 70.0, 52.0],
                               context.transform,
                               graphics);
        piston_window::ellipse(thought_colour,
                               [self.pos[0] + ORDER_OFFSET[0] - 4.0, self.pos[1] + ORDER_OFFSET[1] - 40.0, 50.0, 45.0],
                               context.transform,
                               graphics);
        piston_window::ellipse(thought_colour,
                               [self.pos[0] + ORDER_OFFSET[0] - 14.0, self.pos[1] + ORDER_OFFSET[1] - 15.0, 60.0, 55.0],
                               context.transform,
                               graphics);

        piston_window::ellipse(thought_colour,
                               [self.pos[0] + ORDER_OFFSET[0] - 50.0, self.pos[1] + ORDER_OFFSET[1] - 53.0, 14.0, 14.0],
                               context.transform,
                               graphics);
        piston_window::ellipse(thought_colour,
                               [self.pos[0] + ORDER_OFFSET[0] - 65.0, self.pos[1] + ORDER_OFFSET[1] - 74.0, 12.0, 12.0],
                               context.transform,
                               graphics);
        piston_window::ellipse(thought_colour,
                               [self.pos[0] + ORDER_OFFSET[0] - 67.0, self.pos[1] + ORDER_OFFSET[1] - 100.0, 10.0, 11.0],
                               context.transform,
                               graphics);
        piston_window::ellipse(thought_colour,
                               [self.pos[0] + ORDER_OFFSET[0] - 60.0, self.pos[1] + ORDER_OFFSET[1] - 126.0, 9.0, 10.0],
                               context.transform,
                               graphics);

        self.order.draw(context, graphics);
        if let Some(meal) = &self.meal {
            meal.draw(context, graphics);
        }
    }
// pub enum Topping {
//     Filling(Filling),
//     Onion,
//     Condiment(Condiment),
// }

    fn deliver_order(&mut self, order: &Bread<G>) -> Option<Mood> {
        if self.mood.is_none() && order.bounds().intersect_rect(&self.bounds()) {
            let mut toppings = order.toppings.clone();
            let mut score: f64 = 0.0;
            let mut has_filling = false;
            let mut sick = false;
            let mut missing: u32 = 0;
            let mut wrong: u32 = 0;
            let mut burnt: u32 = 0;
            for topping in &self.order.toppings {
                if let Some(i) = toppings.iter().position(|other|
                    match (topping.borrow().topping(), other.borrow().topping()) {
                        (Some(Topping::Filling(_)), Some(Topping::Filling(_))) => true,
                        (Some(Topping::Onion), Some(Topping::Onion)) => true,
                        (Some(Topping::Condiment(c1)), Some(Topping::Condiment(c2))) => c1 == c2,
                        _ => false,
                    }
                ) {
                    let other = toppings.remove(i);
                    match topping.borrow().topping() {
                        Some(Topping::Filling(filling)) => {
                            if let Some(Topping::Filling(other_filling)) = other.borrow().topping() {
                                if filling == other_filling {
                                    has_filling = true;
                                    if filling == Filling::Sausage {
                                        if other.borrow().cooked()[0] < 0.7 {
                                            sick = true;
                                        }
                                    }
                                    if other.borrow().cooked()[1] > 1.4 {
                                        burnt += 1
                                    }
                                    if other.borrow().cooked()[0] > 1.4 {
                                        burnt += 1
                                    }
                                    score += 1.0 - (other.borrow().cooked()[0] - topping.borrow().cooked()[0]).powi(2).min(0.04) * 25.0
                                                 - (other.borrow().cooked()[1] - topping.borrow().cooked()[1]).powi(2).min(0.04) * 25.0;
                                } else {
                                    if filling == Filling::VeggiePatty {
                                        sick = true;
                                    } else {
                                        missing += 1;
                                        wrong += 1;
                                    }
                                }
                            } else {
                                panic!();
                            }
                        },
                        Some(Topping::Onion) => {
                            if other.borrow().cooked()[1] > 1.4 {
                                burnt += 1
                            }
                            if other.borrow().cooked()[0] > 1.4 {
                                burnt += 1
                            }
                            score += 1.0 - (other.borrow().cooked()[0] - topping.borrow().cooked()[0]).powi(2).min(0.04) * 25.0
                                         - (other.borrow().cooked()[1] - topping.borrow().cooked()[1]).powi(2).min(0.04) * 25.0;
                        },
                        _ => {},
                    }
                } else {
                    missing += 1;
                    score -= 1.0;
                }
            }
            for incorrect in toppings {
                wrong += 1;
                if let Some(Topping::Filling(Filling::Sausage)) = incorrect.borrow().topping() {
                    if incorrect.borrow().cooked()[0] < 0.7 {
                        sick = true;
                    }
                }
                if incorrect.borrow().cooked()[1] > 1.4 {
                    burnt += 1
                }
                if incorrect.borrow().cooked()[0] > 1.4 {
                    burnt += 1
                }
            }
            let mut meal: Bread<G> = (*order).clone();
            meal.set_pos(self.pos);
            self.meal = Some(meal);
            let mood = if sick {
                Some(Mood::Sick)
            } else if has_filling {
                let score = score / (self.order.toppings.len() as f64);
                if score < -0.5 || missing + wrong + burnt > 3 {
                    Some(Mood::Sad)
                } else if score < 0.1 || missing + wrong + burnt > 0 {
                    Some(Mood::Neutral)
                } else {
                    Some(Mood::Happy)
                }
            } else {
                Some(Mood::Sad)
            };
            self.mood = mood;
            mood
        } else {
            None
        }
    }
}

pub struct Queue<G: Graphics> {
    head: [f64; 2],
    entry: [f64; 2],
    max_len: usize,
    customers: Vec<Customer<G>>,
}

impl<G: Graphics> Queue<G> {
    pub fn new(head: [f64; 2], entry: [f64; 2], max_len: usize) -> Queue<G> {
        Queue{
            head, entry, max_len,
            customers: Vec::with_capacity(max_len),
        }
    }
}

impl<G: Graphics> Entity<G> for Queue<G> {
    fn bounds(&self) -> Rectangle {
        Rectangle::new([self.head[0] - 50.0, self.head[1] - 50.0], [100.0 + (self.entry[0] - self.head[0]), 100.0 + (self.entry[1] - self.head[1])])
    }

    fn draw(&self, context: Context, graphics: &mut G) {
        for customer in &self.customers {
            customer.draw(context, graphics);
        }
    }

    fn update(&mut self, dt: f64) -> Vec<Rc<RefCell<dyn Entity<G>>>> {
        if self.customers.len() < self.max_len && rand::random::<f64>() < dt * CUSTOMERS_PER_SECOND {
            self.customers.push(Customer::new(self.entry));
        }

        let del = [self.head[0] - self.entry[0], self.head[1] - self.entry[1]];
        let norm = (del[0] * del[0] + del[1] * del[1]).sqrt();
        let del = [del[0] / norm, del[1] / norm];
        let head = self.head;
        self.customers.iter_mut().fold(None as Option<&Customer<G>>, |prev, customer| {
            if customer.mood.is_none() {
                let target = if let Some(prev) = prev {
                    [prev.pos[0] - del[0] * QUEUE_SPACING, prev.pos[1] - del[1] * QUEUE_SPACING]
                } else {
                    head
                };
                let maxdel = [target[0] - customer.pos[0], target[1] - customer.pos[1]];
                if maxdel[0] / del[0] > 0.0 {
                    if maxdel[0] / del[0] > QUEUE_SPEED * dt {
                        customer.set_pos([
                            customer.pos[0] + del[0] * QUEUE_SPEED * dt,
                            customer.pos[1] + del[1] * QUEUE_SPEED * dt,
                        ]);
                    } else {
                        customer.set_pos(target);
                    }
                }
                Some(customer)
            } else {
                customer.set_pos([
                    customer.pos[0],
                    customer.pos[1] - QUEUE_SPEED * dt * 2.0,
                ]);
                if customer.pos[1] > head[1] - 60.0 {
                    Some(customer)
                } else {
                    prev
                }
            }
        });
        self.customers.retain(|c| c.mood.is_none() || c.pos[1] > head[1] - 200.0);
    
        vec![]
    }

    fn deliver_order(&mut self, order: &Bread<G>) -> Option<Mood> {
        for customer in &mut self.customers {
            if let Some(mood) = customer.deliver_order(order) {
                return Some(mood);
            }
        }
        None
    }
}
