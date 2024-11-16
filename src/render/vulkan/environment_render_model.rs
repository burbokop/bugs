use bugs_lib::{
    environment::Environment,
    math::{Point, Size},
    utils::Float,
};
use slint::{Rgba8Pixel, SharedPixelBuffer};

use crate::{
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
    device::{
        physical::{PhysicalDevice, PhysicalDeviceType},
        Device, DeviceCreateInfo, Queue, QueueCreateInfo, QueueFlags,
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
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
        GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, Subpass},
    sync::GpuFuture,
    VulkanLibrary,
};

#[derive(BufferContents, Vertex)]
#[repr(C)]
struct MyVertex {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
}

pub struct VulkanEnvironmentRenderModel {
    library: Arc<VulkanLibrary>,
    instance: Arc<Instance>,
    physical_device: Arc<PhysicalDevice>,
    queue_family_index: u32,
    device: Arc<Device>,
    queue: Arc<Queue>,
    vertex_buffer: Subbuffer<[MyVertex]>,
    format: Format,
    memory_allocator: Arc<StandardMemoryAllocator>,
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

        let vertices = [
            MyVertex {
                position: [-0.5, -0.25],
            },
            MyVertex {
                position: [0.0, 0.5],
            },
            MyVertex {
                position: [0.25, -0.1],
            },
        ];
        let vertex_buffer = Buffer::from_iter(
            memory_allocator.clone(),
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

        let format = Format::R8G8B8A8_UNORM;

        Self {
            library,
            instance,
            physical_device,
            queue_family_index,
            device,
            queue,
            vertex_buffer,format,
            memory_allocator,
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
        self. render_output_image = Some(Image::new(
            self.memory_allocator.clone(),
            ImageCreateInfo {
                format,
                usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSFER_SRC,
                extent: [*view_port_size.w(), *view_port_size.h(), 1],
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .unwrap());

        self. render_output_buf = Some(Buffer::from_iter(
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
        .unwrap());
    }

    fn render(
        &self,
        buffer: &mut SharedPixelBuffer<Rgba8Pixel>,
        environment: &Environment<T>,
        camera: &Camera,
        selected_bug_id: &Option<usize>,
        active_tool: Tool,
        tool_action_point: Option<Point<Float>>,
        tool_action_active: bool,
        chunks_display_mode: ChunksDisplayMode,
    ) {
        assert_eq!(
            buffer.as_bytes().len(),
            buffer.width() as usize * buffer.height() as usize * 4
        );
        let buffer_size: Size<u32> = (buffer.width(), buffer.height()).into();

        let render_output_buf = self.render_output_buf.clone().unwrap();
        let render_output_image = self.render_output_image.clone().unwrap();

        mod vs {
            vulkano_shaders::shader! {
                ty: "vertex",
                src: r"
                #version 450

                layout(location = 0) in vec2 position;

                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                }
            ",
            }
        }

        mod fs {
            vulkano_shaders::shader! {
                ty: "fragment",
                src: r"
                #version 450

                layout(location = 0) out vec4 f_color;

                void main() {
                    f_color = vec4(1.0, 0.0, 0.0, 1.0);
                }
            ",
            }
        }

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

        let render_output_image_view = ImageView::new_default(render_output_image.clone()).unwrap();

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

            let vertex_input_state = MyVertex::per_vertex().definition(&vs).unwrap();

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
                            extent: [*buffer_size.w() as f32, *buffer_size.h() as f32],
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
            command_buffer_allocator.clone(),
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],
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
            .bind_vertex_buffers(0, self.vertex_buffer.clone())
            .unwrap();
        unsafe { builder.draw(self.vertex_buffer.len() as u32, 1, 0, 0) }.unwrap();

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

        // Access the bytes copied into the host-accessible output buffer by reference.
        let buffer_content = render_output_buf.read().unwrap();

        assert_eq!(buffer.make_mut_bytes().len(), buffer_content.len());

        for i in 0..buffer.make_mut_bytes().len() {
            buffer.make_mut_bytes()[i] = buffer_content[i];
        }

        // let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("triangle.png");
        // let file = File::create(&path).unwrap();
        // let w = &mut BufWriter::new(file);

        // let mut encoder = png::Encoder::new(w, 1920, 1080);
        // encoder.set_color(png::ColorType::Rgba);
        // encoder.set_depth(png::BitDepth::Eight);
        // let mut writer = encoder.write_header().unwrap();
        // writer.write_image_data(&buffer_content).unwrap();

        // if let Ok(path) = path.canonicalize() {
        //     println!("Saved to {}", path.display());
        // }
    }
}
