use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::swapchain::{self, Surface, Swapchain, SwapchainCreationError};
use vulkano::sampler::Sampler;
use vulkano::image::{Dimensions, ImageUsage, ImmutableImage};
use vulkano::image::attachment::AttachmentImage;
use vulkano::image::swapchain::SwapchainImage;
use vulkano::buffer::{BufferUsage, CpuBufferPool, ImmutableBuffer};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, LayoutAttachmentDescription,
                           LayoutPassDependencyDescription, LayoutPassDescription, LoadOp,
                           RenderPass, RenderPassDesc, RenderPassDescClearValues, StoreOp};
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::pipeline::viewport::Viewport;
use vulkano::descriptor::descriptor_set::{DescriptorSet, FixedSizeDescriptorSetsPool,
                                          PersistentDescriptorSet};
use vulkano::command_buffer::pool::standard::StandardCommandPoolAlloc;
use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder, DynamicState};
use vulkano::instance::{PhysicalDevice, PhysicalDeviceType};
use vulkano::sync::{now, GpuFuture};
use vulkano::image::ImageLayout;
use vulkano::format::{ClearValue, Format};
use vulkano;
use alga::general::SubsetOf;
use rand::{thread_rng, Rng};
use std::sync::Arc;
use std::time::Duration;
use specs::{Join, World};
use show_message::{OkOrShow, SomeOrShow};
use std::f32::consts::PI;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}
impl_vertex!(Vertex, position, tex_coords);

impl Vertex {
    pub fn new(position: [f32; 3], tex_coords: [f32; 2]) -> Self {
        Vertex {
            position,
            tex_coords,
        }
    }
}

pub struct Graphics {
    pub queue: Arc<Queue>,
    pub device: Arc<Device>,
    pub swapchain: Arc<Swapchain<::winit::Window>>,
    pub render_pass: Arc<RenderPass<CustomRenderPassDesc>>,
    pub pipeline: Arc<GraphicsPipelineAbstract + Sync + Send>,
    pub framebuffers: Vec<Arc<FramebufferAbstract + Sync + Send>>,

    pub camera_descriptor_sets_pool:
        FixedSizeDescriptorSetsPool<Arc<GraphicsPipelineAbstract + Sync + Send>>,
    pub view_buffer_pool: CpuBufferPool<vs::ty::View>,
    pub perspective_buffer_pool: CpuBufferPool<vs::ty::Perspective>,
    pub model_descriptor_sets_pool:
        FixedSizeDescriptorSetsPool<Arc<GraphicsPipelineAbstract + Sync + Send>>,
    pub model_buffer_pool: CpuBufferPool<vs::ty::Model>,
    pub cuboid_vertex_buffer: Arc<ImmutableBuffer<[Vertex]>>,
    pub cylinder_vertex_buffer: Arc<ImmutableBuffer<[Vertex]>>,

    pub unlocal_texture_descriptor_set: Arc<DescriptorSet + Send + Sync + 'static>,

