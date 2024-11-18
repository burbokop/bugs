use bugs_lib::{brain::Brain, bug::BrainLog};
use slint::{Rgba8Pixel, SharedPixelBuffer};

use crate::render::BrainRenderModel;

pub struct OpenGlBrainRenderModel {}

impl Default for OpenGlBrainRenderModel {
    fn default() -> Self {
        Self {}
    }
}

impl BrainRenderModel for OpenGlBrainRenderModel {
    fn render(
        &self,
        buffer: &mut SharedPixelBuffer<Rgba8Pixel>,
        brain: &Brain,
        log: &BrainLog,
        selected_node: Option<(usize, usize)>,
    ) {
        todo!()
    }
}
