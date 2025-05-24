use bugs_lib::{
    chunk::RawChunkIndex,
    color::Color,
    environment::Environment,
    math::{map_into_range, Matrix, Point, Rect},
    utils::Float,
};
use sdl2::{
    render::{Canvas, TextureQuery},
    surface::Surface,
    ttf::Font,
};

use super::{color_to_sdl2_rgba_color, point_to_sdl2_point, rect_to_sdl2_rect};

fn draw_centered_text(
    canvas: &mut Canvas<Surface>,
    font: &Font,
    text: &str,
    center: Point<Float>,
    color: Color,
) {
    if text.len() > 0 {
        let texture_creator = canvas.texture_creator();
        let surface = font
            .render(text)
            .blended(color_to_sdl2_rgba_color(&color))
            .map_err(|e| e.to_string())
            .unwrap();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())
            .unwrap();

        let TextureQuery { width, height, .. } = texture.query();
        canvas
            .copy(
                &texture,
                None,
                sdl2::rect::Rect::from_center(point_to_sdl2_point(&center), width, height),
            )
            .unwrap();
    }
}

fn draw_chunk(
    canvas: &mut Canvas<Surface>,
    font: &Font,
    rect: &Rect<Float>,
    ocupants_count: usize,
    color: Color,
) {
    let sdl_color = color_to_sdl2_rgba_color(&color);
    let size_of_x = font.size_of_char('X').unwrap();
    if size_of_x.0 as Float > *rect.w() || size_of_x.1 as Float > *rect.h() {
        let max_ocupants_count = 8;
        canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
        canvas.set_draw_color(if ocupants_count >= max_ocupants_count {
            sdl_color
        } else {
            sdl2::pixels::Color::RGBA(
                sdl_color.r,
                sdl_color.g,
                sdl_color.b,
                map_into_range(
                    ocupants_count as Float,
                    0. ..max_ocupants_count as Float,
                    (sdl_color.a as Float / 16.)..sdl_color.a as Float,
                ) as u8,
            )
        });

        canvas
            .fill_rect(rect_to_sdl2_rect(&rect.clone().extended((1., 1.).into())))
            .unwrap();
    } else {
        canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
        canvas.set_draw_color(if ocupants_count > 0 {
            sdl_color
        } else {
            sdl2::pixels::Color::RGBA(sdl_color.r, sdl_color.g, sdl_color.b, sdl_color.a / 4)
        });
        canvas
            .draw_rect(rect_to_sdl2_rect(&rect.clone().extended((1., 1.).into())))
            .unwrap();
        if ocupants_count > 0 {
            draw_centered_text(
                canvas,
                &font,
                &format!("{}", ocupants_count),
                rect.center(),
                color,
            );
        }
    }
}

fn draw_chunk_simplified(
    canvas: &mut Canvas<Surface>,
    rect: &Rect<Float>,
    ocupants_count: usize,
    color: Color,
) {
    let sdl_color = color_to_sdl2_rgba_color(&color);
    if ocupants_count > 0 {
        let max_ocupants_count = 8;
        canvas.set_blend_mode(sdl2::render::BlendMode::None);
        canvas.set_draw_color(if ocupants_count >= max_ocupants_count {
            sdl_color
        } else {
            sdl2::pixels::Color::RGBA(
                sdl_color.r,
                sdl_color.g,
                sdl_color.b,
                map_into_range(
                    ocupants_count as Float,
                    0. ..max_ocupants_count as Float,
                    (sdl_color.a as Float / 16.)..sdl_color.a as Float,
                ) as u8,
            )
        });

        canvas
            .fill_rect(rect_to_sdl2_rect(&rect.clone().extended((1., 1.).into())))
            .unwrap();
    }
}

pub(crate) fn draw_food_chunks<T>(
    canvas: &mut Canvas<Surface>,
    font: &Font,
    environment: &Environment<T>,
    view_port_rect_in_world_space: Rect<Float>,
    transformation: &Matrix<Float>,
) {
    for (index, ocupants) in environment.food_chunks_in_area(view_port_rect_in_world_space) {
        let index: RawChunkIndex = index.into();
        let rect = transformation
            * &Rect::from((
                index.x() as Float * 256.,
                index.y() as Float * 256.,
                256.,
                256.,
            ));
        draw_chunk(
            canvas,
            font,
            &rect,
            ocupants.len(),
            Color {
                a: 1.,
                r: 1.,
                g: 0.4296875,
                b: 0.6328125,
            },
        )
    }
}

