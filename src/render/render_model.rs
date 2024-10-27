use super::Camera;
use crate::{
    bug,
    environment::Environment,
    math::Size,
    utils::{color_to_sdl2_rgba_color, Float},
};
use complexible::complex_numbers::{Angle, ComplexNumber};
use sdl2::{gfx::primitives::DrawRenderer as _, pixels::Color, rect::Rect, surface::Surface};
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};

pub struct RenderModel {
    buffer: SharedPixelBuffer<Rgba8Pixel>,
}

impl Default for RenderModel {
    fn default() -> Self {
        Self {
            buffer: SharedPixelBuffer::new(0, 0),
        }
    }
}

impl RenderModel {
    pub fn render(
        &mut self,
        environment: &Environment,
        camera: &Camera,
        selected_bug_id: &Option<usize>,
        requested_canvas_width: u32,
        requested_canvas_height: u32,
    ) -> Image {
        if self.buffer.width() != requested_canvas_width
            || self.buffer.height() != requested_canvas_height
        {
            self.buffer = SharedPixelBuffer::new(requested_canvas_width, requested_canvas_height);
        }

        let buffer_size = (self.buffer.width(), self.buffer.height());
        assert_eq!(
            self.buffer.as_bytes().len(),
            self.buffer.width() as usize * self.buffer.height() as usize * 4
        );

        {
            let surface = Surface::from_data(
                self.buffer.make_mut_bytes(),
                buffer_size.0,
                buffer_size.1,
                buffer_size.0 * 4,
                sdl2::pixels::PixelFormatEnum::RGBA32,
            )
            .unwrap();

            let mut canvas = surface.into_canvas().unwrap();

            let transformation = camera.transformation();

            canvas.set_draw_color(Color::RGB(211, 250, 199));
            canvas.clear();

            canvas.set_draw_color(Color::RGB(73, 54, 87));
            for food in environment.food() {
                let position = &transformation * &food.position();
                let size =
                    &transformation * &Size::from((food.energy() * 10., food.energy() * 10.));

                canvas
                    .fill_rect(Rect::from_center(
                        (*position.x() as i32, *position.y() as i32),
                        *size.w() as u32,
                        *size.h() as u32,
                    ))
                    .unwrap();
            }

            canvas.set_draw_color(Color::RGB(255, 183, 195));
            for bug in environment.bugs() {
                let position = &transformation * &bug.position();

                let rotation = ComplexNumber::from_polar(1., Angle::from_radians(bug.rotation()));
                let pos = ComplexNumber::from_cartesian(*position.x(), *position.y());

                let scale = Float::max(*transformation.scale_x(), *transformation.scale_y());

                let size = 5. * scale;

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
                        color_to_sdl2_rgba_color(bug.color()),
                    )
                    .unwrap();

                canvas
                    .trigon(
                        pp0.real() as i16,
                        pp0.imag() as i16,
                        pp1.real() as i16,
                        pp1.imag() as i16,
                        pp2.real() as i16,
                        pp2.imag() as i16,
                        Color::RGB(255, 183, 195),
                    )
                    .unwrap();

                if &Some(bug.id()) == selected_bug_id {
                    canvas
                        .circle(
                            *position.x() as i16,
                            *position.y() as i16,
                            (bug::EAT_FOOD_MAX_PROXIMITY * scale) as i16,
                            Color::RGB(255, 183, 195),
                        )
                        .unwrap();
                }
            }

            canvas.present();
        }
        slint::Image::from_rgba8(self.buffer.clone())
    }
}
