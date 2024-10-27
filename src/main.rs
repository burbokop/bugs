#![feature(new_range_api)]
#![feature(extract_if)]

use complexible::complex_numbers::Angle;
use environment::Environment;
use math::Point;
use render::{render_scene, Camera};
use slint::{ComponentHandle, PlatformError, RgbaColor, Timer, TimerMode};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};
use utils::Float;

mod brain;
mod bug;
mod chromo_utils;
mod environment;
mod math;
mod render;
mod utils;

fn colorToRgbaColor(c: &utils::Color) -> RgbaColor<f32> {
    RgbaColor {
        alpha: c.a as f32,
        red: c.r as f32,
        green: c.g as f32,
        blue: c.b as f32,
    }
}

slint::slint! {
    export { MainWindow, BugInfo } from "src/main.slint";
}

struct State {
    camera: Camera,
    selected_bug_id: Option<usize>,
}

pub fn main() -> Result<(), PlatformError> {
    let width = 800;
    let height = 600;

    let mut rng = rand::thread_rng();

    let environment = Rc::new(RefCell::new(Environment::new(
        &mut rng,
        0. ..(width as Float),
        0. ..(height as Float),
        0. ..1.,
        256,
    )));

    let state = Rc::new(RefCell::new(State {
        selected_bug_id: None,
        camera: Default::default(),
    }));

    let weak_state = Rc::downgrade(&state);

    let weak_environment = Rc::downgrade(&environment);

    let timer = Timer::default();
    let mut last_tick_instant = Instant::now();

    let weak_environment_copy0 = weak_environment.clone();
    timer.start(
        TimerMode::Repeated,
        std::time::Duration::from_millis(1000 / 30),
        move || {
            let now = Instant::now();
            let dt = now - last_tick_instant;
            last_tick_instant = now;
            let environment = weak_environment_copy0.upgrade().unwrap();
            let mut environment = environment.try_borrow_mut().unwrap();
            environment.proceed(dt, &mut rng);
        },
    );

    let main_window = MainWindow::new().unwrap();
    let weak_main_window0 = main_window.as_weak();
    let weak_main_window1 = main_window.as_weak();

    let weak_environment_copy1 = weak_environment.clone();
    let weak_state_copy1 = weak_state.clone();
    main_window.on_pointer_event(move |k, x: f32, y: f32| {
        let state = weak_state_copy1.upgrade().unwrap();
        let mut state = state.try_borrow_mut().unwrap();

        let point: Point<_> =
            &(!&state.camera.transformation()).unwrap() * &Point::from((x as Float, y as Float));

        if k == 0 {
            let main_window = weak_main_window0.upgrade().unwrap();
            let environment = weak_environment_copy1.upgrade().unwrap();
            let environment = environment.borrow();
            let cw = main_window.get_requested_canvas_width();
            let ch = main_window.get_requested_canvas_height();

            for bug in environment.bugs() {
                if (point - bug.position()).len() < 10. {
                    println!("select: {:?}", bug.id());
                    state.selected_bug_id = Some(bug.id());
                    break;
                }
            }
        }
    });

    let weak_state_copy3 = weak_state.clone();
    main_window.on_scroll_event(move |pos_x, pos_y, delta_x, delta_y, shift, control| {
        let position = (pos_x as Float, pos_y as Float).into();

        let state = weak_state_copy3.upgrade().unwrap();
        let mut state = state.try_borrow_mut().unwrap();

        let default_deltas_per_step: Float = 120.;

        let angle_delta_to_scale_division = |angle_delta: Float| {
            let base: Float = 1.2;

            base.powf(angle_delta / default_deltas_per_step)
        };

        let angle_delta_to_translation_delta = |angle_delta: Float| {
            let velocity: Float = 10.; // px per step
            return velocity * angle_delta / default_deltas_per_step;
        };

        if control {
            // zoom
            state.camera.concat_scale_centered(
                angle_delta_to_scale_division(delta_y as Float),
                position,
                position,
            );
        } else if shift {
            // scroll horizontally
            state
                .camera
                .add_translation((angle_delta_to_translation_delta(delta_y as Float), 0.).into());
        } else {
            // scroll vertically
            state
                .camera
                .add_translation((0., angle_delta_to_translation_delta(delta_y as Float)).into());
        }

        true
    });

    let weak_environment_copy2 = weak_environment.clone();
    let weak_state_copy2 = weak_state.clone();

    let mut prev_render_instant = Instant::now();

    let render_timer = Timer::default();
    render_timer.start(
        TimerMode::Repeated,
        std::time::Duration::from_millis(1000 / 30),
        move || {
            if let Some(window) = weak_main_window1.upgrade() {
                let now = Instant::now();
                let dt = now - prev_render_instant;
                prev_render_instant = now;

                let environment = weak_environment.upgrade().unwrap();
                let environment = environment.try_borrow_mut().unwrap();
                let state = weak_state_copy2.upgrade().unwrap();
                let state = state.borrow();

                let texture = render_scene(
                    &environment,
                    &state.camera,
                    &state.selected_bug_id,
                    window.get_requested_canvas_width() as u32,
                    window.get_requested_canvas_height() as u32,
                );
                window.set_canvas(texture);
                window.set_fps(1. / dt.as_secs_f32());

                if let Some(bug) = state
                    .selected_bug_id
                    .and_then(|id| environment.find_bug_by_id(id))
                {
                    window.set_selected_bug_info(BugInfo {
                        age: bug.age() as f32,
                        baby_charge: bug.baby_charge() as f32,
                        color: colorToRgbaColor(bug.color()).into(),
                        energy_level: bug.energy_level() as f32,
                        id: bug.id() as i32,
                        rotation: Angle::from_radians(bug.rotation()).d.value as f32,
                        size: bug.size() as f32,
                        x: *bug.position().x() as f32,
                        y: *bug.position().y() as f32,
                    });
                }

                window.window().request_redraw();
            }
        },
    );

    main_window.run()
}
