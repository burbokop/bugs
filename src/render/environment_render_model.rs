use super::Camera;
use crate::Tool;
use bugs_lib::{
    environment::Environment,
    math::{Point, Rect, Size},
    utils::Float,
};
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum EnvironmentDisplayMode {
    Optic,
    Crc,
    CrcChunks,
    FoodChunks,
    BugChunks,
    FoodAndBugChunks,
}

impl EnvironmentDisplayMode {
    pub(crate) fn prev(self) -> Self {
        match self {
            EnvironmentDisplayMode::Optic => EnvironmentDisplayMode::FoodAndBugChunks,
            EnvironmentDisplayMode::Crc => EnvironmentDisplayMode::Optic,
            EnvironmentDisplayMode::CrcChunks => EnvironmentDisplayMode::Crc,
            EnvironmentDisplayMode::FoodChunks => EnvironmentDisplayMode::CrcChunks,
            EnvironmentDisplayMode::BugChunks => EnvironmentDisplayMode::FoodChunks,
            EnvironmentDisplayMode::FoodAndBugChunks => EnvironmentDisplayMode::BugChunks,
        }
    }

    pub(crate) fn next(self) -> Self {
        match self {
            EnvironmentDisplayMode::Optic => EnvironmentDisplayMode::Crc,
            EnvironmentDisplayMode::Crc => EnvironmentDisplayMode::CrcChunks,
            EnvironmentDisplayMode::CrcChunks => EnvironmentDisplayMode::FoodChunks,
            EnvironmentDisplayMode::FoodChunks => EnvironmentDisplayMode::BugChunks,
            EnvironmentDisplayMode::BugChunks => EnvironmentDisplayMode::FoodAndBugChunks,
            EnvironmentDisplayMode::FoodAndBugChunks => EnvironmentDisplayMode::Optic,
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
        chunks_display_mode: EnvironmentDisplayMode,
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
        environment_display_mode: EnvironmentDisplayMode,
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
            environment_display_mode,
        );

        Image::from_rgba8(self.buffer.clone())
    }
}
