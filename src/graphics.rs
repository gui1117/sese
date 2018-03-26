use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::swapchain::{self, Swapchain, SwapchainCreationError, Surface};
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};
use vulkano::image::{AttachmentImage, Dimensions, ImageUsage, ImmutableImage};
use vulkano::image::swapchain::SwapchainImage;
use vulkano::buffer::{BufferUsage, CpuBufferPool, ImmutableBuffer};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, LayoutAttachmentDescription,
                           LayoutPassDependencyDescription, LayoutPassDescription, LoadOp,
                           RenderPassAbstract, RenderPassDesc,
                           RenderPassDescClearValues, StoreOp, RenderPass};
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::pipeline::viewport::{Viewport, Scissor};
use vulkano::descriptor::descriptor_set::{DescriptorSet, FixedSizeDescriptorSetsPool,
                                          PersistentDescriptorSet};
use vulkano::command_buffer::pool::standard::StandardCommandPoolAlloc;
use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder, DynamicState};
use vulkano::instance::{PhysicalDevice, PhysicalDeviceType};
use vulkano::sync::{now, GpuFuture};
use vulkano::image::ImageLayout;
use vulkano::format::{self, ClearValue, Format};
use vulkano;
use ncollide::shape;
use alga::general::SubsetOf;
use conrod::render::PrimitiveKind;
use conrod::position::Scalar;
use std::sync::Arc;
use std::fs::File;
use std::time::Duration;
use std::f32::consts::PI;
use specs::World;
use show_message::{OkOrShow, SomeOrShow};

#[repr(C)]
pub enum UiMode {
    Text,
    Geometry,
}

#[derive(Debug, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
    color: [f32; 4],
    mode: u32,
}
impl_vertex!(Vertex, position, tex_coords, color, mode);

pub struct Graphics {
    pub queue: Arc<Queue>,
    pub device: Arc<Device>,
    pub swapchain: Arc<Swapchain<::winit::Window>>,
    pub render_pass: Arc<RenderPass<CustomRenderPassDesc>>,
    pub pipeline: Arc<GraphicsPipelineAbstract + Sync + Send>,
    pub framebuffers: Vec<Arc<FramebufferAbstract + Sync + Send>>,
    // view_buffer_pool: CpuBufferPool<vs::ty::View>,
    // world_buffer_pool: CpuBufferPool<vs::ty::World>,
    // descriptor_sets_pool: FixedSizeDescriptorSetsPool<Arc<GraphicsPipelineAbstract + Sync + Send>>,
    future: Option<Box<GpuFuture>>,
}

// TODO: return result failure ?
impl Graphics {
    pub fn framebuffers_and_descriptors(
        device: &Arc<Device>,
        images: &Vec<Arc<SwapchainImage<::winit::Window>>>,
        render_pass: &Arc<RenderPass<CustomRenderPassDesc>>,
    ) -> Vec<Arc<FramebufferAbstract + Sync + Send>> {
        let depth_buffer_attachment = AttachmentImage::transient(
            device.clone(),
            images[0].dimensions(),
            format::Format::D16Unorm,
        ).unwrap();

        images
            .iter()
            .map(|image| {
                Arc::new(
                    Framebuffer::start(render_pass.clone())
                        .add(image.clone())
                        .unwrap()
                        .add(depth_buffer_attachment.clone())
                        .unwrap()
                        .build()
                        .unwrap(),
                ) as Arc<_>
            })
            .collect::<Vec<_>>()
    }

