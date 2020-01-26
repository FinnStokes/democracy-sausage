use piston_window::*;
use fps_counter::FPSCounter;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

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
        //.vsync(true)
        //.fullscreen(true)
        .build().unwrap();

    let mut fps_counter = FPSCounter::new();
    let mut fps = 0;

    let mut selected: Option<Rc<RefCell<dyn Entity>>> = None;
    let mut last_pos: Option<[f64; 2]> = None;
    let mut transform: Option<[[f64; 3]; 2]> = None;

    let mut scene = Scene::new();

    let mut last_time = Instant::now();

    while let Some(e) = window.next() {
        window.draw_2d(&e, |context, raw_graphics, _device| {
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
            clear([90.0 / 255.0, 202.0 / 255.0, 77.0 / 255.0, 1.0], raw_graphics);
            scene.draw(context, raw_graphics);
            if letterbox_v > 0.0 {
                piston_window::rectangle([0.0, 0.0, 0.0, 1.0],
                                         [0.0, -2.0 * letterbox_v, 640.0, 2.0 * letterbox_v],
                                         context.transform,
                                         raw_graphics);
                piston_window::rectangle([0.0, 0.0, 0.0, 1.0],
                                         [0.0, 480.0, 640.0, 2.0 * letterbox_v],
                                         context.transform,
                                         raw_graphics);
            }
            if letterbox_h > 0.0 {
                piston_window::rectangle([0.0, 0.0, 0.0, 1.0],
                                         [-2.0 * letterbox_h, 0.0, 2.0 * letterbox_h, 480.0],
                                         context.transform,
                                         raw_graphics);
                piston_window::rectangle([0.0, 0.0, 0.0, 1.0],
                                         [640.0, 0.0, 2.0 * letterbox_h, 480.0],
                                         context.transform,
                                         raw_graphics);
            }
            fps = fps_counter.tick();
        });
        window.set_title(fps.to_string());
        
        if let Some(_) = e.update_args() {
            let time = Instant::now();
            let dt = (time - last_time).as_secs_f64();
            last_time = time;
            scene.update(dt);

            if let Some(ref mut selected) = selected {
                selected.borrow_mut().update_selected(dt);
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