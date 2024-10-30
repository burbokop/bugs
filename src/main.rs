#![feature(new_range_api)]
#![feature(extract_if)]

use environment::{Environment, FoodSource, FoodSourceCreateInfo};
use math::Point;
use render::{BrainRenderModel, Camera, EnvironmentRenderModel};
use slint::{ComponentHandle, PlatformError, Timer, TimerMode};
use std::rc::Rc;
use std::time::{Duration, Instant};
use std::{cell::RefCell, ops::Range};
use utils::{color_to_slint_rgba_color, Float};

mod brain;
mod bug;
mod chromo_utils;
mod environment;
mod math;
mod render;
mod utils;

slint::slint! {
    export { MainWindow, BugInfo, EnvInfo } from "src/main.slint";
}
struct State {
    environment: Environment,
    camera: Camera,
    environment_render_model: RefCell<EnvironmentRenderModel>,
    brain_render_model: RefCell<BrainRenderModel>,
    selected_bug_id: Option<usize>,
    time_speed: Float,
    pause: bool,
    selected_node: Option<(usize, usize)>,
}

pub fn main() -> Result<(), PlatformError> {
    let mut rng = rand::thread_rng();

    let state = Rc::new(RefCell::new(State {
        environment: Environment::new(
            &mut rng,
            vec![
                FoodSourceCreateInfo {
                    position: (0., 0.).into(),
                    size: (1000., 1000.).into(),
                    energy_range: (0. ..1.).into(),
                    spawn_interval: Duration::from_millis(1000),
                },
                FoodSourceCreateInfo {
                    position: (0., 2000.).into(),
                    size: (2000., 2000.).into(),
                    energy_range: (0. ..1.).into(),
                    spawn_interval: Duration::from_millis(2000 * 10),
                },
                FoodSourceCreateInfo {
                    position: (-2000., 0.).into(),
                    size: (2000., 2000.).into(),
                    energy_range: (0. ..1.).into(),
                    spawn_interval: Duration::from_millis(4000 * 10),
                },
                FoodSourceCreateInfo {
                    position: (0., -2000.).into(),
                    size: (2000., 2000.).into(),
                    energy_range: (0. ..1.).into(),
                    spawn_interval: Duration::from_millis(8000 * 10),
                },
                FoodSourceCreateInfo {
                    position: (2000., 0.).into(),
                    size: (2000., 2000.).into(),
                    energy_range: (0. ..1.).into(),
                    spawn_interval: Duration::from_millis(16000 * 10),
                },
            ],
            -1000. ..1000.,
            -1000. ..1000.,
            0. ..1.,
            32768,
            (0., 0.).into(),
        ),
        selected_bug_id: None,
        camera: Default::default(),
        environment_render_model: Default::default(),
        brain_render_model: Default::default(),
        time_speed: 1.,
        pause: true,
        selected_node: None,
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
                if !state.pause {
                    let time_speed = state.time_speed;
                    state.environment.proceed(dt.mul_f64(time_speed), &mut rng);
                }
            },
        );
    }

    let main_window = MainWindow::new().unwrap();

    {
        let weak_state = Rc::downgrade(&state);
        main_window.on_pointer_event(move |k, x: f32, y: f32| {
            let state = weak_state.upgrade().unwrap();
            let mut state = state.try_borrow_mut().unwrap();

            let point: Point<_> = &(!&state.camera.transformation()).unwrap()
                * &Point::from((x as Float, y as Float));

            if k == 0 {
                let nearest_bug = state
                    .environment
                    .bugs()
                    .min_by(|a, b| {
                        (point - a.position())
                            .len()
                            .partial_cmp(&(point - b.position()).len())
                            .unwrap()
                    })
                    .map(|bug| (bug.id(), bug.position()));

                if let Some(nearest_bug) = nearest_bug {
                    state.selected_bug_id =
                        if (point - nearest_bug.1).len() < bug::EAT_FOOD_MAX_PROXIMITY {
                            Some(nearest_bug.0)
                        } else {
                            None
                        };
                }
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
                state.camera.add_translation(
                    (angle_delta_to_translation_delta(delta_y as Float), 0.).into(),
                );
            } else {
                // scroll vertically
                state.camera.add_translation(
                    (0., angle_delta_to_translation_delta(delta_y as Float)).into(),
                );
            }

            true
        });
    }

    {
        let _weak_state = Rc::downgrade(&state);
        main_window.on_key_press_event(move |_text| false);
    }

    {
        let weak_state = Rc::downgrade(&state);
        main_window.on_key_release_event(move |text| {
            let state = weak_state.upgrade().unwrap();
            let mut state = state.try_borrow_mut().unwrap();
            if let Ok(lvl) = text.parse::<u32>() {
                state.time_speed = (2_u32).pow(lvl) as f64;
                true
            } else if text == " " {
                state.pause = !state.pause;
                true
            } else if text == "w" {
                let i = &mut state.selected_node.get_or_insert((0, 0)).1;
                *i = (*i - 1) % 8;
                true
            } else if text == "a" {
                let i = &mut state.selected_node.get_or_insert((0, 0)).0;
                *i = (*i - 1) % 2;
                true
            } else if text == "s" {
                let i = &mut state.selected_node.get_or_insert((0, 0)).1;
                *i = (*i + 1) % 8;
                true
            } else if text == "d" {
                let i = &mut state.selected_node.get_or_insert((0, 0)).0;
                *i = (*i + 1) % 2;
                true
            } else if text == "f" {
                state.selected_node = None;
                true
            } else {
                false
            }
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

                    let mut environment_render_model = state.environment_render_model.borrow_mut();

                    let texture = environment_render_model.render(
                        &state.environment,
                        &state.camera,
                        &state.selected_bug_id,
                        window.get_requested_env_canvas_width() as u32,
                        window.get_requested_env_canvas_height() as u32,
                    );
                    window.set_env_canvas(texture);
                    window.set_env_info(EnvInfo {
                        now: state
                            .environment
                            .now()
                            .duration_since(state.environment.creation_time().clone())
                            .as_millis() as i64,
                        pause: state.pause,
                        time_speed: state.time_speed as f32,
                    });
                    window.set_fps(1. / dt.as_secs_f32());

                    if let Some(bug) = state
                        .selected_bug_id
                        .and_then(|id| state.environment.find_bug_by_id(id))
                    {
                        window.set_selected_bug_info(BugInfo {
                            genes: bug
                                .chromosome()
                                .genes
                                .iter()
                                .map(|x| *x as f32)
                                .collect::<Vec<_>>()[..]
                                .into(),
                            age: bug.age(state.environment.now().clone()).unwrap() as f32,
                            baby_charge: bug.baby_charge().unwrap() as f32,
                            color: color_to_slint_rgba_color(bug.color()).into(),
                            energy_level: bug.energy_level().unwrap() as f32,
                            id: bug.id() as i32,
                            rotation: bug.rotation().degrees() as f32,
                            size: bug.size().unwrap() as f32,
                            x: *bug.position().x() as f32,
                            y: *bug.position().y() as f32,
                        });

                        if let Some(brain_log) = bug.last_brain_log() {
                            let mut brain_render_model = state.brain_render_model.borrow_mut();

                            window.set_brain_canvas(brain_render_model.render(
                                bug.brain(),
                                brain_log,
                                state.selected_node,
                                window.get_requested_brain_canvas_width() as u32,
                                window.get_requested_brain_canvas_height() as u32,
                            ));

                            window.set_selected_bug_last_brain_log(BugBrainLog {
                                input: BugBrainInput {
                                    age: brain_log.input.age.unwrap() as f32,
                                    baby_charge: brain_log.input.baby_charge.unwrap() as f32,
                                    color_of_nearest_bug: color_to_slint_rgba_color(
                                        &brain_log.input.color_of_nearest_bug,
                                    )
                                    .into(),
                                    direction_to_nearest_bug: brain_log
                                        .input
                                        .direction_to_nearest_bug
                                        .degrees()
                                        as f32,
                                    direction_to_nearest_food: brain_log
                                        .input
                                        .direction_to_nearest_food
                                        .degrees()
                                        as f32,
                                    energy_level: brain_log.input.energy_level.unwrap() as f32,
                                    proximity_to_bug: brain_log.input.proximity_to_bug as f32,
                                    proximity_to_food: brain_log.input.proximity_to_food as f32,
                                },
                                output: BugBrainOutput {
                                    baby_charging_rate: brain_log.output.baby_charging_rate.unwrap()
                                        as f32,
                                    desired_rotation: brain_log.output.desired_rotation.degrees()
                                        as f32,
                                    rotation_velocity: brain_log
                                        .output
                                        .rotation_velocity
                                        .unwrap_degrees()
                                        as f32,
                                    velocity: brain_log.output.velocity as f32,
                                },
                            });
                        }
                    }

                    window.window().request_redraw();
                }
            },
        );
    }

    main_window.run()
}
