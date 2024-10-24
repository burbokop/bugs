#![feature(new_range_api)]
#![feature(extract_if)]

use complexible::complex_numbers::{Angle, ComplexNumber};
use environment::Environment;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::surface::Surface;
use slint::{PlatformError, SharedPixelBuffer, Timer, TimerMode};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use utils::Float;

mod brain;
mod bug;
mod chromo_utils;
mod environment;
mod utils;

slint::slint! {
    export { MainWindow } from "src/main.slint";
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

    let main_window = MainWindow::new().unwrap();

    let timer = Timer::default();
    let mut last_tick_instant = Instant::now();
    {
        let environment = environment.clone();
        timer.start(
            TimerMode::Repeated,
            std::time::Duration::from_millis(1000 / 30),
            move || {
                let now = Instant::now();
                let dt = now - last_tick_instant;
                last_tick_instant = now;
                environment.borrow_mut().proceed(dt, &mut rng);
            },
        );
    }

    main_window.on_render_environment(move |_: i64| -> slint::Image {
        let mut pixel_buffer = SharedPixelBuffer::new(width, height);
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

            let environment = environment.borrow();

            canvas.set_draw_color(Color::RGB(73, 54, 87));
            for food in environment.food() {
                canvas
                    .fill_rect(Rect::from_center(
                        (food.x() as i32, food.y() as i32),
                        (food.energy() * 10.) as u32,
                        (food.energy() * 10.) as u32,
                    ))
                    .unwrap();
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
            }

            canvas.present();
        }
        slint::Image::from_rgba8(pixel_buffer)
    });

    main_window.run()
}