    pub fn new(window: &Arc<Surface<::winit::Window>>, save: &mut ::resource::Save) -> Graphics {
        let physical = PhysicalDevice::enumerate(window.instance())
            .max_by_key(|device| {
                if let Some(uuid) = save.vulkan_device_uuid().as_ref() {
                    if uuid == device.uuid() {
                        return 100;
                    }
                }
                match device.ty() {
                    PhysicalDeviceType::IntegratedGpu => 4,
                    PhysicalDeviceType::DiscreteGpu => 3,
                    PhysicalDeviceType::VirtualGpu => 2,
                    PhysicalDeviceType::Cpu => 1,
                    PhysicalDeviceType::Other => 0,
                }
            })
            .some_or_show("Failed to enumerate Vulkan devices");
        save.set_vulkan_device_uuid_lazy(physical.uuid());

        let queue_family = physical
            .queue_families()
            .find(|&q| {
                q.supports_graphics() && q.supports_compute()
                    && window.is_supported(q).unwrap_or(false)
            })
            .some_or_show("Failed to find a vulkan graphical queue family");

        let (device, mut queues) = {
            let device_ext = DeviceExtensions {
                khr_swapchain: true,
                ..DeviceExtensions::none()
            };

            Device::new(
                physical,
                physical.supported_features(),
                &device_ext,
                [(queue_family, 0.5)].iter().cloned(),
            ).ok_or_show(|e| format!("Failed to create vulkan device: {}", e))
        };

        let queue = queues.next()
            .some_or_show("Failed to find queue with supported features");

        let (swapchain, images) = {
            let caps = window
                .capabilities(physical)
                .expect("failed to get surface capabilities");

            let dimensions = caps.current_extent.unwrap_or([1280, 1024]);
            let format = caps.supported_formats[0].0;
            let image_usage = ImageUsage {
                color_attachment: true,
                ..ImageUsage::none()
            };

            Swapchain::new(
                device.clone(),
                window.clone(),
                caps.min_image_count,
                format,
                dimensions,
                1,
                image_usage,
                &queue,
                swapchain::SurfaceTransform::Identity,
                swapchain::CompositeAlpha::Opaque,
                swapchain::PresentMode::Fifo,
                true,
                None,
            ).expect("failed to create swapchain")
        };

        let render_pass = Arc::new(
            CustomRenderPassDesc {
                swapchain_image_format: swapchain.format(),
            }.build_render_pass(device.clone())
                .unwrap(),
        );

        let vs = vs::Shader::load(device.clone()).expect("failed to create shader module");
        let fs = fs::Shader::load(device.clone()).expect("failed to create shader module");

        let pipeline = Arc::new(
            vulkano::pipeline::GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_strip()
                .viewports_scissors_dynamic(1)
                .cull_mode_back()
                .fragment_shader(fs.main_entry_point(), ())
                .blend_alpha_blending()
                .render_pass(vulkano::framebuffer::Subpass::from(render_pass.clone(), 0).unwrap())
                .build(device.clone())
                .unwrap(),
        );

        // let view_buffer_pool =
        //     CpuBufferPool::<vs::ty::View>::new(device.clone(), BufferUsage::uniform_buffer());

        // let world_buffer_pool =
        //     CpuBufferPool::<vs::ty::World>::new(device.clone(), BufferUsage::uniform_buffer());

        // let descriptor_sets_pool = FixedSizeDescriptorSetsPool::new(pipeline.clone() as Arc<_>, 0);

        let framebuffers = Graphics::framebuffers_and_descriptors(
            &device,
            &images,
            &render_pass,
        );

        // TODO: when future will be needed
        // let future = Some(Box::new(future.then_signal_fence_and_flush().unwrap()) as Box<_>);

        Graphics {
            future: Some(Box::new(now(device.clone())) as Box<GpuFuture>),
            device,
            queue,
            swapchain,
            render_pass,
            pipeline,
            framebuffers,
            // view_buffer_pool,
            // world_buffer_pool,
            // descriptor_sets_pool,
        }
    }

    fn recreate(&mut self, window: &Arc<Surface<::winit::Window>>) {
        let mut remaining_try = 20;
        let recreate = loop {
            let dimensions = window
                .capabilities(self.device.physical_device())
                .expect("failed to get surface capabilities")
                .current_extent
                .unwrap_or([1024, 768]);

            let res = self.swapchain.recreate_with_dimension(dimensions);

            if remaining_try == 0 {
                break res;
            }

            match res {
                Err(SwapchainCreationError::UnsupportedDimensions) => (),
                res @ _ => {
                    break res;
                }
            }
            remaining_try -= 1;
            ::std::thread::sleep(::std::time::Duration::from_millis(50));
        };

        let (swapchain, images) = recreate.unwrap();
        self.swapchain = swapchain;

        let framebuffers = Graphics::framebuffers_and_descriptors(
            &self.device,
            &images,
            &self.render_pass,
        );
        self.framebuffers = framebuffers;
    }

