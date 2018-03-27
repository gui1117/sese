use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::swapchain::{self, Swapchain, SwapchainCreationError, Surface};
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};
use vulkano::image::{AttachmentImage, Dimensions, ImageUsage, ImmutableImage, ImageAccess, StorageImage};
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
use conrod::text;
use std::sync::Arc;
use std::fs::File;
use std::time::Duration;
use std::f32::consts::PI;
use std::cell::RefCell;
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

    pub glyph_cache: text::GlyphCache<'static>,
    pub glyph_cache_pixel_buffer: Vec<u8>,
    pub glyph_cache_image_descriptor_set: Arc<DescriptorSet + Sync + Send>,
    pub glyph_cache_image_sampler: Arc<Sampler>,
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
    ) -> (
        Vec<Arc<FramebufferAbstract + Sync + Send>>,
        (),
    ){
        let dimensions = images[0].dimensions().width_height();

        let framebuffers = images
            .iter()
            .map(|image| {
                Arc::new(
                    Framebuffer::start(render_pass.clone())
                        .add(image.clone())
                        .unwrap()
                        .build()
                        .unwrap(),
                ) as Arc<_>
            })
            .collect::<Vec<_>>();

        (framebuffers, ())
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
                .viewports_dynamic_scissors_irrelevant(1)
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

        let (framebuffers, ()) = Graphics::framebuffers_and_descriptors(
            &device,
            &images,
            &render_pass,
        );

        let glyph_cache = text::GlyphCache::new(::CFG.glyph_width, ::CFG.glyph_height, ::CFG.glyph_scale_tolerance, ::CFG.glyph_position_tolerance);
        let glyph_cache_pixel_buffer = vec!(0; (::CFG.glyph_width * ::CFG.glyph_height) as usize);

        let (glyph_cache_image, future) = ImmutableImage::from_iter(
            glyph_cache_pixel_buffer.iter().cloned(),
            Dimensions::Dim2d { width: ::CFG.glyph_width as u32, height: ::CFG.glyph_height as u32 },
            Format::R8Unorm,
            queue.clone(),
        ).unwrap();

        let glyph_cache_image_sampler = Sampler::new(
            device.clone(),
            Filter::Linear,
            Filter::Linear,
            MipmapMode::Nearest,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            0.0, 1.0, 0.0, 0.0,
        ).unwrap();

        let glyph_cache_image_descriptor_set = Arc::new(
            PersistentDescriptorSet::start(pipeline.clone(), 0)
                .add_sampled_image(glyph_cache_image.clone(), glyph_cache_image_sampler.clone())
                .unwrap()
                .build()
                .unwrap()
        );

        let future = Some(Box::new(future.then_signal_fence_and_flush().unwrap()) as Box<_>);

        Graphics {
            future,
            device,
            queue,
            swapchain,
            render_pass,
            pipeline,
            framebuffers,
            glyph_cache,
            glyph_cache_pixel_buffer,
            glyph_cache_image_sampler,
            glyph_cache_image_descriptor_set,

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

        let (framebuffers, ()) = Graphics::framebuffers_and_descriptors(
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
                vec![[0.0, 0.0, 1.0, 1.0].into()],
            )
            .unwrap();

        // TODO: Draw world

        // Draw UI

        let half_win_w = dimensions[0] as f64 / 2.0;
        let half_win_h = dimensions[1] as f64 / 2.0;

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

            let mut vertices = vec![];
            match primitive.kind {
                PrimitiveKind::Rectangle { color } => {
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
                    push_v(r, b);
                    push_v(l, t);
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
                    println!("{:?}", primitive.rect);
                    let positioned_glyphs = text.positioned_glyphs(dpi_factor as f32);

                    // Queue the glyphs to be cached.
                    for glyph in positioned_glyphs.iter() {
                        self.glyph_cache.queue_glyph(font_id.index(), glyph.clone());
                    }

                    let changed = RefCell::new(false);
                    {
                        let ref mut glyph_cache_pixel_buffer = self.glyph_cache_pixel_buffer;
                        self.glyph_cache.cache_queued(|rect, src_data| {
                            *changed.borrow_mut() = true;
                            let width = (rect.max.x - rect.min.x) as usize;
                            let height = (rect.max.y - rect.min.y) as usize;
                            let mut dst_index = rect.min.y as usize * ::CFG.glyph_width  as usize + rect.min.x as usize;
                            let mut src_index = 0;

                            for _ in 0..height {
                                let dst_slice = &mut glyph_cache_pixel_buffer[dst_index..dst_index+width];
                                let src_slice = &src_data[src_index..src_index+width];
                                dst_slice.copy_from_slice(src_slice);

                                dst_index += ::CFG.glyph_width as usize;
                                src_index += width;
                            }
                        }).unwrap();
                    }
                    let changed = changed.borrow();

                    if *changed {
                        let (glyph_cache_image, future) = ImmutableImage::from_iter(
                            self.glyph_cache_pixel_buffer.iter().cloned(),
                            Dimensions::Dim2d { width: ::CFG.glyph_width as u32, height: ::CFG.glyph_height as u32 },
                            Format::R8Unorm,
                            self.queue.clone(),
                        ).unwrap();
                        self.future = Some(Box::new(self.future.take().unwrap().join(future)) as Box<_>);

                        self.glyph_cache_image_descriptor_set = Arc::new(
                            PersistentDescriptorSet::start(self.pipeline.clone(), 0)
                                .add_sampled_image(glyph_cache_image, self.glyph_cache_image_sampler.clone())
                                .unwrap()
                                .build()
                                .unwrap()
                        );
                    }

                    let color = gamma_srgb_to_linear(color.to_fsa());

                    let cache_id = font_id.index();

                    let origin = text::rt::point(0.0, 0.0);
                    let to_gl_rect = |screen_rect: text::rt::Rect<i32>| text::rt::Rect {
                        min: origin
                            + (text::rt::vector(screen_rect.min.x as f32 / dimensions[0] as f32 - 0.5,
                                          1.0 - screen_rect.min.y as f32 / dimensions[1] as f32 - 0.5)) * 2.0,
                        max: origin
                            + (text::rt::vector(screen_rect.max.x as f32 / dimensions[0] as f32 - 0.5,
                                          1.0 - screen_rect.max.y as f32 / dimensions[1] as f32 - 0.5)) * 2.0
                    };

                    for g in positioned_glyphs {
                        if let Ok(Some((uv_rect, screen_rect))) = self.glyph_cache.rect_for(cache_id, g) {
                            let gl_rect = to_gl_rect(screen_rect);
                            let v = |p: [f32; 2], t: [f32; 2]| Vertex {
                                position: [p[0], -p[1]],
                                tex_coords: t,
                                color: color,
                                mode: UiMode::Text as u32,
                            };
                            let mut push_v = |p, t| vertices.push(v(p, t));
                            push_v([gl_rect.min.x, gl_rect.max.y], [uv_rect.min.x, uv_rect.max.y]);
                            push_v([gl_rect.min.x, gl_rect.min.y], [uv_rect.min.x, uv_rect.min.y]);
                            push_v([gl_rect.max.x, gl_rect.min.y], [uv_rect.max.x, uv_rect.min.y]);

                            push_v([gl_rect.max.x, gl_rect.max.y], [uv_rect.max.x, uv_rect.max.y]);
                            push_v([gl_rect.max.x, gl_rect.min.y], [uv_rect.max.x, uv_rect.min.y]);
                            push_v([gl_rect.min.x, gl_rect.max.y], [uv_rect.min.x, uv_rect.max.y]);
                        }
                    }
                },
                _ => unreachable!(),
            }

            let (buffer, future) = ImmutableBuffer::from_iter(vertices.into_iter(), BufferUsage::vertex_buffer(), self.queue.clone()).unwrap();
            self.future = Some(Box::new(self.future.take().unwrap().join(future)) as Box<_>);

            command_buffer_builder = command_buffer_builder.draw(
                self.pipeline.clone(),
                screen_dynamic_state.clone(),
                vec![buffer],
                (self.glyph_cache_image_descriptor_set.clone()),
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

layout(set = 0, binding = 0) uniform sampler2D tex;

void main() {
    // Text
    if (v_mode == uint(0)) {
        f_color = v_color * vec4(1.0, 1.0, 1.0, texture(tex, v_tex_coords).r);

    // 2D Geometry
    } else if (v_mode == uint(1)) {
        f_color = v_color;
    }
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
        1
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
                depth_stencil: None,
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
