use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::swapchain::{self, Swapchain, SwapchainCreationError, Surface};
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};
use vulkano::image::{AttachmentImage, Dimensions, ImageUsage, ImmutableImage, ImageAccess, StorageImage};
use vulkano::image::swapchain::SwapchainImage;
use vulkano::buffer::{BufferUsage, CpuBufferPool, ImmutableBuffer, CpuAccessibleBuffer};
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
use std::sync::Arc;
use std::fs::File;
use std::time::Duration;
use std::f32::consts::PI;
use std::cell::RefCell;
use specs::World;
use show_message::{OkOrShow, SomeOrShow};

#[derive(Debug, Clone)]
pub struct ImGuiVertex {
    pos: [f32; 2],
    uv: [f32; 2],
    col: [f32; 4],
}
impl_vertex!(ImGuiVertex, pos, uv, col);
impl ImGuiVertex {
    fn from_im_draw_vert(vertex: &::imgui::ImDrawVert) -> Self {
        let r = vertex.col as u8 as f32;
        let g = (vertex.col >> 8) as u8 as f32;
        let b = (vertex.col >> 16) as u8 as f32;
        let a = (vertex.col >> 24) as u8 as f32;
        ImGuiVertex {
            pos: [vertex.pos.x, vertex.pos.y],
            uv: [vertex.uv.x, vertex.uv.y],
            col: [r, g, b, a],
        }
    }
}

pub struct Graphics {
    pub queue: Arc<Queue>,
    pub device: Arc<Device>,
    pub swapchain: Arc<Swapchain<::winit::Window>>,
    pub render_pass: Arc<RenderPass<CustomRenderPassDesc>>,
    pub imgui_pipeline: Arc<GraphicsPipelineAbstract + Sync + Send>,
    pub framebuffers: Vec<Arc<FramebufferAbstract + Sync + Send>>,

    pub imgui_descriptor_set: Arc<DescriptorSet + Sync + Send>,

    // view_buffer_pool: CpuBufferPool<imgui_vs::ty::View>,
    // world_buffer_pool: CpuBufferPool<imgui_vs::ty::World>,
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

