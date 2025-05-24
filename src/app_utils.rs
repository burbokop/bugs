use bugs_lib::color::Color;
use slint::Rgba8Pixel;

pub(crate) fn color_to_slint_rgba_f32_color(c: &Color) -> slint::RgbaColor<f32> {
    slint::RgbaColor {
        alpha: c.a as f32,
        red: c.r as f32,
        green: c.g as f32,
        blue: c.b as f32,
    }
}

pub(crate) fn color_to_slint_rgba8_color(c: &Color) -> Rgba8Pixel {
    Rgba8Pixel {
        r: (c.r * 255.) as u8,
        g: (c.g * 255.) as u8,
        b: (c.b * 255.) as u8,
        a: (c.a * 255.) as u8,
    }
}
