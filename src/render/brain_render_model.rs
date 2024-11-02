use bugs::{brain::Brain, bug::BrainLog, utils::Float};
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
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};

pub struct BrainRenderModel {
    buffer: SharedPixelBuffer<Rgba8Pixel>,
}

impl Default for BrainRenderModel {
    fn default() -> Self {
        Self {
            buffer: SharedPixelBuffer::new(0, 0),
        }
    }
}

fn draw_layer_activations<const SIZE: usize>(
    canvas: &mut Canvas<Surface>,
    font: &Font,
    layer: [Float; SIZE],
    max_width: usize,
    selected_node: Option<(usize, usize)>,
    layer_index: isize,
    x: i32,
) {
    for (i, a) in layer.iter().enumerate() {
        let off = (max_width - layer.len()) / 2;
        let point = (x, (40 + 40 * (off + i)) as i32);

        let selected = selected_node
            .map(|s| s.0 as isize == layer_index && s.1 == i)
            .unwrap_or(true);

        let node_color = Color::RGB(165, 136, 171);
        let text_color = Color::RGB(47, 72, 88);
        let node_radius = 16.;

        if selected {
            canvas
                .filled_circle(
                    point.0 as i16,
                    point.1 as i16,
                    node_radius as i16,
                    node_color,
                )
                .unwrap();
        } else {
            canvas
                .circle(
                    point.0 as i16,
                    point.1 as i16,
                    node_radius as i16,
                    node_color,
                )
                .unwrap();
        }

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
) {
    for (i, a) in layer.iter().enumerate() {
        let off = (max_width - layer.len()) / 2;
        let point = (x, (40 + 40 * (off + i)) as i32);

        let selected = selected_node
            .map(|s| s.0 as isize == layer_index && s.1 == i)
            .unwrap_or(true);

        let node_color = Color::RGB(255, 183, 3);
        let text_color = Color::RGB(47, 72, 88);
        let node_radius = 12.;

        if selected {
            canvas
                .filled_circle(
                    point.0 as i16,
                    point.1 as i16,
                    node_radius as i16,
                    node_color,
                )
                .unwrap();
        } else {
            canvas
                .circle(
                    point.0 as i16,
                    point.1 as i16,
                    node_radius as i16,
                    node_color,
                )
                .unwrap();
        }

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
            let point0 = (x0, (40 + 40 * (off_i + i)) as i32);
            let point1 = (x1, (40 + 40 * (off_j + j)) as i32);

            canvas
                .line(
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
            let point0 = (x0, (40 + 40 * (off_i + INPUT_SIZE)) as i32);
            let point1 = (x1, (40 + 40 * (off_j + j)) as i32);
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

impl BrainRenderModel {
    pub fn render(
        &mut self,
        brain: &Brain,
        log: &BrainLog,
        selected_node: Option<(usize, usize)>,
        requested_canvas_width: u32,
        requested_canvas_height: u32,
    ) -> Image {
        if self.buffer.width() != requested_canvas_width
            || self.buffer.height() != requested_canvas_height
        {
            self.buffer = SharedPixelBuffer::new(requested_canvas_width, requested_canvas_height);
        }

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

        let font = ttf_context.load_font_from_rwops(rwops, 12).unwrap();

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

            canvas.set_draw_color(Color::RGB(255, 183, 195));
            canvas.clear();

            let (a0, a1, a2) = log.activations;

            let max_width = a0.len().max(a1.len()).max(a2.len());

            draw_connections::<16, 8>(
                &mut canvas,
                &font,
                &brain.layers().0,
                max_width,
                selected_node,
                0,
                20 + 40,
                20 + 40 + 100,
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
            );

            draw_layer_activations(
                &mut canvas,
                &font,
                a0,
                max_width,
                selected_node,
                -1,
                20 + 40,
            );

            draw_layer_activations(
                &mut canvas,
                &font,
                a1,
                max_width,
                selected_node,
                0,
                20 + 40 + 100,
            );

            draw_layer_activations(
                &mut canvas,
                &font,
                a2,
                max_width,
                selected_node,
                1,
                20 + 40 + 200,
            );

            draw_layer_text(
                &mut canvas,
                &font,
                [
                    "E/C", "FP/", "R-F", "A", "BP/", "R-B", "a", "r", "g", "b", "B/C", "R0", "R1",
                    "R2", "R3", "R4",
                ],
                max_width,
                selected_node,
                -1,
                20,
            );

            draw_layer_text(
                &mut canvas,
                &font,
                ["V", "R", "RV", "BR", "R0", "R1", "R2", "R3"],
                max_width,
                selected_node,
                1,
                20 + 40 + 200 + 40,
            );

            canvas.present();
        }
        slint::Image::from_rgba8(self.buffer.clone())
    }
}