pub(crate) fn draw_bug_chunks<T>(
    canvas: &mut Canvas<Surface>,
    font: &Font,
    environment: &Environment<T>,
    view_port_rect_in_world_space: Rect<Float>,
    transformation: &Matrix<Float>,
) {
    for (index, ocupants) in environment.bug_chunks_in_area(view_port_rect_in_world_space) {
        let index: RawChunkIndex = index.into();
        let rect = transformation
            * &Rect::from((
                index.x() as Float * 256.,
                index.y() as Float * 256.,
                256.,
                256.,
            ));
        draw_chunk(
            canvas,
            font,
            &rect,
            ocupants.len(),
            Color {
                a: 1.,
                r: 0.,
                g: 0.,
                b: 1.,
            },
        )
    }
}

pub(crate) fn draw_food_chunks_simplified<T>(
    canvas: &mut Canvas<Surface>,
    environment: &Environment<T>,
    view_port_rect_in_world_space: Rect<Float>,
    transformation: &Matrix<Float>,
) {
    for (index, ocupants) in environment.food_chunks_in_area(view_port_rect_in_world_space) {
        let index: RawChunkIndex = index.into();
        let rect = transformation
            * &Rect::from((
                index.x() as Float * 256.,
                index.y() as Float * 256.,
                256.,
                256.,
            ));
        draw_chunk_simplified(
            canvas,
            &rect,
            ocupants.len(),
            Color {
                a: 1.,
                r: 1.,
                g: 0.4296875,
                b: 0.6328125,
            },
        )
    }
}

pub(crate) fn draw_bug_chunks_simplified<T>(
    canvas: &mut Canvas<Surface>,
    environment: &Environment<T>,
    view_port_rect_in_world_space: Rect<Float>,
    transformation: &Matrix<Float>,
) {
    for (index, ocupants) in environment.bug_chunks_in_area(view_port_rect_in_world_space) {
        let index: RawChunkIndex = index.into();
        let rect = transformation
            * &Rect::from((
                index.x() as Float * 256.,
                index.y() as Float * 256.,
                256.,
                256.,
            ));
        draw_chunk_simplified(
            canvas,
            &rect,
            ocupants.len(),
            Color {
                a: 1.,
                r: 0.,
                g: 0.,
                b: 1.,
            },
        )
    }
}

pub(crate) fn draw_crc_chunks_simplified<T>(
    canvas: &mut Canvas<Surface>,
    environment: &Environment<T>,
    view_port_rect_in_world_space: Rect<Float>,
    transformation: &Matrix<Float>,
) {
    let x0 = environment.bug_chunks().filter_map(|(_, ocupants)| {
        if ocupants.len() > 0 {
            Some(
                ocupants.iter().map(|x| x.borrow().crc()).sum::<Float>()
                    / (ocupants.len() as Float),
            )
        } else {
            None
        }
    });

    let x1 = environment.bug_chunks().filter_map(|(_, ocupants)| {
        if ocupants.len() > 0 {
            Some(
                ocupants.iter().map(|x| x.borrow().crc()).sum::<Float>()
                    / (ocupants.len() as Float),
            )
        } else {
            None
        }
    });

    if let (Some(min), Some(max)) = (
        x0.min_by(|a, b| a.partial_cmp(b).unwrap()),
        x1.max_by(|a, b| a.partial_cmp(b).unwrap()),
    ) {
        for (index, ocupants) in environment.bug_chunks_in_area(view_port_rect_in_world_space) {
            let index: RawChunkIndex = index.into();
            let rect = transformation
                * &Rect::from((
                    index.x() as Float * 256.,
                    index.y() as Float * 256.,
                    256.,
                    256.,
                ));

            if ocupants.len() > 0 {
                let average_crc = ocupants.iter().map(|x| x.borrow().crc()).sum::<Float>()
                    / (ocupants.len() as Float);
                let coef = map_into_range(average_crc, min..max, 0. ..1.);

                let color = Color::from_hsv(1., coef * (2. / 3.), 1., 1.);

                canvas.set_blend_mode(sdl2::render::BlendMode::None);
                canvas.set_draw_color(color_to_sdl2_rgba_color(&color));

                canvas
                    .fill_rect(rect_to_sdl2_rect(&rect.clone().extended((1., 1.).into())))
                    .unwrap();
            }
        }
    }
}
