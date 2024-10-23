#![feature(new_range_api)]
#![feature(extract_if)]

use complexible::complex_numbers::{Angle, ComplexNumber};
use draw::draw_filled_triangle;
use environment::Environment;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use utils::Float;
use std::time::{Duration, Instant};

mod environment;
mod brain;
mod utils;
mod bug;
mod draw;
mod chromo_utils;

fn complex_to_point(complex: ComplexNumber) -> Point {
    (complex.real() as i32, complex.imag() as i32).into()
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let width = 800;
    let height = 600;

    let mut rng = rand::thread_rng();

    let mut environment = Environment::new(
        &mut rng,
        0. ..(width as Float),
        0. ..(height as Float),
        0. ..1.,
        256
    );

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", width, height)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.set_draw_color(Color::RGB(255, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;

    let mut last_tick_instant = Instant::now();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        let now = Instant::now();
        let dt = now - last_tick_instant;
        last_tick_instant = now;
        environment.proceed(dt, &mut rng);

        canvas.set_draw_color(Color::RGB(211, 250, 199));
        canvas.clear();

        canvas.set_draw_color(Color::RGB(73, 54, 87));
        for food in environment.food() {
            canvas.fill_rect(Rect::from_center(
                (food.x() as i32, food.y() as i32),
                (food.energy() * 10.) as u32,
                (food.energy() * 10.) as u32
            )).unwrap();
        }

        canvas.set_draw_color(Color::RGB(255, 183, 195));
        for bug in environment.bugs() {
            let rotation = ComplexNumber::from_polar(1., Angle::from_radians(bug.rotation()));
            let pos = ComplexNumber::from_cartesian(bug.x(), bug.y());

            let size = 5.;

            let p0 = ComplexNumber::from_cartesian(4. * size, 0. * size);
            let p1 = ComplexNumber::from_cartesian(-1. * size, -1. * size);
            let p2 = ComplexNumber::from_cartesian(-1. * size, 1. * size);

            let pp0 = p0.mul(&rotation).add(&pos);
            let pp1 = p1.mul(&rotation).add(&pos);
            let pp2 = p2.mul(&rotation).add(&pos);

            draw_filled_triangle(
                &mut canvas,
                complex_to_point(pp0),
                complex_to_point(pp1),
                complex_to_point(pp2),
                Color::RGB(255, 183, 195));
        }

        canvas.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        // The rest of the game loop goes here...
    }

    Ok(())
}