    pub fn new(window: &Arc<Surface<::winit::Window>>, imgui: &mut ::imgui::ImGui, save: &mut ::resource::Save) -> Graphics {
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

        let imgui_vs = imgui_vs::Shader::load(device.clone()).expect("failed to create shader module");
        let imgui_fs = imgui_fs::Shader::load(device.clone()).expect("failed to create shader module");

        let imgui_pipeline = Arc::new(
            vulkano::pipeline::GraphicsPipeline::start()
                .vertex_input_single_buffer::<ImGuiVertex>()
                .vertex_shader(imgui_vs.main_entry_point(), ())
                .triangle_list()
                .cull_mode_front()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(imgui_fs.main_entry_point(), ())
                .blend_alpha_blending()
                .render_pass(vulkano::framebuffer::Subpass::from(render_pass.clone(), 0).unwrap())
                .build(device.clone())
                .unwrap(),
        );

        let dim = swapchain.dimensions();
        let imgui_matrix_buffer = ImmutableBuffer::from_data(
            imgui_vs::ty::Matrix {
                matrix: [
                    [2.0 / dim[0] as f32, 0.0, 0.0, 0.0],
                    [0.0, 2.0 / -(dim[1] as f32), 0.0, 0.0],
                    [0.0, 0.0, -1.0, 0.0],
                    [-1.0, 1.0, 0.0, 1.0],
                ],
            },
            BufferUsage::uniform_buffer(),
            queue.clone(),
        ).unwrap().0;

        let imgui_texture = imgui
            .prepare_texture(|handle| {
                ImmutableImage::from_iter(
                    handle.pixels.iter().cloned(),
                    Dimensions::Dim2d {
                        width: handle.width,
                        height: handle.height,
                    },
                    format::R8G8B8A8Unorm,
                    queue.clone(),
                )
            })
            .unwrap().0;

        let imgui_descriptor_set = Arc::new(PersistentDescriptorSet::start(imgui_pipeline.clone(), 0)
            .add_buffer(imgui_matrix_buffer).unwrap()
            .add_sampled_image(
                imgui_texture,
                Sampler::new(
                    device.clone(),
                    Filter::Nearest,
                    Filter::Linear,
                    MipmapMode::Linear,
                    SamplerAddressMode::MirroredRepeat,
                    SamplerAddressMode::MirroredRepeat,
                    SamplerAddressMode::MirroredRepeat,
                    0.0, 1.0, 0.0, 0.0,
                ).unwrap(),
            ).unwrap()
            .build().unwrap()
        );

        // let view_buffer_pool =
        //     CpuBufferPool::<imgui_vs::ty::View>::new(device.clone(), BufferUsage::uniform_buffer());

        // let world_buffer_pool =
        //     CpuBufferPool::<imgui_vs::ty::World>::new(device.clone(), BufferUsage::uniform_buffer());

        // let descriptor_sets_pool = FixedSizeDescriptorSetsPool::new(pipeline.clone() as Arc<_>, 0);

        let (framebuffers, ()) = Graphics::framebuffers_and_descriptors(
            &device,
            &images,
            &render_pass,
        );

        let future = Some(Box::new(now(device.clone())) as Box<_>);

        Graphics {
            future,
            device,
            queue,
            swapchain,
            render_pass,
            imgui_pipeline,
            framebuffers,
            imgui_descriptor_set,

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

    pub fn draw(&mut self, world: &mut World, window: &Arc<Surface<::winit::Window>>, game_state: Box<::game_state::GameState>) -> Box<::game_state::GameState> {
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

        let (command_buffer, game_state) = self.build_command_buffer(image_num, window.window().hidpi_factor(), world, game_state);

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
        game_state
    }

    fn build_command_buffer(
        &mut self,
        image_num: usize,
        hidpi_factor: f32,
        world: &mut World,
        game_state: Box<::game_state::GameState>,
    ) -> (
        AutoCommandBuffer<StandardCommandPoolAlloc>,
        Box<::game_state::GameState>,
    ) {

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
        let mut imgui = world.write_resource::<::resource::ImGuiOption>().take().unwrap();

        let next_game_state;
        let ref_cell_cmd_builder = RefCell::new(Some(command_buffer_builder));
        {
            let ui = imgui.frame(
                (dimensions[0], dimensions[1]),
                ((dimensions[0] as f32 * hidpi_factor) as u32, (dimensions[1] as f32 * hidpi_factor) as u32),
                world.read_resource::<::resource::UpdateTime>().0,
            );

            next_game_state = game_state.update_draw_ui(&ui, world);

            ui.render::<_, ()>(|ui, drawlist| {
                let mut cmd_builder = ref_cell_cmd_builder.borrow_mut().take().unwrap();

                let vertex_buffer = CpuAccessibleBuffer::from_iter(
                    self.device.clone(),
                    BufferUsage::vertex_buffer(),
                    drawlist.vtx_buffer
                        .iter()
                        .map(|vtx| ImGuiVertex::from_im_draw_vert(vtx)),
                ).unwrap();

                let index_buffer = CpuAccessibleBuffer::from_iter(
                    self.device.clone(),
                    BufferUsage::index_buffer(),
                    drawlist.idx_buffer.iter().cloned(),
                ).unwrap();

                for cmd in drawlist.cmd_buffer {
                    let dynamic_state = DynamicState {
                        viewports: Some(vec![
                            Viewport {
                                origin: [0.0, 0.0],
                                dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                                depth_range: 0.0..1.0,
                            },
                        ]),
                        // TODO:
                        // Scissor {
                        //     origin: [
                        //         cmd.clip_rect.x as i32,
                        //         (dimensions[1] - cmd.clip_rect.w) as i32,
                        //     ],
                        //     dimensions: [
                        //         ((cmd.clip_rect.z - cmd.clip_rect.x) * scale_width) as u32,
                        //         ((cmd.clip_rect.w - cmd.clip_rect.y) * scale_height) as u32,
                        //     ],
                        // }
                        ..DynamicState::none()
                    };


                    cmd_builder = cmd_builder
                        .draw_indexed(
                            self.imgui_pipeline.clone(),
                            screen_dynamic_state.clone(),
                            vec![vertex_buffer.clone()],
                            index_buffer.clone(),
                            self.imgui_descriptor_set.clone(),
                            (),
                        )
                        .unwrap();
                }
                *ref_cell_cmd_builder.borrow_mut() = Some(cmd_builder);
                Ok(())
            }).unwrap();
        }

        let command_buffer_builder = ref_cell_cmd_builder.borrow_mut().take().unwrap();

        let command = command_buffer_builder
            .end_render_pass()
            .unwrap()
            .build()
            .unwrap();

        *world.write_resource::<::resource::ImGuiOption>() = Some(imgui);

        (command, next_game_state)
    }

}

mod imgui_vs {
    #[derive(VulkanoShader)]
    #[ty = "vertex"]
    #[src = "

#version 450

layout(set = 0, binding = 0) uniform Matrix {
    mat4 matrix;
} matrix;

layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec4 col;

layout(location = 0) out vec2 f_uv;
layout(location = 1) out vec4 f_color;

void main() {
    f_uv = uv;
    f_color = col / 255.0;
    gl_Position = matrix.matrix * vec4(pos.xy, 0, 1);
    gl_Position.y = - gl_Position.y;
}
    "]
    struct _Dummy;
}

mod imgui_fs {
    #[derive(VulkanoShader)]
    #[ty = "fragment"]
    #[src = "

#version 450

layout(set = 0, binding = 1) uniform sampler2D tex;

layout(location = 0) in vec2 f_uv;
layout(location = 1) in vec4 f_color;

layout(location = 0) out vec4 out_color;

void main() {
  out_color = f_color * texture(tex, f_uv.st);
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
