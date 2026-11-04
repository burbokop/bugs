use bugs_lib::{brain::Brain, bug::BrainLog, utils::Float};
use font_loader::system_fonts;
use sdl2::{
    gfx::primitives::DrawRenderer as _,
    pixels::Color,
    rect::{Point, Rect},
    render::{Canvas, TextureQuery},
    rwops::RWops,
    surface::Surface,
    ttf::Font,
};
use simple_neural_net::PerceptronLayer;
use slint::{Rgba8Pixel, SharedPixelBuffer};

use crate::render::BrainRenderModel;

/// Maps layout coordinates (defined in unscaled units) into buffer pixels. Scaling the geometry
/// itself, instead of using `Canvas::set_scale` (which just enlarges pixels), keeps lines,
/// circles and text crisp at any canvas size.
#[derive(Clone, Copy)]
struct Transform {
    scale: f32,
    offset: (f32, f32),
}

impl Transform {
    /// Largest scale at which the layout rect `min..max` fits into `size`, centered.
    fn fit(min: (f32, f32), max: (f32, f32), size: (u32, u32)) -> Self {
        let content_size = (max.0 - min.0, max.1 - min.1);
        let scale = f32::min(
            size.0 as f32 / content_size.0,
            size.1 as f32 / content_size.1,
        );
        let offset = (
            (size.0 as f32 - content_size.0 * scale) / 2. - min.0 * scale,
            (size.1 as f32 - content_size.1 * scale) / 2. - min.1 * scale,
        );
        Self { scale, offset }
    }

    fn point(&self, p: (i32, i32)) -> (i32, i32) {
        (
            (p.0 as f32 * self.scale + self.offset.0).round() as i32,
            (p.1 as f32 * self.scale + self.offset.1).round() as i32,
        )
    }

    fn length(&self, l: f32) -> f32 {
        l * self.scale
    }
}

fn draw_node(
    canvas: &mut Canvas<Surface>,
    point: (i32, i32),
    radius: f32,
    filled: bool,
    color: Color,
) {
    let radius = radius.round().max(1.) as i16;
    if filled {
        canvas
            .filled_circle(point.0 as i16, point.1 as i16, radius, color)
            .unwrap();
    }
    // Anti-aliased outline smooths the edge of the filled circle too
    canvas
        .aa_circle(point.0 as i16, point.1 as i16, radius, color)
        .unwrap();
}

fn draw_layer_activations<const SIZE: usize>(
    canvas: &mut Canvas<Surface>,
    font: &Font,
    layer: [Float; SIZE],
    max_width: usize,
    selected_node: Option<(usize, usize)>,
    layer_index: isize,
    x: i32,
    transform: Transform,
) {
    for (i, a) in layer.iter().enumerate() {
        let off = (max_width - layer.len()) / 2;
        let point = transform.point((x, (40 + 40 * (off + i)) as i32));

        let selected = selected_node
            .map(|s| s.0 as isize == layer_index && s.1 == i)
            .unwrap_or(true);

        let node_color = Color::RGB(165, 136, 171);
        let text_color = Color::RGB(47, 72, 88);
        let node_radius = transform.length(16.);

        draw_node(canvas, point, node_radius, selected, node_color);

        let texture_creator = canvas.texture_creator();
        let surface = font
            .render(&format!("{:.2}", a))
            .blended(text_color)
            .map_err(|e| e.to_string())
            .unwrap();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())
            .unwrap();

        let TextureQuery { width, height, .. } = texture.query();
        canvas
            .copy(&texture, None, Rect::from_center(point, width, height))
            .unwrap();
    }
}

