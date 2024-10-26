#![feature(new_range_api)]
#![feature(extract_if)]

use complexible::complex_numbers::{Angle, ComplexNumber};
use environment::Environment;
use math::Point;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::surface::Surface;
use slint::{ComponentHandle, Image, PlatformError, RgbaColor, SharedPixelBuffer, Timer, TimerMode};
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::time::Instant;
use utils::Float;

mod brain;
mod bug;
mod chromo_utils;
mod environment;
mod utils;
mod math;

fn colorToRgbaColor(c: &utils::Color) -> RgbaColor<f32> {
    RgbaColor { alpha: c.a as f32, red: c.r as f32, green: c.g as f32, blue: c.b as f32 }
}

slint::slint! {
    export { MainWindow, BugInfo } from "src/main.slint";
}

fn render(
    environment: &Environment,
    state: &State,
    requested_canvas_width: u32,
    requested_canvas_height: u32,
) -> Image {
    let mut pixel_buffer = SharedPixelBuffer::new(requested_canvas_width, requested_canvas_height);
    let size = (pixel_buffer.width(), pixel_buffer.height());
    assert_eq!(
        pixel_buffer.as_bytes().len(),
        pixel_buffer.width() as usize * pixel_buffer.height() as usize * 4
    );

    {
        let surface = Surface::from_data(
            pixel_buffer.make_mut_bytes(),
            size.0,
            size.1,
            size.0 * 4,
            sdl2::pixels::PixelFormatEnum::RGBA32,
        )
        .unwrap();

        let mut canvas = surface.into_canvas().unwrap();

        canvas.set_draw_color(Color::RGB(211, 250, 199));
        canvas.clear();

        canvas.set_draw_color(Color::RGB(73, 54, 87));
        for food in environment.food() {
            canvas
                .fill_rect(Rect::from_center(
                    (food.position().x() as i32, food.position().y() as i32),
                    (food.energy() * 10.) as u32,
                    (food.energy() * 10.) as u32,
                ))
                .unwrap();
        }

        canvas.set_draw_color(Color::RGB(255, 183, 195));
        for bug in environment.bugs() {
            let rotation = ComplexNumber::from_polar(1., Angle::from_radians(bug.rotation()));
            let pos = ComplexNumber::from_cartesian(bug.position().x(), bug.position().y());

            let size = 5.;

            let p0 = ComplexNumber::from_cartesian(4. * size, 0. * size);
            let p1 = ComplexNumber::from_cartesian(-1. * size, -1. * size);
            let p2 = ComplexNumber::from_cartesian(-1. * size, 1. * size);

            let pp0 = p0.mul(&rotation).add(&pos);
            let pp1 = p1.mul(&rotation).add(&pos);
            let pp2 = p2.mul(&rotation).add(&pos);

            canvas
                .filled_trigon(
                    pp0.real() as i16,
                    pp0.imag() as i16,
                    pp1.real() as i16,
                    pp1.imag() as i16,
                    pp2.real() as i16,
                    pp2.imag() as i16,
                    Color::RGB(255, 183, 195),
                )
                .unwrap();

            if Some(bug.id()) == state.selected_bug_id {
                canvas.circle(
                    bug.position().x() as i16,
                    bug.position().y() as i16,
                    bug::EAT_FOOD_MAX_PROXIMITY as i16,
                    Color::RGB(255, 183, 195),
                ).unwrap();
            }
        }

        canvas.present();
    }
    slint::Image::from_rgba8(pixel_buffer)
}

struct State {
    selected_bug_id: Option<usize>
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

    let state = Rc::new(RefCell::new(State { selected_bug_id: None }));

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
        let point: Point<_> = (x as Float, y as Float).into();
        let state = weak_state_copy1.upgrade().unwrap();
        let mut state = state.try_borrow_mut().unwrap();

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

        // println!("x: {:?}, {:?}, {:?}, {:?}, {:?}", k, x, y, cw, ch);
    });

    let weak_environment_copy2 = weak_environment.clone();
    let weak_state_copy2 = weak_state.clone();

    let render_timer = Timer::default();
    render_timer.start(
        TimerMode::Repeated,
        std::time::Duration::from_millis(1000 / 30),
        move || {
            if let Some(window) = weak_main_window1.upgrade() {
                let environment = weak_environment.upgrade().unwrap();
                let environment = environment.try_borrow_mut().unwrap();
                let state = weak_state_copy2.upgrade().unwrap();
                let state = state.borrow();

                let texture = render(
                    &environment,
                    &state,
                    window.get_requested_canvas_width() as u32,
                    window.get_requested_canvas_height() as u32,
                );
                window.set_canvas(texture);

                if let Some(bug) = state.selected_bug_id.and_then(|id| environment.find_bug_by_id(id)) {
                    window.set_selected_bug_info(BugInfo {
                        age: bug.age() as f32,
                        baby_charge: bug.baby_charge() as f32,
                        color: colorToRgbaColor(bug.color()).into(),
                        energy_level: bug.energy_level() as f32,
                        id: bug.id() as i32,
                        rotation: Angle::from_radians(bug.rotation()).d.value as f32,
                        size: bug.size() as f32,
                        x: bug.position().x() as f32,
                        y: bug.position().y() as f32,
                    });
                }

                window.window().request_redraw();
            }
        },
    );

    main_window.run()
}
