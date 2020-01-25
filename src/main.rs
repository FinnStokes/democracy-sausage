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
        .exit_on_esc(true)
        .vsync(true)
        .fullscreen(true)
        .build().unwrap();

    let mut selected: Option<Rc<RefCell<dyn Entity>>> = None;
    let mut last_pos: Option<[f64; 2]> = None;
    let mut transform: Option<[[f64; 3]; 2]> = None;

    let mut scene = Scene::new();

    while let Some(e) = window.next() {
        window.draw_2d(&e, |context, graphics, _device| {
            let size = context.get_view_size();
            let sx = size[0] / 640.0;
            let sy = size[1] / 480.0;
            let letterbox_h = ((size[0] - 640.0 * sy) / 2.0).max(0.0);
            let letterbox_v = ((size[1] - 480.0 * sx) / 2.0).max(0.0);
            let scale = sx.min(sy);
            let t = math::multiply(
                math::scale(1.0 / scale, 1.0 / scale),
                math::translate([-letterbox_h, -letterbox_v]),
            );
            let context = context
                .trans(letterbox_h, letterbox_v)
                .scale(scale, scale);
            transform = Some(t);
            clear([90.0 / 255.0, 202.0 / 255.0, 77.0 / 255.0, 1.0], graphics);
            scene.draw(context, graphics);
            if letterbox_v > 0.0 {
                piston_window::rectangle([0.0, 0.0, 0.0, 1.0],
                                         [0.0, -letterbox_v, 640.0, letterbox_v],
                                         context.transform,
                                         graphics);
                piston_window::rectangle([0.0, 0.0, 0.0, 1.0],
                                         [0.0, 480.0, 640.0, letterbox_v],
                                         context.transform,
                                         graphics);
            }
            if letterbox_h > 0.0 {
                piston_window::rectangle([0.0, 0.0, 0.0, 1.0],
                                         [-letterbox_h, 0.0, letterbox_h, 480.0],
                                         context.transform,
                                         graphics);
                piston_window::rectangle([0.0, 0.0, 0.0, 1.0],
                                         [640.0, 0.0, letterbox_h, 480.0],
                                         context.transform,
                                         graphics);
            }
        });
        
        if let Some(args) = e.update_args() {
            scene.update(args.dt);

            if let Some(ref mut selected) = selected {
                selected.borrow_mut().update_selected(args.dt);
            }
        }

        if let Some(button) = e.press_args() {
            if let (Some(pos), Some(transform)) = (last_pos, transform) {
                let pos = math::transform_pos(transform, pos);
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
            if let (Some(pos), Some(last_pos), Some(transform)) = (e.mouse_cursor_args(), last_pos, transform) {
                let pos = math::transform_pos(transform, pos);
                let last_pos = math::transform_pos(transform, last_pos);
                selected.borrow_mut().drag(last_pos, pos);
            }
        }

        if let Some(pos) = e.mouse_cursor_args() {
            last_pos = Some(pos);
        }
    }
}
