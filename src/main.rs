#![deny(unused_imports)]

use app_utils::color_to_slint_rgba_f32_color;
use bugs_lib::env_presets;
use bugs_lib::environment::SeededEnvironment;
use bugs_lib::math::{noneg_float, Angle, LerpIntegrator, NoNeg, Point};
use bugs_lib::time_point::{StaticTimePoint, TimePoint as _};
use bugs_lib::utils::{pretty_duration, Color, Float};
use clap::Parser;
use rand::Rng;
use render::sdl::{SdlBrainRenderModel, SdlEnvironmentRenderModel};
use render::vulkan::{VulkanBrainRenderModel, VulkanEnvironmentRenderModel};
use render::{BrainRenderer, Camera, ChunksDisplayMode, EnvironmentRenderer};
use slint::{CloseRequestResponse, ComponentHandle, PlatformError, Timer, TimerMode};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, Instant};

mod app_utils;
mod render;

slint::slint! {
    export { MainWindow, BugInfo, EnvInfo, DisplayTool } from "src/main.slint";
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tool {
    Nuke,
    Food,
    SpawnBug,
    None,
}

impl From<Tool> for DisplayTool {
    fn from(value: Tool) -> Self {
        match value {
            Tool::Nuke => Self::Nuke,
            Tool::Food => Self::Food,
            Tool::SpawnBug => Self::SpawnBug,
            Tool::None => Self::None,
        }
    }
}

impl From<DisplayTool> for Tool {
    fn from(value: DisplayTool) -> Self {
        match value {
            DisplayTool::Nuke => Self::Nuke,
            DisplayTool::Food => Self::Food,
            DisplayTool::SpawnBug => Self::SpawnBug,
            DisplayTool::None => Self::None,
        }
    }
}

pub const NUKE_RADIUS: NoNeg<Float> = noneg_float(200.);

struct State {
    environment: SeededEnvironment<StaticTimePoint>,
    camera: Camera,
    environment_render_model: RefCell<EnvironmentRenderer<StaticTimePoint>>,
    brain_render_model: RefCell<BrainRenderer>,
    selected_bug_id: Option<usize>,
    time_speed: Float,
    pause: bool,
    selected_node: Option<(usize, usize)>,
    tps: Float,
    active_tool: Tool,
    tool_action_point: Option<Point<Float>>,
    tool_action_active: bool,
    chunks_display_mode: ChunksDisplayMode,
    do_render: bool,
    desired_tps: Float,
    quality_deterioration: u32,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
enum Args {
    New(NewCommand),
    Load(LoadCommand),
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
#[clap(rename_all = "kebab_case")]
enum EnvPreset {
    NestedRects,
    Circle,
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
#[clap(rename_all = "kebab_case")]
enum Renderer {
    Sdl,
    Vulkan,
}

/// Generates simulation environment from one of builtin presets
#[derive(Parser)]
struct NewCommand {
    #[arg(short, long)]
    env_preset: EnvPreset,
    #[arg(short, long)]
    renderer: Renderer,
}

/// Loads simulation environment from json save file
#[derive(Parser)]
struct LoadCommand {
    #[arg(short, long)]
    save_file: Option<PathBuf>,
    #[arg(short, long, default_value = "sdl")]
    renderer: Renderer,
}

pub fn main() -> Result<(), PlatformError> {
    let (save_path, environment, renderer) = match Args::parse() {
        Args::New(command) => {
            let exe_path = std::env::current_exe().unwrap();
            let exe_dir = exe_path.parent().unwrap();
            let save_path = exe_dir.join("save.json");

            (
                save_path,
                match command.env_preset {
                    EnvPreset::NestedRects => env_presets::less_food_further_from_center(
                        StaticTimePoint::default(),
                        rand::rng().random(),
                    ),
                    EnvPreset::Circle => env_presets::one_big_circle(
                        StaticTimePoint::default(),
                        rand::rng().random(),
                    ),
                },
                command.renderer,
            )
        }
        Args::Load(command) => {
            let save_path = command.save_file.unwrap_or_else(|| {
                let exe_path = std::env::current_exe().unwrap();
                let exe_dir = exe_path.parent().unwrap();
                exe_dir.join("save.json")
            });
            (
                save_path.clone(),
                serde_json::from_str(&std::fs::read_to_string(&save_path).unwrap()).unwrap(),
                command.renderer,
            )
        }
    };

    println!(
        "save_path: {:?}, (exist: {})",
        save_path,
        save_path.exists()
    );

    let state = Rc::new(RefCell::new(State {
        environment,
        selected_bug_id: None,
        camera: Default::default(),
        environment_render_model: match renderer {
            Renderer::Sdl => RefCell::new(EnvironmentRenderer::new(
                SdlEnvironmentRenderModel::default(),
            )),
            Renderer::Vulkan => RefCell::new(EnvironmentRenderer::new(
                VulkanEnvironmentRenderModel::default(),
            )),
        },
        brain_render_model: match renderer {
            Renderer::Sdl => RefCell::new(BrainRenderer::new(SdlBrainRenderModel::default())),
            Renderer::Vulkan => RefCell::new(BrainRenderer::new(VulkanBrainRenderModel::default())),
        },
        time_speed: 1.,
        pause: true,
        selected_node: None,
        tps: 0.,
        active_tool: Tool::None,
        tool_action_point: None,
        tool_action_active: false,
        chunks_display_mode: ChunksDisplayMode::None,
        do_render: true,
        desired_tps: 30.,
        quality_deterioration: 1,
    }));

    let (ctrl_c_tx, ctrl_c_rx) = std::sync::mpsc::channel();
    ctrlc::set_handler(move || {
        ctrl_c_tx
            .send(())
            .expect("Could not send signal on channel.")
    })
    .expect("Error setting Ctrl-C handler");

    let timer = Rc::new(Timer::default());
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
                    if state.tool_action_active {
                        if let Some(tool_action_point) = state.tool_action_point {
                            match state.active_tool {
                                Tool::Nuke => state
                                    .environment
                                    .irradiate_area(tool_action_point, NUKE_RADIUS),
                                Tool::Food => state.environment.add_food(tool_action_point),
                                Tool::SpawnBug => state.environment.add_bug(tool_action_point),
                                Tool::None => {}
                            }
                        }
                    }

                    let time_speed = state.time_speed;
                    state.environment.proceed(dt.mul_f64(time_speed));
                    state.tps = 1. / dt.as_secs_f64();
                } else {
                    state.tps = 0.;
                }
            },
        );
    }

    let weak_timer = Rc::downgrade(&timer);
    let set_desired_tps = move |state: &mut State, tps: Float| {
        let timer = weak_timer.upgrade().unwrap();
        timer.set_interval(std::time::Duration::from_millis((1000. / tps) as u64));
        state.desired_tps = tps;
    };

    let main_window = MainWindow::new().unwrap();

    {
        let weak_state = Rc::downgrade(&state);
        main_window.on_tool_clicked(move |tool: DisplayTool| {
            let state = weak_state.upgrade().unwrap();
            let mut state = state.try_borrow_mut().unwrap();
            state.active_tool = tool.into();
        })
    }

    {
        let weak_state = Rc::downgrade(&state);
        main_window.on_pointer_event(move |event_type, button, x: f32, y: f32| {
            let state = weak_state.upgrade().unwrap();
            let mut state = state.try_borrow_mut().unwrap();

            let point: Point<_> = &(!&state.camera.transformation()).unwrap()
                * &Point::from((x as Float, y as Float));

            if event_type == 0 {
                if button == 0 {
                    struct BugInfo {
                        id: usize,
                        position: Point<Float>,
                        eat_range: NoNeg<Float>,
                    }

                    let nearest_bug = state
                        .environment
                        .bugs()
                        .min_by(|a, b| {
                            (point - a.position())
                                .len()
                                .partial_cmp(&(point - b.position()).len())
                                .unwrap()
                        })
                        .map(|bug| BugInfo {
                            id: bug.id(),
                            position: bug.position(),
                            eat_range: bug.eat_range(),
                        });

                    if let Some(nearest_bug) = nearest_bug {
                        state.selected_bug_id = if (point - nearest_bug.position).len()
                            < nearest_bug.eat_range.unwrap()
                        {
                            Some(nearest_bug.id)
                        } else {
                            None
                        };
                    }
                    state.tool_action_active = false
                } else {
                    state.active_tool = Tool::None;
                }
            } else if event_type == 1 {
                if button == 0 {
                    state.tool_action_active = true
                }
            } else if event_type == 2 {
                state.tool_action_point = Some(point)
            } else if event_type == 3 {
                state.tool_action_point = None;
                state.tool_action_active = false
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

            // let delta_y = delta_y / 100.;

            if control {
                // zoom
                state.camera.concat_scale_centered(
                    angle_delta_to_scale_division(delta_y as Float),
                    position,
                    position,
                );
            } else if shift {
                // let delta_y = delta_y * 100.;

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

            let f1 = [0xEF, 0x9C, 0x84];
            let f2 = [0xEF, 0x9C, 0x85];
            let f3 = [0xEF, 0x9C, 0x86];

            if let Ok(lvl) = text.parse::<u32>() {
                if lvl > 0 {
                    state.time_speed = (2_u32).pow(lvl - 1) as f64;
                    // match lvl {
                    //     9 => set_desired_tps(240.),
                    //     8 => set_desired_tps(120.),
                    //     7 => set_desired_tps(60.),
                    //     _ => set_desired_tps(30.),
                    // }

                    match lvl {
                        1 => set_desired_tps(&mut state, 30.),
                        2 => set_desired_tps(&mut state, 60.),
                        3 => set_desired_tps(&mut state, 120.),
                        4 => set_desired_tps(&mut state, 240.),
                        5 => set_desired_tps(&mut state, 480.),
                        6 => set_desired_tps(&mut state, 960.),
                        7 => set_desired_tps(&mut state, 1920.),
                        8 => set_desired_tps(&mut state, 1920.),
                        9 => set_desired_tps(&mut state, 1920.),
                        _ => panic!("Oops"),
                    }
                }
                true
            } else if text.as_str().as_bytes() == f1 {
                state.chunks_display_mode = state.chunks_display_mode.clone().rotated();
                true
            } else if text.as_str().as_bytes() == f2 {
                state.environment.collect_unused_chunks();
                true
            } else if text.as_str().as_bytes() == f3 {
                state.do_render = !state.do_render;
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
            } else if text == "," {
                if state.quality_deterioration > 0 {
                    state.quality_deterioration -= 1;
                }
                true
            } else if text == "." {
                state.quality_deterioration += 1;
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
        let desired_fps = match renderer {
            Renderer::Sdl => 30,
            Renderer::Vulkan => 15,
        };

        let render_interval = Duration::from_millis(1000 / desired_fps);

        let weak_state = Rc::downgrade(&state);
        let weak_window = main_window.as_weak();
        let save_path = save_path.clone();
        let mut fps_integrator: LerpIntegrator<Float> = LerpIntegrator::new(0.2);
        let mut tps_integrator: LerpIntegrator<Float> = LerpIntegrator::new(0.2);
        render_timer.start(TimerMode::Repeated, render_interval, move || {
            if let Some(window) = weak_window.upgrade() {
                let now = Instant::now();
                let dt = now - prev_render_instant;
                prev_render_instant = now;

                let state = weak_state.upgrade().unwrap();
                let state = state.borrow();

                let mut environment_render_model = state.environment_render_model.borrow_mut();

                if state.do_render {
                    let texture = environment_render_model.render(
                        &state.environment,
                        &state.camera,
                        &state.selected_bug_id,
                        state.active_tool,
                        state.tool_action_point,
                        state.tool_action_active,
                        state.chunks_display_mode.clone(),
                        window.get_requested_env_canvas_width() as u32,
                        window.get_requested_env_canvas_height() as u32,
                        state.quality_deterioration,
                    );
                    window.set_env_canvas(texture);
                }
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
                    food_count: state.environment.food_count() as i32,
                });
                window.set_fps(*fps_integrator.proceed(1. / dt.as_secs_f64()) as f32);
                window.set_tps(*tps_integrator.proceed(state.tps) as f32);
                window.set_desired_fps(desired_fps as f32);
                window.set_desired_tps(state.desired_tps as f32);
                window.set_quality_deterioration(state.quality_deterioration as i32);

                window.set_active_tool(state.active_tool.into());

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
                        color: color_to_slint_rgba_f32_color(bug.color()).into(),
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
                        vision_arc: (bug.vision_half_arc().unwrap().degrees() * 2.) as f32,
                    });

                    if state.do_render {
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
                                    color_of_nearest_bug: color_to_slint_rgba_f32_color(
                                        &brain_log
                                            .input
                                            .nearest_bug
                                            .as_ref()
                                            .map(|x| x.color.clone())
                                            .unwrap_or(Color {
                                                a: 0.,
                                                r: 0.,
                                                g: 0.,
                                                b: 0.,
                                            }),
                                    )
                                    .into(),
                                    direction_to_nearest_bug: brain_log
                                        .input
                                        .nearest_bug
                                        .as_ref()
                                        .map(|x| x.direction)
                                        .unwrap_or(Angle::from_radians(0.))
                                        .degrees()
                                        as f32,
                                    direction_to_nearest_food: brain_log
                                        .input
                                        .nearest_food
                                        .as_ref()
                                        .map(|x| x.direction)
                                        .unwrap_or(Angle::from_radians(0.))
                                        .degrees()
                                        as f32,
                                    rotation: brain_log.input.rotation.degrees() as f32,
                                    proximity_to_bug: brain_log
                                        .input
                                        .nearest_bug
                                        .as_ref()
                                        .map(|x| x.dst)
                                        .unwrap_or(noneg_float(1.))
                                        .unwrap()
                                        as f32,
                                    proximity_to_food: brain_log
                                        .input
                                        .nearest_food
                                        .as_ref()
                                        .map(|x| x.dst)
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
                                        .unwrap()
                                        .degrees()
                                        as f32,
                                    velocity: brain_log.output.velocity as f32,
                                },
                            });
                        }
                    }
                }

                window.window().request_redraw();

                if let Ok(_) = ctrl_c_rx.try_recv() {
                    println!("\nSaving into: {:?}...", &save_path);
                    std::fs::write(
                        &save_path,
                        serde_json::to_string_pretty(&state.environment).unwrap(),
                    )
                    .unwrap();
                    window.window().hide().unwrap();
                }
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

    main_window.on_inv_color(|color| {
        slint::Color::from_argb_u8(
            color.alpha(),
            255 - color.red(),
            255 - color.green(),
            255 - color.blue(),
        )
    });

    main_window.run()
}
