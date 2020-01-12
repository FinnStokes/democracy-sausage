#![feature(drain_filter, clamp, option_result_contains)]

use piston_window::*;

use std::cell::RefCell;
use std::rc::Rc;

mod colour;
mod entity;
mod geometry;
mod scene;

use entity::Entity;
use scene::Scene;

fn main() {
    let mut window: PistonWindow =
        WindowSettings::new("Sizzle!", [640, 480])
        .exit_on_esc(true).build().unwrap();

    let mut selected: Option<Rc<RefCell<dyn Entity>>> = None;
    let mut last_pos: Option<[f64; 2]> = None;

    let mut scene = Scene::new();

    while let Some(e) = window.next() {
        window.draw_2d(&e, |context, graphics, _device| {
            clear([90.0 / 255.0, 202.0 / 255.0, 77.0 / 255.0, 1.0], graphics);
            scene.draw(context, graphics);
        });
        
        if let Some(args) = e.update_args() {
            scene.update(args.dt);

            if let Some(ref mut selected) = selected {
                selected.borrow_mut().update_selected(args.dt);
            }
        }

        if let Some(button) = e.press_args() {
            if let Some(pos) = last_pos {
                if button == Button::Mouse(MouseButton::Left) {
                    selected = scene.select(pos);
                    if let Some(ref mut selected) = selected {
                        selected.borrow_mut().grab();
                        scene.grabbed(selected);
                    }
                }
            }
        }

        if let Some(button) = e.release_args() {
            if button == Button::Mouse(MouseButton::Left) {
                if let Some(ref mut selected) = selected {
                    selected.borrow_mut().drop();
                    scene.dropped(selected);
                }
                selected = None;
            }
        }

        if let Some(ref mut selected) = selected {
            if let Some(pos) = e.mouse_cursor_args() {
                if let Some(last_pos) = last_pos {
                    selected.borrow_mut().drag(last_pos, pos);
                }
            }
        }

        if let Some(pos) = e.mouse_cursor_args() {
            last_pos = Some(pos);
        }
    }
}