    pub fn draw(&mut self, world: &mut World, window: &Arc<Surface<::winit::Window>>) {
        self.future.as_mut().unwrap().cleanup_finished();

        // On X with Xmonad and intel HD graphics the acquire stay sometimes forever
        let timeout = Duration::from_secs(2);
        let mut next_image = swapchain::acquire_next_image(self.swapchain.clone(), Some(timeout));
        loop {
            match next_image {
                Err(vulkano::swapchain::AcquireError::OutOfDate)
                | Err(vulkano::swapchain::AcquireError::Timeout) => {
                    self.recreate(&window);
                    next_image =
                        swapchain::acquire_next_image(self.swapchain.clone(), Some(timeout));
                }
                _ => break,
            }
        }

        let (image_num, acquire_future) = next_image.unwrap();

        let command_buffer = self.build_command_buffer(image_num, window.window().hidpi_factor() as f64, world);

        let future = self.future
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(self.queue.clone(), self.swapchain.clone(), image_num)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self.future = Some(Box::new(future) as Box<_>);
            }
            Err(vulkano::sync::FlushError::OutOfDate) => {
                self.future = Some(Box::new(vulkano::sync::now(self.device.clone())) as Box<_>);
            }
            Err(e) => {
                println!("ERROR: {:?}", e);
                self.future = Some(Box::new(vulkano::sync::now(self.device.clone())) as Box<_>);
            }
        }
    }

    fn build_command_buffer(
        &mut self,
        image_num: usize,
        dpi_factor: f64,
        world: &mut World,
    ) -> AutoCommandBuffer<StandardCommandPoolAlloc> {

        let dimensions = self.swapchain.dimensions();

        let screen_dynamic_state = DynamicState {
            viewports: Some(vec![
                Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                    depth_range: 0.0..1.0,
                },
            ]),
            ..DynamicState::none()
        };

        let mut command_buffer_builder = AutoCommandBufferBuilder::primary_one_time_submit(
            self.device.clone(),
            self.queue.family(),
        ).unwrap()
            .begin_render_pass(
                self.framebuffers[image_num].clone(),
                false,
                vec![[0.0, 0.0, 1.0, 1.0].into(), 1.0.into()],
            )
            .unwrap();

        // TODO: Draw world

        // Draw UI

        let half_win_w = (dimensions[0]/2) as f64;
        let half_win_h = (dimensions[1]/2) as f64;

        // Functions for converting for conrod scalar coords to GL vertex coords (-1.0 to 1.0).
        let vx = |x: Scalar| (x * dpi_factor / half_win_w) as f32;
        let vy = |y: Scalar| -(y * dpi_factor / half_win_h) as f32;

        pub fn gamma_srgb_to_linear(c: [f32; 4]) -> [f32; 4] {
            fn component(f: f32) -> f32 {
                // Taken from https://github.com/PistonDevelopers/graphics/src/color.rs#L42
                if f <= 0.04045 {
                    f / 12.92
                } else {
                    ((f + 0.055) / 1.055).powf(2.4)
                }
            }
            [component(c[0]), component(c[1]), component(c[2]), c[3]]
        }

        let owned_primitives = world.read_resource::<::resource::OwnedPrimitives>();
        let mut walk_primitive = owned_primitives.walk();
        while let Some(primitive) = walk_primitive.next() {

            let dynamic_state = DynamicState {
                viewports: Some(vec![
                    Viewport {
                        origin: [
                            (primitive.rect.left() * dpi_factor + half_win_w) as f32,
                            (primitive.rect.bottom() * dpi_factor + half_win_h) as f32,
                        ],
                        dimensions: [
                            (primitive.rect.right() - primitive.rect.left()) as f32,
                            (primitive.rect.top() - primitive.rect.bottom()) as f32,
                        ],
                        depth_range: 0.0..1.0,
                    },
                ]),
                scissors: Some(vec![
                    Scissor {
                        origin: [
                            (primitive.scizzor.left() * dpi_factor + half_win_w) as i32,
                            (primitive.scizzor.bottom() * dpi_factor + half_win_h) as i32,
                        ],
                        dimensions: [
                            (primitive.scizzor.w() * dpi_factor) as u32,
                            (primitive.scizzor.h() * dpi_factor) as u32,
                        ],
                    },
                ]),
                ..DynamicState::none()
            };
            println!("{:?}", dynamic_state);

            let mut vertices = vec![];
            match primitive.kind {
                PrimitiveKind::Rectangle { color } => {
                    let color = ::conrod::color::Color::Rgba(1.0, 0.0, 1.0, 0.5);
                    let color = gamma_srgb_to_linear(color.to_fsa());
                    let (l, r, b, t) = primitive.rect.l_r_b_t();

                    let v = |x, y| {
                        Vertex {
                            position: [vx(x), vy(y)],
                            tex_coords: [0.0, 0.0],
                            color: color,
                            mode: UiMode::Geometry as u32,
                        }
                    };

                    let mut push_v = |x, y| vertices.push(v(x, y));

                    // Bottom left triangle.
                    push_v(l, t);
                    push_v(r, b);
                    push_v(l, b);

                    // Top right triangle.
                    push_v(l, t);
                    push_v(r, b);
                    push_v(r, t);
                },
                PrimitiveKind::TrianglesSingleColor { color, triangles } => {
                    if triangles.is_empty() {
                        continue;
                    }

                    let color = gamma_srgb_to_linear(color.into());

                    let v = |p: [Scalar; 2]| {
                        Vertex {
                            position: [vx(p[0]), vy(p[1])],
                            tex_coords: [0.0, 0.0],
                            color: color,
                            mode: UiMode::Geometry as u32,
                        }
                    };

                    for triangle in triangles {
                        vertices.push(v(triangle[0]));
                        vertices.push(v(triangle[1]));
                        vertices.push(v(triangle[2]));
                    }
                },
                PrimitiveKind::TrianglesMultiColor { triangles } => {
                    if triangles.is_empty() {
                        continue;
                    }

                    let v = |(p, c): ([Scalar; 2], ::conrod::color::Rgba)| {
                        Vertex {
                            position: [vx(p[0]), vy(p[1])],
                            tex_coords: [0.0, 0.0],
                            color: gamma_srgb_to_linear(c.into()),
                            mode: UiMode::Geometry as u32,
                        }
                    };

                    for triangle in triangles {
                        vertices.push(v(triangle[0]));
                        vertices.push(v(triangle[1]));
                        vertices.push(v(triangle[2]));
                    }
                },
                PrimitiveKind::Text { color, text, font_id } => {
                    unimplemented!();
                },
                _ => unreachable!(),
            }

            println!("{:#?}", vertices);
            // TODO: from data ???
            let (buffer, future) = ImmutableBuffer::from_iter(vertices.iter().cloned(), BufferUsage::vertex_buffer(), self.queue.clone()).unwrap();
            self.future = Some(Box::new(self.future.take().unwrap().join(future)) as Box<_>);

            command_buffer_builder = command_buffer_builder.draw(
                self.pipeline.clone(),
                dynamic_state,
                vec![buffer],
                (),
                (),
            )
                .unwrap();
        }


        command_buffer_builder
            .end_render_pass()
            .unwrap()
            .build()
            .unwrap()
    }

}

