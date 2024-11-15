use super::Camera;
use crate::{
    app_utils::{color_to_sdl2_rgba_color, point_to_sdl2_point, rect_to_sdl2_rect},
    Tool, NUKE_RADIUS,
};
use bugs_lib::{
    environment::Environment,
    math::{map_into_range, Complex, DeltaAngle, NoNeg, Point, Rect, Size},
    utils::Float,
};
use font_loader::system_fonts;
use sdl2::{
    gfx::primitives::DrawRenderer,
    pixels::Color,
    render::{Canvas, TextureQuery},
    rwops::RWops,
    surface::Surface,
    ttf::Font,
};
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};
use std::f64::consts::PI;

#[derive(Debug, Clone)]
pub(crate) enum ChunksDisplayMode {
    FoodChunks,
    BugChunks,
    Both,
    None,
}

impl ChunksDisplayMode {
    pub(crate) fn rotated(self) -> Self {
        match self {
            ChunksDisplayMode::FoodChunks => ChunksDisplayMode::BugChunks,
            ChunksDisplayMode::BugChunks => ChunksDisplayMode::Both,
            ChunksDisplayMode::Both => ChunksDisplayMode::None,
            ChunksDisplayMode::None => ChunksDisplayMode::FoodChunks,
        }
    }
}

pub struct EnvironmentRenderModel {
    buffer: SharedPixelBuffer<Rgba8Pixel>,
}

