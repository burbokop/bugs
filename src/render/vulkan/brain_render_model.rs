use bugs_lib::{brain::Brain, bug::BrainLog};
use slint::{Rgba8Pixel, SharedPixelBuffer};

use crate::render::BrainRenderModel;

pub struct VulkanBrainRenderModel {}

impl Default for VulkanBrainRenderModel {
    fn default() -> Self {
        Self {}
    }
}

impl BrainRenderModel for VulkanBrainRenderModel {
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
