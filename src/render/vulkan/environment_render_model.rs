use bugs_lib::{
    chunk::RawChunkIndex,
    environment::Environment,
    math::{map_into_range, Point, Rect, Size},
    utils::{Color, Float},
};
use slint::{Rgba8Pixel, SharedPixelBuffer};

use crate::{
    app_utils::color_to_slint_rgba8_color,
    render::{Camera, ChunksDisplayMode, EnvironmentRenderModel},
    Tool,
};

use std::{default::Default, sync::Arc};
use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        CopyImageToBufferInfo, PrimaryCommandBufferAbstract, RenderPassBeginInfo, SubpassBeginInfo,
        SubpassContents,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, DescriptorSet, WriteDescriptorSet,
    },
    device::{
        physical::{PhysicalDevice, PhysicalDeviceType},
        Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
    },
    format::Format,
    image::{view::ImageView, Image, ImageCreateInfo, ImageUsage},
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        graphics::{
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::{self, Vertex as _, VertexDefinition},
            viewport::{Viewport, ViewportState},
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
        GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout,
        PipelineShaderStageCreateInfo,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, Subpass},
    sync::GpuFuture,
    VulkanLibrary,
};

use super::glsl_convertions;

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
        #version 450

        layout(location = 0) in vec2 position;
        layout(location = 1) in vec4 color;

        layout(location = 0) out vec4 color_output;

        layout(set = 0, binding = 0) uniform Global {
            mat3 transformation;
            vec2 view_port_size;
        } global;

        vec2 transform(vec2 p) {
            vec3 v = vec3(p, 1.0) * global.transformation;
            return vec2(
                v.x / v.z,
                v.y / v.z);
        }

        vec2 reorigin(vec2 p) {
            return ((p - global.view_port_size / 2) / global.view_port_size) * 2.;
        }

        void main() {
            gl_Position = vec4(reorigin(transform(position)), 0.0, 1.0);
            color_output = color;
        }
    ",
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r"
        #version 450

        layout(location = 0) in vec4 color;

        layout(location = 0) out vec4 f_color;

        void main() {
            f_color = color;
        }
    ",
    }
}

#[derive(Debug, BufferContents, vertex_input::Vertex)]
#[repr(C)]
struct Vertex {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
    #[format(R32G32B32A32_SFLOAT)]
    color: [f32; 4],
}

impl Vertex {
    fn from_point(p: Point<Float>, c: Color) -> Self {
        Self {
            position: glsl_convertions::point_to_vec2(p.as_f32()),
            color: glsl_convertions::color_to_vec4(c),
        }
    }
}

struct VertexShape<const V: usize, const I: usize> {
    vertices: [Vertex; V],
    indices: [u32; I],
}

struct VertexShapeVec {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl Default for VertexShapeVec {
    fn default() -> Self {
        Self {
            vertices: Default::default(),
            indices: Default::default(),
        }
    }
}

impl VertexShapeVec {
    pub fn push<const V: usize, const I: usize>(&mut self, shape: VertexShape<V, I>) {
        let offset = self.vertices.len() as u32;
        self.vertices.extend(shape.vertices.into_iter());
        self.indices
            .extend(shape.indices.into_iter().map(|i| offset + i));
    }
}

impl From<VertexShapeVec> for (Vec<Vertex>, Vec<u32>) {
    fn from(value: VertexShapeVec) -> Self {
        (value.vertices, value.indices)
    }
}

mod vertex_shapes {
    use super::*;

    pub(super) fn rect(r: Rect<Float>, c: Color) -> VertexShape<4, 6> {
        VertexShape {
            vertices: [
                Vertex::from_point(r.left_top(), c.clone()),
                Vertex::from_point(r.right_top(), c.clone()),
                Vertex::from_point(r.right_bottom(), c.clone()),
                Vertex::from_point(r.left_bottom(), c.clone()),
            ],
            indices: [0, 1, 2, 0, 2, 3],
        }
    }
}

fn draw_chunk_simplified(
    shapes: &mut VertexShapeVec,
    rect: Rect<Float>,
    ocupants_count: usize,
    color: Color,
) {
    if ocupants_count > 0 {
        let max_ocupants_count = 8;
        shapes.push(vertex_shapes::rect(
            rect,
            if ocupants_count >= max_ocupants_count {
                color
            } else {
                color.map_a(|a| {
                    map_into_range(
                        ocupants_count as Float,
                        0. ..max_ocupants_count as Float,
                        (a / 16.)..a,
                    )
                })
            },
        ));
    }
}

pub struct VulkanEnvironmentRenderModel {
    library: Arc<VulkanLibrary>,
    instance: Arc<Instance>,
    physical_device: Arc<PhysicalDevice>,
    queue_family_index: u32,
    device: Arc<Device>,
    queue: Arc<Queue>,
    format: Format,
    memory_allocator: Arc<StandardMemoryAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    render_output_image: Option<Arc<Image>>,
    render_output_buf: Option<Subbuffer<[u8]>>,
}

impl Default for VulkanEnvironmentRenderModel {
    fn default() -> Self {
        let library = VulkanLibrary::new().unwrap();

        let instance = Instance::new(
            library.clone(),
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                ..Default::default()
            },
        )
        .unwrap();

        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()
            .unwrap()
            // No need for swapchain extension support.
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .position(|q| q.queue_flags.intersects(QueueFlags::GRAPHICS))
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            })
            .expect("no suitable physical device found");

        println!(
            "Using device: {} (type: {:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type,
        );

        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                enabled_extensions: DeviceExtensions {
                    khr_storage_buffer_storage_class: true,
                    ..DeviceExtensions::empty()
                },
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .unwrap();

        let queue = queues.next().unwrap();
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let format = Format::R8G8B8A8_UNORM;

        Self {
            library,
            instance,
            physical_device,
            queue_family_index,
            device,
            queue,
            format,
            memory_allocator,
            descriptor_set_allocator,
            render_output_buf: None,
            render_output_image: None,
        }
    }
}