mod vs {
    #[derive(VulkanoShader)]
    #[ty = "vertex"]
    #[src = "
#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 tex_coords;
layout(location = 2) in vec4 color;
layout(location = 3) in uint mode;

layout(location = 0) out vec2 v_tex_coords;
layout(location = 1) out vec4 v_color;
layout(location = 2) flat out uint v_mode;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    v_tex_coords = tex_coords;
    v_color = color;
    v_mode = mode;
}
"]
    struct _Dummy;
}

mod fs {
    #[derive(VulkanoShader)]
    #[ty = "fragment"]
    #[src = "
#version 450

layout(location = 0) in vec2 v_tex_coords;
layout(location = 1) in vec4 v_color;
layout(location = 2) flat in uint v_mode;

layout(location = 0) out vec4 f_color;

// layout(set = 0, binding = 0) uniform sampler2D tex;

void main() {
    f_color = v_color;
    // Text
    // if (v_mode == uint(0)) {
    //     f_color = v_color * vec4(1.0, 1.0, 1.0, texture(tex, v_tex_coords).r);
    // // 2D Geometry
    // } else if (v_mode == uint(1)) {
    // }
}
"]
    struct _Dummy;
}

pub struct CustomRenderPassDesc {
    swapchain_image_format: Format,
}

unsafe impl RenderPassDesc for CustomRenderPassDesc {
    #[inline]
    fn num_attachments(&self) -> usize {
        2
    }

    #[inline]
    fn attachment_desc(&self, id: usize) -> Option<LayoutAttachmentDescription> {
        match id {
            // Colors
            0 => Some(LayoutAttachmentDescription {
                format: self.swapchain_image_format,
                samples: 1,
                load: LoadOp::Clear,
                store: StoreOp::Store,
                stencil_load: LoadOp::Clear,
                stencil_store: StoreOp::Store,
                initial_layout: ImageLayout::Undefined,
                final_layout: ImageLayout::ColorAttachmentOptimal,
            }),
            // Depth buffer
            1 => Some(LayoutAttachmentDescription {
                format: Format::D16Unorm,
                samples: 1,
                load: LoadOp::Clear,
                store: StoreOp::DontCare,
                stencil_load: LoadOp::Clear,
                stencil_store: StoreOp::DontCare,
                initial_layout: ImageLayout::Undefined,
                final_layout: ImageLayout::DepthStencilAttachmentOptimal,
            }),
            _ => None,
        }
    }

    #[inline]
    fn num_subpasses(&self) -> usize {
        1
    }

    #[inline]
    fn subpass_desc(&self, id: usize) -> Option<LayoutPassDescription> {
        match id {
            // draw
            0 => Some(LayoutPassDescription {
                color_attachments: vec![(0, ImageLayout::ColorAttachmentOptimal)],
                depth_stencil: Some((1, ImageLayout::DepthStencilAttachmentOptimal)),
                input_attachments: vec![],
                resolve_attachments: vec![],
                preserve_attachments: vec![],
            }),
            _ => None,
        }
    }

    #[inline]
    fn num_dependencies(&self) -> usize {
        0
    }

    #[inline]
    fn dependency_desc(&self, id: usize) -> Option<LayoutPassDependencyDescription> {
        match id {
            _ => None,
        }
    }
}

unsafe impl RenderPassDescClearValues<Vec<ClearValue>> for CustomRenderPassDesc {
    fn convert_clear_values(&self, values: Vec<ClearValue>) -> Box<Iterator<Item = ClearValue>> {
        // FIXME: safety checks
        Box::new(values.into_iter())
    }
}