impl Default for EnvironmentRenderModel {
    fn default() -> Self {
        Self {
            buffer: SharedPixelBuffer::new(0, 0),
        }
    }
}

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
            .blended(color)
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
    let size_of_x = font.size_of_char('X').unwrap();
    if size_of_x.0 as Float > *rect.w() || size_of_x.1 as Float > *rect.h() {
        let max_ocupants_count = 8;
        canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
        canvas.set_draw_color(if ocupants_count >= max_ocupants_count {
            color
        } else {
            Color::RGBA(
                color.r,
                color.g,
                color.b,
                map_into_range(
                    ocupants_count as Float,
                    0. ..max_ocupants_count as Float,
                    (color.a as Float / 16.)..color.a as Float,
                ) as u8,
            )
        });

        canvas
            .fill_rect(rect_to_sdl2_rect(&rect.clone().extended((1., 1.).into())))
            .unwrap();
    } else {
        canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
        canvas.set_draw_color(if ocupants_count > 0 {
            color
        } else {
            Color::RGBA(color.r, color.g, color.b, color.a / 4)
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

impl EnvironmentRenderModel {
    pub fn render<T>(
        &mut self,
        environment: &Environment<T>,
        camera: &Camera,
        selected_bug_id: &Option<usize>,
        active_tool: Tool,
        tool_action_point: Option<Point<Float>>,
        tool_action_active: bool,
        chunks_display_mode: ChunksDisplayMode,
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

            let mut property = system_fonts::FontPropertyBuilder::new().monospace().build();
            let sysfonts = system_fonts::query_specific(&mut property);
            let font_bytes = system_fonts::get(
                &system_fonts::FontPropertyBuilder::new()
                    .family(sysfonts.first().unwrap())
                    .build(),
            )
            .unwrap();
            let rwops = RWops::from_bytes(&font_bytes.0[..]).unwrap();

            let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string()).unwrap();

            let font = ttf_context.load_font_from_rwops(rwops, 16).unwrap();

            let transformation = camera.transformation();

            canvas.set_draw_color(Color::RGB(211, 250, 199));
            canvas.clear();

            canvas.set_draw_color(Color::RGB(0, 255, 87));
            for source in environment.food_sources() {
                let position = &transformation * &source.position();
                let size = &transformation * &source.size();

                canvas
                    .draw_rect(sdl2::rect::Rect::from_center(
                        (*position.x() as i32, *position.y() as i32),
                        *size.w() as u32,
                        *size.h() as u32,
                    ))
                    .unwrap();
            }

            let view_port_rect: Rect<_> = (
                0.,
                0.,
                requested_canvas_width as Float,
                requested_canvas_height as Float,
            )
                .into();

            for food in environment.food() {
                let position = &transformation * &food.position();
                let size = &transformation
                    * &Size::from((food.radius().unwrap() * 2., food.radius().unwrap() * 2.));

                let aabb = Rect::from_center(position, size);

                if view_port_rect.contains(&aabb) || view_port_rect.instersects(&aabb) {
                    canvas
                        .filled_circle(
                            *position.x() as i16,
                            *position.y() as i16,
                            (size.w().max(*size.h()) / 2.) as i16,
                            Color::RGB(73, 54, 87),
                        )
                        .unwrap();
                }
            }
            let scale = Float::max(*transformation.scale_x(), *transformation.scale_y());

            match chunks_display_mode {
                ChunksDisplayMode::FoodChunks => {
                    for (index, ocupants_count) in environment.food_chunks() {
                        let rect = &transformation
                            * &Rect::from((
                                index.x() as Float * 256.,
                                index.y() as Float * 256.,
                                256.,
                                256.,
                            ));
                        if view_port_rect.contains(&rect) || view_port_rect.instersects(&rect) {
                            draw_chunk(
                                &mut canvas,
                                &font,
                                &rect,
                                ocupants_count,
                                Color::RGB(255, 110, 162),
                            )
                        }
                    }
                }
                ChunksDisplayMode::BugChunks => {
                    for (index, ocupants_count) in environment.bug_chunks() {
                        let rect = &transformation
                            * &Rect::from((
                                index.x() as Float * 256.,
                                index.y() as Float * 256.,
                                256.,
                                256.,
                            ));
                        if view_port_rect.contains(&rect) || view_port_rect.instersects(&rect) {
                            draw_chunk(
                                &mut canvas,
                                &font,
                                &rect,
                                ocupants_count,
                                Color::RGB(0, 0, 255),
                            )
                        }
                    }
                }
                ChunksDisplayMode::Both => {
                    for (index, ocupants_count) in environment.food_chunks() {
                        let rect = &transformation
                            * &Rect::from((
                                index.x() as Float * 256.,
                                index.y() as Float * 256.,
                                256.,
                                256.,
                            ));
                        if view_port_rect.contains(&rect) || view_port_rect.instersects(&rect) {
                            draw_chunk(
                                &mut canvas,
                                &font,
                                &rect,
                                ocupants_count,
                                Color::RGB(255, 110, 162),
                            )
                        }
                    }
                    for (index, ocupants_count) in environment.bug_chunks() {
                        let rect = &transformation
                            * &Rect::from((
                                index.x() as Float * 256.,
                                index.y() as Float * 256.,
                                256.,
                                256.,
                            ));
                        if view_port_rect.contains(&rect) || view_port_rect.instersects(&rect) {
                            draw_chunk(
                                &mut canvas,
                                &font,
                                &rect,
                                ocupants_count,
                                Color::RGB(0, 0, 255),
                            )
                        }
                    }
                }
                ChunksDisplayMode::None => {}
            }

            canvas.set_draw_color(Color::RGB(255, 183, 195));
            for bug in environment.bugs() {
                let position = &transformation * &bug.position();

                let rotation = complexible::complex_numbers::ComplexNumber::from_polar(
                    1.,
                    complexible::complex_numbers::Angle::from_radians(bug.rotation().radians()),
                );
                let pos = complexible::complex_numbers::ComplexNumber::from_cartesian(
                    *position.x(),
                    *position.y(),
                );

                let radius = bug.eat_range().unwrap() * scale;

                let size = 5. * scale * bug.size().unwrap();

                let aabb = Rect::from_center(position, (radius * 2., radius * 2.).into());

                if view_port_rect.contains(&aabb)
                    || view_port_rect.instersects(&aabb)
                    || Some(bug.id()) == *selected_bug_id
                {
                    let p0 = complexible::complex_numbers::ComplexNumber::from_cartesian(
                        4. * size,
                        0. * size,
                    );
                    let p1 = complexible::complex_numbers::ComplexNumber::from_cartesian(
                        -1. * size,
                        -1. * size,
                    );
                    let p2 = complexible::complex_numbers::ComplexNumber::from_cartesian(
                        -1. * size,
                        1. * size,
                    );

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
                        if let Some(log) = bug.last_brain_log() {
                            {
                                let rl = Complex::from_polar(radius, bug.rotation());
                                canvas
                                    .line(
                                        *position.x() as i16,
                                        *position.y() as i16,
                                        *position.x() as i16 + *rl.real() as i16,
                                        *position.y() as i16 + *rl.imag() as i16,
                                        Color::RGB(255, 0, 0),
                                    )
                                    .unwrap();
                            }

                            if let Some(nearest_food) = &log.input.nearest_food {
                                let rl = Complex::from_polar(radius, nearest_food.direction);
                                canvas
                                    .line(
                                        *position.x() as i16,
                                        *position.y() as i16,
                                        *position.x() as i16 + *rl.real() as i16,
                                        *position.y() as i16 + *rl.imag() as i16,
                                        Color::RGB(0, 255, 0),
                                    )
                                    .unwrap();
                            }

                            {
                                let rl =
                                    Complex::from_polar(
                                        radius,
                                        bug.rotation()
                                            + log.output.relative_desired_rotation
                                            + DeltaAngle::from_radians(
                                                if log.output.velocity > 0. { 0. } else { PI },
                                            ),
                                    );
                                canvas
                                    .line(
                                        *position.x() as i16,
                                        *position.y() as i16,
                                        *position.x() as i16 + *rl.real() as i16,
                                        *position.y() as i16 + *rl.imag() as i16,
                                        Color::RGB(255, 183, 195),
                                    )
                                    .unwrap();
                            }
                        }

                        canvas
                            .circle(
                                *position.x() as i16,
                                *position.y() as i16,
                                radius as i16,
                                Color::RGB(255, 183, 195),
                            )
                            .unwrap();

                        canvas
                            .circle(
                                *position.x() as i16,
                                *position.y() as i16,
                                (bug.vision_range().unwrap() * scale) as i16,
                                Color::RGB(255, 183, 3),
                            )
                            .unwrap();

                        let chunks_info: Option<(Box<dyn Iterator<Item = (isize, isize)>>, Color)> =
                            match chunks_display_mode {
                                ChunksDisplayMode::FoodChunks => Some((
                                    Box::new(environment.food_chunks_circular_traverse_iter(
                                        bug.position(),
                                        bug.vision_range(),
                                    )),
                                    Color::RGB(255, 0, 0),
                                )),
                                ChunksDisplayMode::BugChunks => Some((
                                    Box::new(environment.bug_chunks_circular_traverse_iter(
                                        bug.position(),
                                        bug.vision_range(),
                                    )),
                                    Color::RGB(255, 255, 0),
                                )),
                                ChunksDisplayMode::Both => None,
                                ChunksDisplayMode::None => None,
                            };

                        if let Some((chunks_iter, chunks_color)) = chunks_info {
                            for (i, (x, y)) in chunks_iter.enumerate() {
                                let rect = &transformation
                                    * &Rect::from((
                                        x as Float * 256.,
                                        y as Float * 256.,
                                        256.,
                                        256.,
                                    ));
                                canvas.set_draw_color(chunks_color);
                                canvas.draw_rect(rect_to_sdl2_rect(&rect)).unwrap();

                                let texture_creator = canvas.texture_creator();
                                let surface = font
                                    .render(&format!("{}", i))
                                    .blended(chunks_color)
                                    .map_err(|e| e.to_string())
                                    .unwrap();
                                let texture = texture_creator
                                    .create_texture_from_surface(&surface)
                                    .map_err(|e| e.to_string())
                                    .unwrap();

                                let TextureQuery { width, height, .. } = texture.query();
                                canvas
                                    .copy(&texture, None, rect_to_sdl2_rect(&(rect / 2.)))
                                    .unwrap();
                            }
                        }
                    }
                }
            }

            if let Some(tool_action_point) = tool_action_point {
                let tool_action_point = &transformation * &tool_action_point;
                if active_tool == Tool::Nuke {
                    if tool_action_active {
                        canvas
                            .filled_circle(
                                *tool_action_point.x() as i16,
                                *tool_action_point.y() as i16,
                                (NUKE_RADIUS.unwrap() * scale) as i16,
                                Color::RGBA(255, 183, 3, 64),
                            )
                            .unwrap()
                    } else {
                        canvas
                            .circle(
                                *tool_action_point.x() as i16,
                                *tool_action_point.y() as i16,
                                (NUKE_RADIUS.unwrap() * scale) as i16,
                                Color::RGB(255, 183, 3),
                            )
                            .unwrap()
                    }
                } else if active_tool == Tool::Food || active_tool == Tool::SpawnBug {
                    canvas
                        .filled_circle(
                            *tool_action_point.x() as i16,
                            *tool_action_point.y() as i16,
                            5,
                            if active_tool == Tool::Food {
                                Color::RGB(183, 255, 3)
                            } else {
                                Color::RGB(183, 3, 255)
                            },
                        )
                        .unwrap()
                }
            }

            canvas.present();
        }
        slint::Image::from_rgba8(self.buffer.clone())
    }
}