fn draw_layer_text<const SIZE: usize>(
    canvas: &mut Canvas<Surface>,
    font: &Font,
    layer: [&str; SIZE],
    max_width: usize,
    selected_node: Option<(usize, usize)>,
    layer_index: isize,
    x: i32,
    transform: Transform,
) {
    for (i, a) in layer.iter().enumerate() {
        let off = (max_width - layer.len()) / 2;
        let point = transform.point((x, (40 + 40 * (off + i)) as i32));

        let selected = selected_node
            .map(|s| s.0 as isize == layer_index && s.1 == i)
            .unwrap_or(true);

        let node_color = if selected {
            Color::RGB(255, 183, 3)
        } else {
            Color::RGB(0, 0, 0)
        };
        let text_color = Color::RGB(47, 72, 88);
        let node_radius = transform.length(12.);

        draw_node(canvas, point, node_radius, selected, node_color);

        let texture_creator = canvas.texture_creator();
        let surface = font
            .render(a)
            .blended(text_color)
            .map_err(|e| e.to_string())
            .unwrap();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())
            .unwrap();

        let TextureQuery { width, height, .. } = texture.query();
        canvas
            .copy(&texture, None, Rect::from_center(point, width, height))
            .unwrap();
    }
}

fn draw_connections<const INPUT_SIZE: usize, const OUTPUT_SIZE: usize>(
    canvas: &mut Canvas<Surface>,
    font: &Font,
    layer: &PerceptronLayer<Float, INPUT_SIZE, OUTPUT_SIZE>,
    max_width: usize,
    selected_node: Option<(usize, usize)>,
    layer_index: usize,
    x0: i32,
    x1: i32,
    transform: Transform,
) {
    let max_weight = layer
        .perceptrons()
        .iter()
        .map(|p| p.weights().iter())
        .flatten()
        .cloned()
        .reduce(|a, b| a.abs().max(b.abs()))
        .unwrap()
        .abs();

    let connection_color = Color::RGB(165, 136, 171);
    let negative_connection_color = Color::RGB(227, 10, 125);
    let text_color = Color::RGB(47, 72, 88);
    let bias_text_color = Color::RGB(249, 248, 113);

    for j in 0..OUTPUT_SIZE {
        let selected = selected_node
            .map(|s| s.0 == layer_index && s.1 == j)
            .unwrap_or(true);

        if !selected {
            continue;
        }

        for i in 0..INPUT_SIZE {
            let w = layer.perceptrons()[j].weights()[i];

            let off_i = (max_width - INPUT_SIZE) / 2;
            let off_j = (max_width - OUTPUT_SIZE) / 2;
            let point0 = transform.point((x0, (40 + 40 * (off_i + i)) as i32));
            let point1 = transform.point((x1, (40 + 40 * (off_j + j)) as i32));

            canvas
                .aa_line(
                    point0.0 as i16,
                    point0.1 as i16,
                    point1.0 as i16,
                    point1.1 as i16,
                    if w >= 0. {
                        Color::RGBA(
                            connection_color.r,
                            connection_color.g,
                            connection_color.b,
                            (w.abs() / max_weight * 255.) as u8,
                        )
                    } else {
                        Color::RGBA(
                            negative_connection_color.r,
                            negative_connection_color.g,
                            negative_connection_color.b,
                            (w.abs() / max_weight * 255.) as u8,
                        )
                    },
                )
                .unwrap();

            if selected_node.is_some() {
                let center = (Point::from(point0) + Point::from(point1)) / 2;

                let texture_creator = canvas.texture_creator();
                let surface = font
                    .render(&format!("{:.2}", w))
                    .blended(text_color)
                    .map_err(|e| e.to_string())
                    .unwrap();
                let texture = texture_creator
                    .create_texture_from_surface(&surface)
                    .map_err(|e| e.to_string())
                    .unwrap();

                let TextureQuery { width, height, .. } = texture.query();
                canvas
                    .copy(&texture, None, Rect::from_center(center, width, height))
                    .unwrap();
            }
        }

        if selected_node.is_some() {
            let off_i = (max_width - INPUT_SIZE) / 2;
            let off_j = (max_width - OUTPUT_SIZE) / 2;
            let point0 = transform.point((x0, (40 + 40 * (off_i + INPUT_SIZE)) as i32));
            let point1 = transform.point((x1, (40 + 40 * (off_j + j)) as i32));
            let center = (Point::from(point0) + Point::from(point1)) / 2;

            let texture_creator = canvas.texture_creator();
            let surface = font
                .render(&format!("{:.2}", layer.perceptrons()[j].bias()))
                .blended(bias_text_color)
                .map_err(|e| e.to_string())
                .unwrap();
            let texture = texture_creator
                .create_texture_from_surface(&surface)
                .map_err(|e| e.to_string())
                .unwrap();

            let TextureQuery { width, height, .. } = texture.query();
            canvas
                .copy(&texture, None, Rect::from_center(center, width, height))
                .unwrap();
        }
    }
}

