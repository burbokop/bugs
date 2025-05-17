use bugs_lib::{
    math::{Point, Rect},
    utils::{Color, Float},
};
use slint::Rgba8Pixel;

pub(crate) fn color_to_slint_rgba_f32_color(c: &Color) -> slint::RgbaColor<f32> {
    slint::RgbaColor {
        alpha: c.a as f32,
        red: c.r as f32,
        green: c.g as f32,
        blue: c.b as f32,
    }
}

pub(crate) fn color_to_sdl2_rgba_color(c: &Color) -> sdl2::pixels::Color {
    sdl2::pixels::Color::RGBA(
        (c.r * 255.) as u8,
        (c.g * 255.) as u8,
        (c.b * 255.) as u8,
        (c.a * 255.) as u8,
    )
}

pub(crate) fn color_to_slint_rgba8_color(c: &Color) -> Rgba8Pixel {
    Rgba8Pixel {
        r: (c.r * 255.) as u8,
        g: (c.g * 255.) as u8,
        b: (c.b * 255.) as u8,
        a: (c.a * 255.) as u8,
    }
}

pub(crate) fn rect_to_sdl2_rect(c: &Rect<Float>) -> sdl2::rect::Rect {
    (*c.x() as i32, *c.y() as i32, *c.w() as u32, *c.h() as u32).into()
}

pub(crate) fn point_to_sdl2_point(c: &Point<Float>) -> sdl2::rect::Point {
    (*c.x() as i32, *c.y() as i32).into()
}
