use app_utils::color_to_slint_rgba_color;
use bugs::environment::{Environment, FoodSourceCreateInfo};
use bugs::math::{noneg_float, Angle, Point};
use bugs::time_point::{StaticTimePoint, TimePoint as _};
use bugs::utils::{pretty_duration, Color, Float};
use render::{BrainRenderModel, Camera, EnvironmentRenderModel};
use slint::{CloseRequestResponse, ComponentHandle, PlatformError, Timer, TimerMode};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

mod app_utils;
mod render;

slint::slint! {
    export { MainWindow, BugInfo, EnvInfo } from "src/main.slint";
}

struct State {
    environment: Environment<StaticTimePoint>,
    camera: Camera,
    environment_render_model: RefCell<EnvironmentRenderModel>,
    brain_render_model: RefCell<BrainRenderModel>,
    selected_bug_id: Option<usize>,
    time_speed: Float,
    pause: bool,
    selected_node: Option<(usize, usize)>,
    tps: Float,
}

pub fn main() -> Result<(), PlatformError> {
    let exe_path = std::env::current_exe().unwrap();
    let exe_dir = exe_path.parent().unwrap();
    let save_path = exe_dir.join("save.json");
    println!("save_path: {:?}", save_path);

    let mut rng = rand::thread_rng();

    let state = Rc::new(RefCell::new(State {
        environment: if save_path.exists() {
            serde_json::from_str(&std::fs::read_to_string(&save_path).unwrap()).unwrap()
        } else {
            Environment::generate(
                StaticTimePoint::default(),
                &mut rng,
                // max energy increases by 2^x, and spawn interval increases by 3^x
                vec![
                    FoodSourceCreateInfo {
                        position: (0., 0.).into(),
                        size: (1000., 1000.).into(),
                        energy_range: (0. ..1.).into(),
                        spawn_interval: Duration::from_millis((4_u64).pow(0) * 1000),
                    },
                    FoodSourceCreateInfo {
                        position: (0., 0.).into(),
                        size: (2000., 2000.).into(),
                        energy_range: (0. ..2.).into(),
                        spawn_interval: Duration::from_millis((4_u64).pow(1) * 1000),
                    },
                    FoodSourceCreateInfo {
                        position: (0., 0.).into(),
                        size: (4000., 4000.).into(),
                        energy_range: (0. ..4.).into(),
                        spawn_interval: Duration::from_millis((4_u64).pow(2) * 1000),
                    },
                    FoodSourceCreateInfo {
                        position: (0., 0.).into(),
                        size: (16000., 16000.).into(),
                        energy_range: (0. ..8.).into(),
                        spawn_interval: Duration::from_millis((4_u64).pow(3) * 1000),
                    },
                    FoodSourceCreateInfo {
                        position: (0., 0.).into(),
                        size: (32000., 32000.).into(),
                        energy_range: (0. ..16.).into(),
                        spawn_interval: Duration::from_millis((4_u64).pow(4) * 1000),
                    },
                    FoodSourceCreateInfo {
                        position: (0., 0.).into(),
                        size: (64000., 64000.).into(),
                        energy_range: (0. ..32.).into(),
                        spawn_interval: Duration::from_millis((4_u64).pow(5) * 1000),
                    },
                ],
                -1000. ..1000.,
                -1000. ..1000.,
                0. ..1.,
                32768,
                (0., 0.).into(),
            )
        },
        selected_bug_id: None,
        camera: Default::default(),
        environment_render_model: Default::default(),
        brain_render_model: Default::default(),
        time_speed: 1.,
        pause: true,
        selected_node: None,
        tps: 0.,
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
                    state.tps = 1. / dt.as_secs_f64();
                } else {
                    state.tps = 0.;
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
                    .map(|bug| (bug.id(), bug.position(), bug.size()));

                if let Some(nearest_bug) = nearest_bug {
                    state.selected_bug_id = if (point - nearest_bug.1).len()
                        < (bugs::bug::EAT_FOOD_MAX_PROXIMITY * nearest_bug.2).unwrap()
                    {
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
        let save_path = save_path.clone();
        main_window.on_key_release_event(move |text| {
            let state = weak_state.upgrade().unwrap();
            let mut state = state.try_borrow_mut().unwrap();
            if let Ok(lvl) = text.parse::<u32>() {
                state.time_speed = (2_u32).pow(lvl) as f64;
                true
            } else if text == "q" {
                std::fs::write(
                    &save_path,
                    serde_json::to_string_pretty(&state.environment).unwrap(),
                )
                .unwrap();
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
        #[cfg(not(debug_assertions))]
        let render_interval = Duration::from_millis(1000 / 30);
        #[cfg(debug_assertions)]
        let render_interval = Duration::from_millis(2000);

        let weak_state = Rc::downgrade(&state);
        let weak_window = main_window.as_weak();
        render_timer.start(TimerMode::Repeated, render_interval, move || {
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
                    now: pretty_duration(
                        state
                            .environment
                            .now()
                            .duration_since(state.environment.creation_time()),
                    )
                    .into(),
                    pause: state.pause,
                    time_speed: state.time_speed as f32,
                    bugs_count: state.environment.bugs_count() as i32,
                    food_count: state.environment.food().len() as i32,
                });
                window.set_fps(1. / dt.as_secs_f32());
                window.set_tps(state.tps as f32);

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
                        baby_charge_level: bug.baby_charge_level().unwrap() as f32,
                        baby_charge_capacity: bug.baby_charge_capacity().unwrap() as f32,
                        color: color_to_slint_rgba_color(bug.color()).into(),
                        energy_level: bug.energy_level().unwrap() as f32,
                        energy_capacity: bug.energy_capacity().unwrap() as f32,
                        id: bug.id() as i32,
                        rotation: bug.rotation().degrees() as f32,
                        size: bug.size().unwrap() as f32,
                        x: *bug.position().x() as f32,
                        y: *bug.position().y() as f32,
                        heat_capacity: bug.heat_capacity().unwrap() as f32,
                        heat_level: bug.heat_level().unwrap() as f32,
                        vision_range: bug.vision_range().unwrap() as f32,
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
                                color_of_nearest_bug: color_to_slint_rgba_color(
                                    &brain_log.input.color_of_nearest_bug.as_ref().unwrap_or(
                                        &Color {
                                            a: 0.,
                                            r: 0.,
                                            g: 0.,
                                            b: 0.,
                                        },
                                    ),
                                )
                                .into(),
                                direction_to_nearest_bug: brain_log
                                    .input
                                    .direction_to_nearest_bug
                                    .unwrap_or(Angle::from_radians(0.))
                                    .degrees()
                                    as f32,
                                direction_to_nearest_food: brain_log
                                    .input
                                    .direction_to_nearest_food
                                    .unwrap_or(Angle::from_radians(0.))
                                    .degrees()
                                    as f32,
                                rotation: brain_log.input.rotation.degrees() as f32,
                                proximity_to_bug: brain_log
                                    .input
                                    .proximity_to_bug
                                    .unwrap_or(noneg_float(1.))
                                    .unwrap()
                                    as f32,
                                proximity_to_food: brain_log
                                    .input
                                    .proximity_to_food
                                    .unwrap_or(noneg_float(1.))
                                    .unwrap()
                                    as f32,
                            },
                            output: BugBrainOutput {
                                baby_charging_rate: brain_log.output.baby_charging_rate.unwrap()
                                    as f32,
                                desired_rotation: (bug.rotation()
                                    + brain_log.output.relative_desired_rotation)
                                    .degrees()
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
        });
    }

    {
        let weak_state = Rc::downgrade(&state);
        main_window
            .window()
            .on_close_requested(move || -> CloseRequestResponse {
                let state = weak_state.upgrade().unwrap();
                let state = state.borrow();
                std::fs::write(
                    &save_path,
                    serde_json::to_string_pretty(&state.environment).unwrap(),
                )
                .unwrap();
                CloseRequestResponse::HideWindow
            });
    }

    main_window.run()
}
