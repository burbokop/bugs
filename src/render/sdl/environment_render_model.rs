use crate::{
    render::{
        sdl::{
            color_to_sdl2_rgba_color, draw_bug_chunks, draw_bug_chunks_simplified,
            draw_crc_chunks_simplified, draw_food_chunks, draw_food_chunks_simplified,
            rect_to_sdl2_rect,
        },
        Camera, EnvironmentRenderModel,
    },
    EnvironmentDisplayMode, Tool, NUKE_RADIUS,
};
use bugs_lib::{
    environment::Environment,
    food_source::FoodSourceShape,
    math::{noneg_float, Complex, DeltaAngle, Matrix, Point, Rect, Size},
    range::Range,
    utils::Float,
};
use font_loader::system_fonts;
use sdl2::{
    gfx::primitives::DrawRenderer, pixels::Color, render::TextureQuery, rwops::RWops,
    surface::Surface,
};
use slint::{Rgba8Pixel, SharedPixelBuffer};
use std::f64::consts::PI;

pub struct SdlEnvironmentRenderModel {}

impl Default for SdlEnvironmentRenderModel {
    fn default() -> Self {
        Self {}
    }
}

impl<T> EnvironmentRenderModel<T> for SdlEnvironmentRenderModel {
    fn init(&mut self, _: Size<u32>) {}

