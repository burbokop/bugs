use super::Camera;
use crate::Tool;
use bugs_lib::{
    environment::Environment,
    math::{Point, Rect, Size},
    utils::Float,
};
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};

#[derive(Debug, Clone, PartialEq)]
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

pub trait EnvironmentRenderModel<T> {
    /// is called on start, on window resize, etc. (not too frequent)
    fn init(&mut self, view_port_size: Size<u32>);

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
        chunks_display_mode: ChunksDisplayMode,
    );
}

pub struct EnvironmentRenderer<T> {
    buffer: SharedPixelBuffer<Rgba8Pixel>,
    model: Box<dyn EnvironmentRenderModel<T>>,
}

impl<T> EnvironmentRenderer<T> {
    pub(crate) fn new<Model: EnvironmentRenderModel<T> + 'static>(model: Model) -> Self {
        Self {
            buffer: SharedPixelBuffer::new(0, 0),
            model: Box::new(model),
        }
    }

    pub(crate) fn render(
        &mut self,
        environment: &Environment<T>,
        camera: &Camera,
        selected_bug_id: &Option<usize>,
        active_tool: Tool,
        tool_action_point: Option<Point<Float>>,
        tool_action_active: bool,
        chunks_display_mode: ChunksDisplayMode,
        mut requested_canvas_width: u32,
        mut requested_canvas_height: u32,
        quality_deterioration: u32,
    ) -> Image {
        requested_canvas_height /= quality_deterioration;
        requested_canvas_width /= quality_deterioration;

        if self.buffer.width() != requested_canvas_width
            || self.buffer.height() != requested_canvas_height
        {
            self.buffer = SharedPixelBuffer::new(requested_canvas_width, requested_canvas_height);
            self.model
                .init((self.buffer.width(), self.buffer.height()).into());
        }

        let buffer_size: Size<u32> = (self.buffer.width(), self.buffer.height()).into();
        let view_port_rect: Rect<_> = (
            0.,
            0.,
            (*buffer_size.w() * quality_deterioration) as Float,
            (*buffer_size.h() * quality_deterioration) as Float,
        )
            .into();

        self.model.render(
            &mut self.buffer,
            view_port_rect,
            environment,
            camera,
            selected_bug_id,
            active_tool,
            tool_action_point,
            tool_action_active,
            chunks_display_mode,
        );

        Image::from_rgba8(self.buffer.clone())
    }
}
