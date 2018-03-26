use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::swapchain::{self, Swapchain, SwapchainCreationError, Surface};
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};
use vulkano::image::{AttachmentImage, Dimensions, ImageUsage, ImmutableImage};
use vulkano::buffer::{BufferUsage, CpuBufferPool, ImmutableBuffer};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, LayoutAttachmentDescription,
                           LayoutPassDependencyDescription, LayoutPassDescription, LoadOp,
                           RenderPassAbstract, RenderPassDesc,
                           RenderPassDescClearValues, StoreOp};
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::pipeline::viewport::Viewport;
use vulkano::descriptor::descriptor_set::{DescriptorSet, FixedSizeDescriptorSetsPool,
                                          PersistentDescriptorSet};
use vulkano::command_buffer::pool::standard::StandardCommandPoolAlloc;
use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder, DynamicState};
use vulkano::instance::PhysicalDevice;
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
use specs::World;

pub struct Graphics {
}

impl Graphics {
    pub fn new(window: &Arc<Surface<::winit::Window>>) -> Graphics {
        Graphics {
        }
    }

    pub fn draw(&mut self, world: &mut World, window: &Arc<Surface<::winit::Window>>) {
    }
}
