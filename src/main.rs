#![feature(new_range_api)]
#![feature(extract_if)]

use complexible::complex_numbers::Angle;
use environment::Environment;
use math::Point;
use render::{Camera, RenderModel};
use slint::{ComponentHandle, PlatformError, RgbaColor, Timer, TimerMode};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use utils::Float;

mod brain;
mod bug;
mod chromo_utils;
mod environment;
mod math;
mod render;
mod utils;

fn color_to_rgba_color(c: &utils::Color) -> RgbaColor<f32> {
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
    environment: Environment,
    camera: Camera,
    render_model: RefCell<RenderModel>,
    selected_bug_id: Option<usize>,
    time_speed: Float,
}

pub fn main() -> Result<(), PlatformError> {
    let width = 1000;
    let height = 1000;

    let mut rng = rand::thread_rng();

    let state = Rc::new(RefCell::new(State {
        environment: Environment::new(
            &mut rng,
            0. ..(width as Float),
            0. ..(height as Float),
            0. ..1.,
            512,
            (500., 500.).into()
        ),
        selected_bug_id: None,
        camera: Default::default(),
        render_model: Default::default(),
        time_speed: 0.
    }));

    let timer = Timer::default();
    let mut last_tick_instant = Instant::now();

    {
        let weak_state = Rc::downgrade(&state);
        timer.start(
            TimerMode::Repeated,
            std::time::Duration::from_millis(1000 / 30),
            move || {
                let now = Instant::now();
                let dt = now - last_tick_instant;
                last_tick_instant = now;
                let state = weak_state.upgrade().unwrap();
                let mut state = state.borrow_mut();
                let time_speed = state.time_speed;
                state.environment.proceed(dt.mul_f64(time_speed), &mut rng);
            },
        );
    }

    let main_window = MainWindow::new().unwrap();

    {
        let weak_state = Rc::downgrade(&state);
        main_window.on_pointer_event(move |k, x: f32, y: f32| {
            let state = weak_state.upgrade().unwrap();
            let mut state = state.try_borrow_mut().unwrap();

            let point: Point<_> =
                &(!&state.camera.transformation()).unwrap() * &Point::from((x as Float, y as Float));

            if k == 0 {
                let selected_bug_id = state.environment.bugs().find_map(|bug| {
                    if (point - bug.position()).len() < bug::EAT_FOOD_MAX_PROXIMITY {
                        Some(bug.id())
                    } else {
                        None
                    }
                });

                state.selected_bug_id = selected_bug_id;
            }
        });
    }

    {
        let weak_state = Rc::downgrade(&state);
        main_window.on_scroll_event(move |pos_x, pos_y, _delta_x, delta_y, shift, control| {
            let position = (pos_x as Float, pos_y as Float).into();

            let state = weak_state.upgrade().unwrap();
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
    }

    {
        let _weak_state = Rc::downgrade(&state);
        main_window.on_key_press_event(move|_text| {
            false
        });
    }

    {
        let weak_state = Rc::downgrade(&state);
        main_window.on_key_release_event(move|text| {
            let state = weak_state.upgrade().unwrap();
            let mut state = state.try_borrow_mut().unwrap();
            if let Ok(lvl) = text.parse::<u32>() {
                state.time_speed = (2_u32).pow(lvl) as f64;
                true
            } else {false}
        });
    }
    main_window.invoke_init_focus();



    let mut prev_render_instant = Instant::now();

    let render_timer = Timer::default();

    {
        let weak_state = Rc::downgrade(&state);
        let weak_window = main_window.as_weak();
        render_timer.start(
            TimerMode::Repeated,
            std::time::Duration::from_millis(1000 / 30),
            move || {
                if let Some(window) = weak_window.upgrade() {
                    let now = Instant::now();
                    let dt = now - prev_render_instant;
                    prev_render_instant = now;

                    let state = weak_state.upgrade().unwrap();
                    let state = state.borrow();

                    let mut render_model = state.render_model.borrow_mut();

                    let texture = render_model.render(
                        &state.environment,
                        &state.camera,
                        &state.selected_bug_id,
                        window.get_requested_canvas_width() as u32,
                        window.get_requested_canvas_height() as u32,
                    );
                    window.set_canvas(texture);
                    window.set_fps(1. / dt.as_secs_f32());
                    window.set_time_speed(state.time_speed as f32);

                    if let Some(bug) = state
                        .selected_bug_id
                        .and_then(|id| state.environment.find_bug_by_id(id))
                    {
                        window.set_selected_bug_info(BugInfo {
                            age: bug.age(state.environment.now().clone()) as f32,
                            baby_charge: bug.baby_charge() as f32,
                            color: color_to_rgba_color(bug.color()).into(),
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
    }

    main_window.run()
}
