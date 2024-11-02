use bugs::utils::Color;

pub(crate) fn color_to_slint_rgba_color(c: &Color) -> slint::RgbaColor<f32> {
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