    // TODO: maybe use an array
    pub tile_assets: HashMap<::tile::TileSize, (Arc<DescriptorSet + Send + Sync + 'static>, Arc<ImmutableBuffer<[Vertex]>>, [f32; 3])>,
    pub tube_assets: HashMap<::tube::TubeSize, (Arc<DescriptorSet + Send + Sync + 'static>, Arc<ImmutableBuffer<[Vertex]>>, [f32; 3])>,

    future: Option<Box<GpuFuture>>,
}

// TODO: return result failure ?
impl Graphics {
    pub fn framebuffers_and_descriptors(
        device: &Arc<Device>,
        images: &Vec<Arc<SwapchainImage<::winit::Window>>>,
        render_pass: &Arc<RenderPass<CustomRenderPassDesc>>,
    ) -> (Vec<Arc<FramebufferAbstract + Sync + Send>>, ()) {
        // FIXME: one depth buffer for each image ?
        let depth_buffer_attachment =
            AttachmentImage::transient(device.clone(), images[0].dimensions(), Format::D16Unorm)
                .unwrap();

        let framebuffers = images
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

        let queue = queues
            .next()
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
                .triangle_list()
                .cull_mode_back()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .depth_stencil_simple_depth()
                .blend_alpha_blending()
                .render_pass(vulkano::framebuffer::Subpass::from(render_pass.clone(), 0).unwrap())
                .build(device.clone())
                .unwrap(),
        ) as Arc<GraphicsPipelineAbstract + Send + Sync>;

        let camera_descriptor_sets_pool = FixedSizeDescriptorSetsPool::new(pipeline.clone(), 0);
        let view_buffer_pool =
            CpuBufferPool::<vs::ty::View>::new(device.clone(), BufferUsage::uniform_buffer());
        let perspective_buffer_pool = CpuBufferPool::<vs::ty::Perspective>::new(
            device.clone(),
            BufferUsage::uniform_buffer(),
        );

        let model_descriptor_sets_pool = FixedSizeDescriptorSetsPool::new(pipeline.clone(), 1);
        let model_buffer_pool =
            CpuBufferPool::<vs::ty::Model>::new(device.clone(), BufferUsage::uniform_buffer());

        let (cuboid_vertex_buffer, _future) = ImmutableBuffer::from_iter(
            [
                Vertex::new([1.0, -1.0, -1.0], [1.0, 0.0]),
                Vertex::new([-1.0, -1.0, -1.0], [0.0, 0.0]),
                Vertex::new([-1.0, 1.0, -1.0], [0.0, 1.0]),
                Vertex::new([1.0, 1.0, -1.0], [1.0, 1.0]),
                Vertex::new([1.0, -1.0, -1.0], [1.0, 0.0]),
                Vertex::new([-1.0, 1.0, -1.0], [0.0, 1.0]),
                Vertex::new([-1.0, -1.0, 1.0], [0.0, 0.0]),
                Vertex::new([1.0, -1.0, 1.0], [1.0, 0.0]),
                Vertex::new([-1.0, 1.0, 1.0], [0.0, 1.0]),
                Vertex::new([1.0, -1.0, 1.0], [1.0, 0.0]),
                Vertex::new([1.0, 1.0, 1.0], [1.0, 1.0]),
                Vertex::new([-1.0, 1.0, 1.0], [0.0, 1.0]),
                Vertex::new([-1.0, -1.0, -1.0], [0.0, 0.0]),
                Vertex::new([-1.0, -1.0, 1.0], [0.0, 1.0]),
                Vertex::new([-1.0, 1.0, -1.0], [1.0, 0.0]),
                Vertex::new([-1.0, -1.0, 1.0], [0.0, 1.0]),
                Vertex::new([-1.0, 1.0, 1.0], [1.0, 1.0]),
                Vertex::new([-1.0, 1.0, -1.0], [1.0, 0.0]),
                Vertex::new([1.0, -1.0, 1.0], [0.0, 1.0]),
                Vertex::new([1.0, -1.0, -1.0], [0.0, 0.0]),
                Vertex::new([1.0, 1.0, -1.0], [1.0, 0.0]),
                Vertex::new([1.0, 1.0, 1.0], [1.0, 1.0]),
                Vertex::new([1.0, -1.0, 1.0], [0.0, 1.0]),
                Vertex::new([1.0, 1.0, -1.0], [1.0, 0.0]),
                Vertex::new([-1.0, -1.0, -1.0], [0.0, 0.0]),
                Vertex::new([1.0, -1.0, -1.0], [1.0, 0.0]),
                Vertex::new([-1.0, -1.0, 1.0], [0.0, 1.0]),
                Vertex::new([1.0, -1.0, 1.0], [1.0, 1.0]),
                Vertex::new([-1.0, -1.0, 1.0], [0.0, 1.0]),
                Vertex::new([1.0, -1.0, -1.0], [1.0, 0.0]),
                Vertex::new([1.0, 1.0, -1.0], [1.0, 0.0]),
                Vertex::new([-1.0, 1.0, -1.0], [0.0, 0.0]),
                Vertex::new([-1.0, 1.0, 1.0], [0.0, 1.0]),
                Vertex::new([-1.0, 1.0, 1.0], [0.0, 1.0]),
                Vertex::new([1.0, 1.0, 1.0], [1.0, 1.0]),
                Vertex::new([1.0, 1.0, -1.0], [1.0, 0.0]),
            ].iter()
                .cloned(),
            BufferUsage::vertex_buffer(),
            queue.clone(),
        ).unwrap();

        let (cylinder_vertex_buffer, _future) = ImmutableBuffer::from_iter(
            {
                let cylinder_div = 32;
                let mut vertex = vec![];
                for i in 0..cylinder_div {
                    let a0 = (i as f32) * 2.0 * PI / cylinder_div as f32;
                    let a1 = ((i + 1) as f32) * 2.0 * PI / cylinder_div as f32;

                    let p0 = [a0.cos(), a0.sin()];
                    let p1 = [a1.cos(), a1.sin()];

                    vertex.push(Vertex::new([p0[0], -1.0, p0[1]], [0.0, 0.0]));
                    vertex.push(Vertex::new([p1[0], -1.0, p1[1]], [0.0, 0.0]));
                    vertex.push(Vertex::new([0.0, -1.0, 0.0], [0.0, 0.0]));

                    vertex.push(Vertex::new([p1[0], 1.0, p1[1]], [0.0, 0.0]));
                    vertex.push(Vertex::new([p0[0], 1.0, p0[1]], [0.0, 0.0]));
                    vertex.push(Vertex::new([0.0, 1.0, 0.0], [0.0, 0.0]));

                    vertex.push(Vertex::new([p0[0], -1.0, p0[1]], [0.0, 0.0]));
                    vertex.push(Vertex::new([p0[0], 1.0, p0[1]], [0.0, 0.0]));
                    vertex.push(Vertex::new([p1[0], 1.0, p1[1]], [0.0, 0.0]));

                    vertex.push(Vertex::new([p1[0], -1.0, p1[1]], [0.0, 0.0]));
                    vertex.push(Vertex::new([p0[0], -1.0, p0[1]], [0.0, 0.0]));
                    vertex.push(Vertex::new([p1[0], 1.0, p1[1]], [0.0, 0.0]));
                }
                vertex
            }.iter()
                .cloned(),
            BufferUsage::vertex_buffer(),
            queue.clone(),
        ).unwrap();

        let (framebuffers, ()) =
            Graphics::framebuffers_and_descriptors(&device, &images, &render_pass);

        // IDEA: all are good but
        // gaussian is too grey
        // nearest looks pixelized
        let texture_generation_filter = ::image::FilterType::Lanczos3;

        let (unlocal_texture, _future) = {
            let dimensions = Dimensions::Dim2d {
                width: ::CFG.unlocal_texture_size,
                height: ::CFG.unlocal_texture_size,
            };

            let image = ::texture::generate_texture(
                dimensions.width(),
                dimensions.height(),
                ::CFG.unlocal_texture_layers,
                texture_generation_filter,
                false,
            );

            ImmutableImage::from_iter(
                image.into_raw().iter().cloned(),
                dimensions,
                Format::R8Unorm,
                queue.clone(),
            ).unwrap()
        };

        let mut tile_assets = HashMap::new();
        let mut _futures = (vec![], vec![]);
        for tile_size in ::tile::TileSize::iter_variants() {
            let dimensions = Dimensions::Dim2d {
                width: ::CFG.unlocal_texture_size * tile_size.width() as u32,
                height: ::CFG.unlocal_texture_size * tile_size.height() as u32,
            };

            let image = ::texture::generate_texture(
                dimensions.width(),
                dimensions.height(),
                ::CFG.unlocal_texture_layers,
                texture_generation_filter,
                false,
            );

            let (texture, future) = ImmutableImage::from_iter(
                image.into_raw().iter().cloned(),
                dimensions,
                Format::R8Unorm,
                queue.clone(),
            ).unwrap();
            _futures.0.push(future);

            let texture_descriptor_set = PersistentDescriptorSet::start(pipeline.clone(), 2)
                .add_sampled_image(texture, Sampler::simple_repeat_linear(device.clone()))
                .unwrap()
                .build()
                .unwrap();

            let texture_descriptor_set = Arc::new(texture_descriptor_set) as Arc<_>;

            let (vertex_buffer, future) = ImmutableBuffer::from_iter(
                ::obj::generate_tile(tile_size.width(), tile_size.height())
                    .iter()
                    .cloned(),
                BufferUsage::vertex_buffer(),
                queue.clone(),
            ).unwrap();

            _futures.1.push(future);

            tile_assets.insert(
                tile_size.clone(),
                (texture_descriptor_set, vertex_buffer, [0.0; 3]),
            );
        }

        let mut tube_assets = HashMap::new();
        for tube_size in ::tube::TubeSize::iter_variants() {
            let dimensions = Dimensions::Dim2d {
                width: (::CFG.unlocal_texture_size as f32 * 2.0 * PI * ::CFG.column_outer_radius)
                    as u32,
                height: ::CFG.unlocal_texture_size * tube_size.size() as u32,
            };

            let image = ::texture::generate_texture(
                dimensions.width(),
                dimensions.height(),
                ::CFG.unlocal_texture_layers,
                texture_generation_filter,
                true,
            );

            let (texture, future) = ImmutableImage::from_iter(
                image.into_raw().iter().cloned(),
                dimensions,
                Format::R8Unorm,
                queue.clone(),
            ).unwrap();
            _futures.0.push(future);

            let texture_descriptor_set = PersistentDescriptorSet::start(pipeline.clone(), 2)
                .add_sampled_image(texture, Sampler::simple_repeat_linear(device.clone()))
                .unwrap()
                .build()
                .unwrap();

            let texture_descriptor_set = Arc::new(texture_descriptor_set) as Arc<_>;

            let (vertex_buffer, future) = ImmutableBuffer::from_iter(
                ::obj::generate_tube(tube_size.size()).iter().cloned(),
                BufferUsage::vertex_buffer(),
                queue.clone(),
            ).unwrap();

            _futures.1.push(future);

            tube_assets.insert(
                tube_size.clone(),
                (texture_descriptor_set, vertex_buffer, [0.0; 3]),
            );
        }

        let unlocal_texture_descriptor_set = PersistentDescriptorSet::start(pipeline.clone(), 2)
            .add_sampled_image(
                unlocal_texture,
                Sampler::simple_repeat_linear(device.clone()),
            )
            .unwrap()
            .build()
            .unwrap();

        let unlocal_texture_descriptor_set = Arc::new(unlocal_texture_descriptor_set) as Arc<_>;

        let future = Some(Box::new(now(device.clone())) as Box<_>);

        let mut graphics = Graphics {
            future,
            device,
            queue,
            swapchain,
            render_pass,
            framebuffers,
            pipeline,

            camera_descriptor_sets_pool,
            view_buffer_pool,
            perspective_buffer_pool,
            model_descriptor_sets_pool,
            model_buffer_pool,
            cuboid_vertex_buffer,
            cylinder_vertex_buffer,

            unlocal_texture_descriptor_set,
            tile_assets,
            tube_assets,
        };

        graphics.reset_colors();

        graphics
    }

    pub fn reset_colors(&mut self) {
        let mut colors = ::colors::GenPale::colors();
        thread_rng().shuffle(&mut colors);
        for tile in self.tile_assets.values_mut() {
            tile.2 = colors.pop().unwrap().into();
        }
        for column in self.tube_assets.values_mut() {
            column.2 = colors.pop().unwrap().into();
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

        let (framebuffers, ()) =
            Graphics::framebuffers_and_descriptors(&self.device, &images, &self.render_pass);
        self.framebuffers = framebuffers;
    }

    pub fn draw(
        &mut self,
        world: &mut World,
        window: &Arc<Surface<::winit::Window>>,
        game_state: Box<::game_state::GameState>,
    ) -> Box<::game_state::GameState> {
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

        let (command_buffer, game_state) = self.build_command_buffer(image_num, world, game_state);

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
                vec![[0.0, 0.0, 1.0, 1.0].into(), 1.0.into()],
            )
            .unwrap();

        // Draw world
        {
            let physic_world = world.read_resource::<::resource::PhysicWorld>();
            let physic_bodies = world.read::<::component::PhysicBody>();
            let players = world.read::<::component::Player>();

            let player_pos = (&players, &physic_bodies)
                .join()
                .next()
                .map(|(_, body)| body.get(&physic_world).position())
                .unwrap();

            let view_trans: ::na::Transform3<f32> = ::na::Similarity3::look_at_rh(
                &::na::Point3::from_coordinates(
                    player_pos.translation.vector
                        + player_pos.rotation * ::na::Vector3::new(-0.2, 0.0, 0.2),
                ),
                &::na::Point3::from_coordinates(
                    player_pos.translation.vector
                        + player_pos.rotation * ::na::Vector3::new(0.0, 0.0, 0.2),
                ),
                &(player_pos.rotation * ::na::Vector3::z()),
                1.0,
            ).to_superset();

            let view = self.view_buffer_pool
                .next(vs::ty::View {
                    view: view_trans.unwrap().into(),
                })
                .unwrap();

            let perspective = self.perspective_buffer_pool
                .next(vs::ty::Perspective {
                    perspective: ::na::Perspective3::new(
                        dimensions[0] as f32 / dimensions[1] as f32,
                        ::std::f32::consts::FRAC_PI_3,
                        0.01,
                        100.0,
                    ).unwrap()
                        .into(),
                })
                .unwrap();

            let camera_descriptor_set = Arc::new(
                self.camera_descriptor_sets_pool
                    .next()
                    .add_buffer(perspective)
                    .unwrap()
                    .add_buffer(view)
                    .unwrap()
                    .build()
                    .unwrap(),
            );

            for tile in &world.read_resource::<::resource::Tiles>().0 {
                let (ref texture_descriptor_set, ref vertex_buffer, color) =
                    self.tile_assets[&tile.size];

                let position: ::na::Transform3<f32> = tile.position.to_superset();

                let model = self.model_buffer_pool
                    .next(vs::ty::Model {
                        model: position.unwrap().into(),
                    })
                    .unwrap();

                let model_descriptor_set = self.model_descriptor_sets_pool
                    .next()
                    .add_buffer(model)
                    .unwrap()
                    .build()
                    .unwrap();

                command_buffer_builder = command_buffer_builder
                    .draw(
                        self.pipeline.clone(),
                        screen_dynamic_state.clone(),
                        vec![vertex_buffer.clone()],
                        (
                            camera_descriptor_set.clone(),
                            model_descriptor_set,
                            texture_descriptor_set.clone(),
                        ),
                        color,
                    )
                    .unwrap();
            }

            for tube in &world.read_resource::<::resource::Tubes>().0 {
                let (ref texture_descriptor_set, ref vertex_buffer, color) =
                    self.tube_assets[&tube.tube_size];

                let position: ::na::Transform3<f32> = tube.position.to_superset();

                let model = self.model_buffer_pool
                    .next(vs::ty::Model {
                        model: position.unwrap().into(),
                    })
                    .unwrap();

                let model_descriptor_set = self.model_descriptor_sets_pool
                    .next()
                    .add_buffer(model)
                    .unwrap()
                    .build()
                    .unwrap();

                command_buffer_builder = command_buffer_builder
                    .draw(
                        self.pipeline.clone(),
                        screen_dynamic_state.clone(),
                        vec![vertex_buffer.clone()],
                        (
                            camera_descriptor_set.clone(),
                            model_descriptor_set,
                            texture_descriptor_set.clone(),
                        ),
                        color,
                    )
                    .unwrap();
            }

            // Draw physic world
            if true {
                for body in physic_bodies.join() {
                    let body = body.get(&physic_world);
                    let shape = body.shape();
                    if let Some(_shape) = shape.as_shape::<::ncollide::shape::Ball<f32>>() {
                        // TODO
                    } else if let Some(shape) = shape.as_shape::<::ncollide::shape::Cylinder<f32>>()
                    {
                        let primitive_trans = ::na::Matrix4::from_diagonal(&::na::Vector4::new(
                            shape.radius(),
                            shape.half_height(),
                            shape.radius(),
                            1.0,
                        ));

                        let position: ::na::Transform3<f32> = body.position().to_superset();

                        let model = self.model_buffer_pool
                            .next(vs::ty::Model {
                                model: (position.unwrap() * primitive_trans).into(),
                            })
                            .unwrap();

                        let model_descriptor_set = self.model_descriptor_sets_pool
                            .next()
                            .add_buffer(model)
                            .unwrap()
                            .build()
                            .unwrap();

                        command_buffer_builder = command_buffer_builder
                            .draw(
                                self.pipeline.clone(),
                                screen_dynamic_state.clone(),
                                vec![self.cylinder_vertex_buffer.clone()],
                                (
                                    camera_descriptor_set.clone(),
                                    model_descriptor_set,
                                    self.unlocal_texture_descriptor_set.clone(),
                                ),
                                [1.0f32, 1.0, 1.0],
                            )
                            .unwrap();
                    } else if let Some(shape) =
                        shape.as_shape::<::ncollide::shape::Cuboid<::na::Vector3<f32>>>()
                    {
                        let radius = shape.half_extents();
                        let primitive_trans = ::na::Matrix4::from_diagonal(&::na::Vector4::new(
                            radius[0],
                            radius[1],
                            radius[2],
                            1.0,
                        ));

                        let position: ::na::Transform3<f32> = body.position().to_superset();

                        let model = self.model_buffer_pool
                            .next(vs::ty::Model {
                                model: (position.unwrap() * primitive_trans).into(),
                            })
                            .unwrap();

                        let model_descriptor_set = self.model_descriptor_sets_pool
                            .next()
                            .add_buffer(model)
                            .unwrap()
                            .build()
                            .unwrap();

                        command_buffer_builder = command_buffer_builder
                            .draw(
                                self.pipeline.clone(),
                                screen_dynamic_state.clone(),
                                vec![self.cuboid_vertex_buffer.clone()],
                                (
                                    camera_descriptor_set.clone(),
                                    model_descriptor_set,
                                    self.unlocal_texture_descriptor_set.clone(),
                                ),
                                [1.0f32, 1.0, 1.0],
                            )
                            .unwrap();
                    }
                }
            }
        }

        // TODO: Draw UI
        let next_game_state = game_state.update_draw_ui(world);

        let command = command_buffer_builder
            .end_render_pass()
            .unwrap()
            .build()
            .unwrap();

        (command, next_game_state)
    }
}

mod vs {
    #[derive(VulkanoShader)]
    #[ty = "vertex"]
    #[src = "

#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 tex_coords;

layout(location = 0) out vec2 v_tex_coords;

layout(set = 0, binding = 0) uniform Perspective {
    mat4 perspective;
} perspective;
layout(set = 0, binding = 1) uniform View {
    mat4 view;
} view;
layout(set = 1, binding = 0) uniform Model {
    mat4 model;
} model;

void main() {
    // TODO: make perspective/view multiplication on the cpu
    gl_Position = perspective.perspective * view.view * model.model * vec4(position, 1.0);
    gl_Position.y = - gl_Position.y;
    v_tex_coords = tex_coords;
}
    "]
    struct _Dummy;
}

mod fs {
    #[derive(VulkanoShader)]
    #[ty = "fragment"]
    #[src = "

#version 450

layout(push_constant) uniform Color {
    vec3 color;
} color;

layout(location = 0) in vec2 v_tex_coords;

layout(location = 0) out vec4 out_color;

layout(set = 2, binding = 0) uniform sampler2D tex;

void main() {
    vec3 black = vec3(0.0, 0.0, 0.0);
    float grey = texture(tex, v_tex_coords).r;
    out_color = vec4(black*grey + color.color*(1.0 - grey), 1.0);
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