    fn render(
        &self,
        buffer: &mut SharedPixelBuffer<Rgba8Pixel>,
        view_port_rect: Rect<Float>,
        environment: &Environment<T>,
        camera: &Camera,
        selected_bug_id: &Option<usize>,
        active_tool: Tool,
        tool_action_point: Option<Point<Float>>,
        tool_action_active: bool,
        environment_display_mode: EnvironmentDisplayMode,
    ) {
        assert_eq!(
            buffer.as_bytes().len(),
            buffer.width() as usize * buffer.height() as usize * 4
        );
        let buffer_size: Size<u32> = (buffer.width(), buffer.height()).into();

        {
            let surface = Surface::from_data(
                buffer.make_mut_bytes(),
                *buffer_size.w(),
                *buffer_size.h(),
                *buffer_size.w() * 4,
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

            let view_port_adjustment_matrix = Matrix::scale(
                *buffer_size.w() as Float / view_port_rect.w(),
                *buffer_size.h() as Float / view_port_rect.h(),
            );

            let transformation = &view_port_adjustment_matrix * &camera.transformation();

            let view_port_rect_in_world_space = &(!&transformation).unwrap() * &view_port_rect;

            canvas.set_draw_color(Color::RGB(211, 250, 199));
            canvas.clear();
            let scale = transformation.average_scale();

            canvas.set_draw_color(Color::RGB(0, 255, 87));
            for source in environment.food_sources() {
                let position = &transformation * &source.position();

                match source.shape() {
                    FoodSourceShape::Rect { size } => {
                        let size = &transformation * size;
                        canvas
                            .draw_rect(sdl2::rect::Rect::from_center(
                                (*position.x() as i32, *position.y() as i32),
                                *size.w() as u32,
                                *size.h() as u32,
                            ))
                            .unwrap();
                    }
                    FoodSourceShape::Circle { radius } => {
                        canvas
                            .circle(
                                *position.x() as i16,
                                *position.y() as i16,
                                (radius.unwrap() * scale) as i16,
                                Color::RGB(0, 255, 87),
                            )
                            .unwrap();
                    }
                }
            }

            if camera.transformation().average_scale() < 0.05
                && (environment_display_mode == EnvironmentDisplayMode::Optic
                    || environment_display_mode == EnvironmentDisplayMode::Crc)
            {
                draw_food_chunks_simplified(
                    &mut canvas,
                    environment,
                    view_port_rect_in_world_space,
                    &transformation,
                );
                if environment_display_mode == EnvironmentDisplayMode::Optic {
                    draw_bug_chunks_simplified(
                        &mut canvas,
                        environment,
                        view_port_rect_in_world_space,
                        &transformation,
                    );
                } else {
                    draw_crc_chunks_simplified(
                        &mut canvas,
                        environment,
                        view_port_rect_in_world_space,
                        &transformation,
                    );
                }
            } else {
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

                match environment_display_mode {
                    EnvironmentDisplayMode::FoodChunks => draw_food_chunks(
                        &mut canvas,
                        &font,
                        environment,
                        view_port_rect_in_world_space,
                        &transformation,
                    ),
                    EnvironmentDisplayMode::BugChunks => draw_bug_chunks(
                        &mut canvas,
                        &font,
                        environment,
                        view_port_rect_in_world_space,
                        &transformation,
                    ),
                    EnvironmentDisplayMode::FoodAndBugChunks => {
                        draw_food_chunks(
                            &mut canvas,
                            &font,
                            environment,
                            view_port_rect_in_world_space,
                            &transformation,
                        );
                        draw_bug_chunks(
                            &mut canvas,
                            &font,
                            environment,
                            view_port_rect_in_world_space,
                            &transformation,
                        );
                    }
                    EnvironmentDisplayMode::Crc => {}
                    EnvironmentDisplayMode::CrcChunks => {}
                    EnvironmentDisplayMode::Optic => {}
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
                                    let rl = Complex::from_polar(
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

                            let arc = Range {
                                start: bug.rotation() - bug.vision_half_arc().unwrap(),
                                end: bug.rotation() + bug.vision_half_arc().unwrap(),
                            };

                            if bug.vision_half_arc() == DeltaAngle::from_radians(noneg_float(PI)) {
                                canvas
                                    .circle(
                                        *position.x() as i16,
                                        *position.y() as i16,
                                        (bug.vision_range().unwrap() * scale) as i16,
                                        Color::RGB(255, 183, 3),
                                    )
                                    .unwrap();
                            } else {
                                canvas
                                    .arc(
                                        *position.x() as i16,
                                        *position.y() as i16,
                                        (bug.vision_range().unwrap() * scale) as i16,
                                        arc.start.degrees() as i16,
                                        arc.end.degrees() as i16,
                                        Color::RGB(255, 183, 3),
                                    )
                                    .unwrap();

                                canvas
                                    .line(
                                        *position.x() as i16,
                                        *position.y() as i16,
                                        (*position.x()
                                            + arc.start.cos() * bug.vision_range().unwrap() * scale)
                                            as i16,
                                        (*position.y()
                                            + arc.start.sin() * bug.vision_range().unwrap() * scale)
                                            as i16,
                                        Color::RGB(255, 183, 3),
                                    )
                                    .unwrap();

                                canvas
                                    .line(
                                        *position.x() as i16,
                                        *position.y() as i16,
                                        (*position.x()
                                            + arc.end.cos() * bug.vision_range().unwrap() * scale)
                                            as i16,
                                        (*position.y()
                                            + arc.end.sin() * bug.vision_range().unwrap() * scale)
                                            as i16,
                                        Color::RGB(255, 183, 3),
                                    )
                                    .unwrap();
                            }

                            if let Some(tool_action_point) = tool_action_point {
                                let yes = if bug.vision_half_arc()
                                    == DeltaAngle::from_radians(noneg_float(PI))
                                {
                                    true
                                } else {
                                    (tool_action_point.clone() - bug.position())
                                        .angle()
                                        .is_contained_in(arc)
                                };

                                let tool_action_point = &transformation * &tool_action_point;

                                canvas
                                    .circle(
                                        *tool_action_point.x() as i16,
                                        *tool_action_point.y() as i16,
                                        10,
                                        if yes {
                                            Color::RGB(255, 0, 3)
                                        } else {
                                            Color::RGB(0, 0, 255)
                                        },
                                    )
                                    .unwrap();
                            }

                            let chunks_info: Option<(
                                Box<dyn Iterator<Item = (isize, isize)>>,
                                Color,
                            )> = match environment_display_mode {
                                EnvironmentDisplayMode::FoodChunks => Some((
                                    Box::new(environment.food_chunks_circular_traverse_iter(
                                        bug.position(),
                                        bug.vision_range(),
                                    )),
                                    Color::RGB(255, 0, 0),
                                )),
                                EnvironmentDisplayMode::BugChunks => Some((
                                    Box::new(environment.bug_chunks_circular_traverse_iter(
                                        bug.position(),
                                        bug.vision_range(),
                                    )),
                                    Color::RGB(255, 255, 0),
                                )),
                                _ => None,
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
    }
}