pub struct SdlBrainRenderModel {}

impl Default for SdlBrainRenderModel {
    fn default() -> Self {
        Self {}
    }
}

impl BrainRenderModel for SdlBrainRenderModel {
    fn render(
        &self,
        buffer: &mut SharedPixelBuffer<Rgba8Pixel>,
        brain: &Brain,
        log: &BrainLog,
        selected_node: Option<(usize, usize)>,
    ) {
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

        let buffer_size = (buffer.width(), buffer.height());

        let (a0, a1, a2) = log.activations;
        let max_width = a0.len().max(a1.len()).max(a2.len());

        // Bounding box of the layout in unscaled units: node columns span x = 20..300 with
        // node radius up to 16 (text columns at the edges use radius 12), node rows are at
        // y = 40 + 40 * i for i in 0..max_width.
        const PADDING: f32 = 8.;
        let transform = Transform::fit(
            (20. - 12. - PADDING, 40. - 16. - PADDING),
            (300. + 12. + PADDING, 40. * max_width as f32 + 16. + PADDING),
            buffer_size,
        );

        // Rasterize the font at the final size instead of stretching 12pt glyphs
        let font_size = transform.length(12.).round().max(1.) as u16;
        let font = ttf_context.load_font_from_rwops(rwops, font_size).unwrap();
        assert_eq!(
            buffer.as_bytes().len(),
            buffer.width() as usize * buffer.height() as usize * 4
        );

        {
            let surface = Surface::from_data(
                buffer.make_mut_bytes(),
                buffer_size.0,
                buffer_size.1,
                buffer_size.0 * 4,
                sdl2::pixels::PixelFormatEnum::RGBA32,
            )
            .unwrap();

            let mut canvas = surface.into_canvas().unwrap();

            canvas.set_draw_color(Color::RGB(255, 183, 195));
            canvas.clear();

            draw_connections::<16, 8>(
                &mut canvas,
                &font,
                &brain.layers().0,
                max_width,
                selected_node,
                0,
                20 + 40,
                20 + 40 + 100,
                transform,
            );

            draw_connections::<8, 8>(
                &mut canvas,
                &font,
                &brain.layers().1,
                max_width,
                selected_node,
                1,
                20 + 40 + 100,
                20 + 40 + 200,
                transform,
            );

            draw_layer_activations(
                &mut canvas,
                &font,
                a0,
                max_width,
                selected_node,
                -1,
                20 + 40,
                transform,
            );

            draw_layer_activations(
                &mut canvas,
                &font,
                a1,
                max_width,
                selected_node,
                0,
                20 + 40 + 100,
                transform,
            );

            draw_layer_activations(
                &mut canvas,
                &font,
                a2,
                max_width,
                selected_node,
                1,
                20 + 40 + 200,
                transform,
            );

            draw_layer_text(
                &mut canvas,
                &font,
                [
                    "E/C", "FP/", "R-F", "FS", "A", "BP/", "R-B", "a", "r", "g", "b", "BS", "B/C",
                    "R0", "R1", "R2",
                ],
                max_width,
                selected_node,
                -1,
                20,
                transform,
            );

            draw_layer_text(
                &mut canvas,
                &font,
                ["V", "R", "RV", "BR", "R0", "R1", "R2", "R3"],
                max_width,
                selected_node,
                1,
                20 + 40 + 200 + 40,
                transform,
            );

            canvas.present();
        }
    }
}
