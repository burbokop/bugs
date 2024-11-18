use super::Camera;
use crate::Tool;
use bugs_lib::{
    environment::Environment,
    math::{Point, Size},
    utils::Float,
};
use slint::Image;

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
    fn render(
        &mut self,
        view_port_size: Size<u32>,
        environment: &Environment<T>,
        camera: &Camera,
        selected_bug_id: &Option<usize>,
        active_tool: Tool,
        tool_action_point: Option<Point<Float>>,
        tool_action_active: bool,
        chunks_display_mode: ChunksDisplayMode,
    ) -> slint::Image;
}

pub struct EnvironmentRenderer<T> {
    model: Box<dyn EnvironmentRenderModel<T>>,
}

impl<T> EnvironmentRenderer<T> {
    pub(crate) fn new<Model: EnvironmentRenderModel<T> + 'static>(model: Model) -> Self {
        Self {
            model: Box::new(model),
        }
    }

    pub(crate) fn render(
        &mut self,
        view_port_size: Size<u32>,
        environment: &Environment<T>,
        camera: &Camera,
        selected_bug_id: &Option<usize>,
        active_tool: Tool,
        tool_action_point: Option<Point<Float>>,
        tool_action_active: bool,
        chunks_display_mode: ChunksDisplayMode,
    ) -> Image {
        self.model.render(
            view_port_size,
            environment,
            camera,
            selected_bug_id,
            active_tool,
            tool_action_point,
            tool_action_active,
            chunks_display_mode,
        )
    }
}