// 2.8
// 2.7
// 14 N

impl<T> EnvironmentRenderModel<T> for VulkanEnvironmentRenderModel {
    fn init(&mut self, view_port_size: Size<u32>) {
        let format = self.format;
        self.render_output_image = Some(
            Image::new(
                self.memory_allocator.clone(),
                ImageCreateInfo {
                    format,
                    usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSFER_SRC,
                    extent: [*view_port_size.w(), *view_port_size.h(), 1],
                    ..Default::default()
                },
                AllocationCreateInfo::default(),
            )
            .unwrap(),
        );

        self.render_output_buf = Some(
            Buffer::from_iter(
                self.memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::TRANSFER_DST,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_HOST
                        | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                    ..Default::default()
                },
                (0..(*view_port_size.w() * *view_port_size.h() * 4)).map(|_| 0u8),
            )
            .unwrap(),
        );
    }

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
    ) {
        let background_color = Color::from_rgb24(211, 250, 199);

        assert_eq!(
            buffer.as_bytes().len(),
            buffer.width() as usize * buffer.height() as usize * 4
        );

        let transformation = camera.transformation();

        let view_port_rect_in_world_space = &(!&transformation).unwrap() * &view_port_rect;

        let mut shapes: VertexShapeVec = Default::default();

        for (index, ocupants_count) in
            environment.food_chunks_in_area(view_port_rect_in_world_space)
        {
            let index: RawChunkIndex = index.into();
            let rect = &Rect::from((
                index.x() as Float * 256.,
                index.y() as Float * 256.,
                256.,
                256.,
            ));
            if view_port_rect_in_world_space.contains(&rect)
                || view_port_rect_in_world_space.instersects(&rect)
            {
                draw_chunk_simplified(
                    &mut shapes,
                    *rect,
                    ocupants_count,
                    Color::from_rgb24(255, 110, 162),
                );
            }
        }

        for (index, ocupants_count) in environment.bug_chunks_in_area(view_port_rect_in_world_space)
        {
            let index: RawChunkIndex = index.into();
            let rect = &Rect::from((
                index.x() as Float * 256.,
                index.y() as Float * 256.,
                256.,
                256.,
            ));
            if view_port_rect_in_world_space.contains(&rect)
                || view_port_rect_in_world_space.instersects(&rect)
            {
                draw_chunk_simplified(
                    &mut shapes,
                    *rect,
                    ocupants_count,
                    Color::from_rgb24(0, 0, 255),
                );
            }
        }

        let (vertices, indices) = shapes.into();

        if vertices.is_empty() {
            buffer
                .make_mut_slice()
                .iter_mut()
                .for_each(|x| *x = color_to_slint_rgba8_color(&background_color));
        } else {
            let vertex_buffer = Buffer::from_iter(
                self.memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::VERTEX_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                vertices,
            )
            .unwrap();

            let index_buffer = Buffer::from_iter(
                self.memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::INDEX_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                indices,
            )
            .unwrap();

            let render_pass = vulkano::single_pass_renderpass!(
                self.device.clone(),
                attachments: {
                    color: {
                        format: self.format,
                        samples: 1,
                        load_op: Clear,
                        store_op: Store,
                    },
                },
                pass: {
                    color: [color],
                    depth_stencil: {},
                },
            )
            .unwrap();

            let render_output_buf = self.render_output_buf.clone().unwrap();
            let render_output_image = self.render_output_image.clone().unwrap();

            let render_output_image_view =
                ImageView::new_default(render_output_image.clone()).unwrap();

            let framebuffer = Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    // Attach the offscreen image to the framebuffer.
                    attachments: vec![render_output_image_view],
                    ..Default::default()
                },
            )
            .unwrap();

            let pipeline = {
                let vs = vs::load(self.device.clone())
                    .unwrap()
                    .entry_point("main")
                    .unwrap();
                let fs = fs::load(self.device.clone())
                    .unwrap()
                    .entry_point("main")
                    .unwrap();

                let vertex_input_state = Vertex::per_vertex().definition(&vs).unwrap();

                let stages = [
                    PipelineShaderStageCreateInfo::new(vs),
                    PipelineShaderStageCreateInfo::new(fs),
                ];

                let layout = PipelineLayout::new(
                    self.device.clone(),
                    PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                        .into_pipeline_layout_create_info(self.device.clone())
                        .unwrap(),
                )
                .unwrap();

                assert!(
                    layout
                        .set_layouts()
                        .get(0)
                        .unwrap()
                        .descriptor_counts()
                        .len()
                        > 0
                );

                let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

                GraphicsPipeline::new(
                    self.device.clone(),
                    None,
                    GraphicsPipelineCreateInfo {
                        stages: stages.into_iter().collect(),
                        vertex_input_state: Some(vertex_input_state),
                        input_assembly_state: Some(InputAssemblyState::default()),
                        viewport_state: Some(ViewportState {
                            viewports: [Viewport {
                                offset: [0.0, 0.0],
                                extent: [buffer.width() as f32, buffer.height() as f32],
                                depth_range: 0.0..=1.0,
                            }]
                            .into_iter()
                            .collect(),
                            ..Default::default()
                        }),
                        rasterization_state: Some(RasterizationState::default()),
                        multisample_state: Some(MultisampleState::default()),
                        color_blend_state: Some(ColorBlendState::with_attachment_states(
                            subpass.num_color_attachments(),
                            ColorBlendAttachmentState::default(),
                        )),
                        subpass: Some(subpass.into()),
                        ..GraphicsPipelineCreateInfo::layout(layout)
                    },
                )
                .unwrap()
            };

            let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
                self.device.clone(),
                Default::default(),
            ));

            // Host-accessible buffer where the offscreen image's contents are copied to after rendering.

            let mut builder = AutoCommandBufferBuilder::primary(
                command_buffer_allocator,
                self.queue.queue_family_index(),
                CommandBufferUsage::OneTimeSubmit,
            )
            .unwrap();

            let global_uniform_object = vs::Global {
                transformation: glsl_convertions::matrix_to_mat3(transformation.as_f32()),
                view_port_size: glsl_convertions::size_to_vec2(view_port_rect.size().as_f32()),
            };

            let global_uniform_buffer = Buffer::from_data(
                self.memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::STORAGE_BUFFER | BufferUsage::UNIFORM_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                    ..Default::default()
                },
                global_uniform_object,
            )
            .unwrap();

            let descriptor_set = DescriptorSet::new(
                self.descriptor_set_allocator.clone(),
                pipeline.layout().set_layouts().get(0).unwrap().clone(),
                [WriteDescriptorSet::buffer(0, global_uniform_buffer)],
                [],
            )
            .unwrap();

            let index_buffer_len = index_buffer.len();
            builder
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: vec![Some(
                            glsl_convertions::color_to_vec4(background_color).into(),
                        )],
                        // This framebuffer has the offscreen image attached to it.
                        ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                    },
                    SubpassBeginInfo {
                        contents: SubpassContents::Inline,
                        ..Default::default()
                    },
                )
                .unwrap()
                .bind_pipeline_graphics(pipeline.clone())
                .unwrap()
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    pipeline.layout().clone(),
                    0,
                    descriptor_set,
                )
                .unwrap()
                .bind_vertex_buffers(0, vertex_buffer)
                .unwrap()
                .bind_index_buffer(index_buffer)
                .unwrap();

            unsafe { builder.draw_indexed(index_buffer_len as u32, 1, 0, 0, 0) }.unwrap();

            builder.end_render_pass(Default::default()).unwrap();

            // The output image stores information in an unknown, non-linear layout, optimized for usage on
            // the device. This step copies the output image into a host-readable linear output buffer
            // where consecutive pixels in the image are laid out consecutively in memory.
            builder
                .copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(
                    render_output_image.clone(),
                    render_output_buf.clone(),
                ))
                .unwrap();

            let command_buffer = builder.build().unwrap();

            let finished = command_buffer.clone().execute(self.queue.clone()).unwrap();

            finished
                .then_signal_fence_and_flush()
                .unwrap()
                .wait(None)
                .unwrap();

            let buffer_content = render_output_buf.read().unwrap();
            assert_eq!(buffer.make_mut_bytes().len(), buffer_content.len());

            buffer.make_mut_bytes().clone_from_slice(&buffer_content);
        }
    }
}
