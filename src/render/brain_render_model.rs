use bugs_lib::{brain::Brain, bug::BrainLog};
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};

pub trait BrainRenderModel {
    fn render(
        &self,
        buffer: &mut SharedPixelBuffer<Rgba8Pixel>,
        brain: &Brain,
        log: &BrainLog,
        selected_node: Option<(usize, usize)>,
    );
}

pub struct BrainRenderer {
    buffer: SharedPixelBuffer<Rgba8Pixel>,
    model: Box<dyn BrainRenderModel>,
}

impl BrainRenderer {
    pub(crate) fn new<Model: BrainRenderModel + 'static>(model: Model) -> Self {
        Self {
            buffer: SharedPixelBuffer::new(0, 0),
            model: Box::new(model),
        }
    }

    pub(crate) fn render(
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

        self.model
            .render(&mut self.buffer, brain, log, selected_node);

        Image::from_rgba8(self.buffer.clone())
    }
}
