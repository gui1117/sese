#[macro_use]
extern crate derive_deref;
#[macro_use]
extern crate failure;
extern crate fps_counter;
extern crate gilrs;
#[macro_use]
extern crate lazy_static;
extern crate nalgebra as na;
extern crate nphysics3d as nphysics;
extern crate ncollide;
extern crate png;
extern crate rand;
extern crate ron;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate specs;
#[macro_use]
extern crate vulkano;
#[macro_use]
extern crate vulkano_shader_derive;
extern crate vulkano_win;
extern crate winit;
extern crate alga;
#[macro_use]
extern crate conrod;
extern crate show_message;

mod component;
mod configuration;
mod resource;
#[macro_use]
mod util;
mod game_state;
mod graphics;
mod retained_storage;

pub use configuration::CFG;

use retained_storage::Retained;
use show_message::OkOrShow;
use game_state::GameState;
use vulkano_win::VkSurfaceBuild;
use vulkano::instance::Instance;
use std::time::Duration;
use std::time::Instant;
use std::thread;
use specs::{DispatcherBuilder, World, Join};

fn main() {
    let mut gilrs = gilrs::Gilrs::new()
        .ok_or_show(|e| format!("Failed to initialize gilrs: {}\n\n{:#?}", e, e));

    let instance = {
        let extensions = vulkano_win::required_extensions();
        let info = app_info_from_cargo_toml!();
        Instance::new(Some(&info), &extensions, None)
            .ok_or_show(|e| format!("Failed to create Vulkan instance.\nPlease see if you graphic cards support Vulkan and if so update your drivers\n\n{}", e))
    };

    let mut events_loop = winit::EventsLoop::new();
    let window = winit::WindowBuilder::new()
        .with_fullscreen(Some(events_loop.get_primary_monitor()))
        .build_vk_surface(&events_loop, instance.clone())
        .ok_or_show(|e| format!("Failed to build vulkan window: {}\n\n{:#?}", e, e));

    try_multiple_time!(window.window().set_cursor_state(winit::CursorState::Grab))
        .ok_or_show(|e| format!("Failed to grab cursor: {}", e));
    window.window().set_cursor(winit::MouseCursor::NoneCursor);

    let mut graphics = graphics::Graphics::new(&window);
    // TODO: set width and height
    let mut ui = conrod::UiBuilder::new([1920.0, 1080.0])
        .build();
    let ids = game_state::Ids::new(ui.widget_id_generator());

    let mut world = World::new();
    world.register::<::component::RigidBody>();
    world.add_resource(::resource::UpdateTime(0.0));
    world.add_resource(::resource::PhysicWorld::new());
    world.maintain();

    let mut update_dispatcher = DispatcherBuilder::new()
        // .add(::system::PhysicSystem, "physic", &[])
        // .add(::system::GravitySystem, "gravity", &[])
        .add_barrier() // Draw barrier
        // .add(::system::AnimationSystem, "animation", &[])
        .build();

    let frame_duration = Duration::new(0, (1_000_000_000.0 / ::CFG.fps as f32) as u32);
    let mut fps_counter = fps_counter::FPSCounter::new();
    let mut last_frame_instant = Instant::now();
    let mut last_update_instant = Instant::now();

    let mut game_state = Box::new(game_state::Game) as Box<GameState>;

    // TODO: load map

    'main_loop: loop {
        // Parse events
        let mut evs = vec![];
        events_loop.poll_events(|ev| {
            evs.push(ev);
        });
        for ev in evs {
            match ev {
                // FIXME: this should be in winit I think
                winit::Event::WindowEvent {
                    event: winit::WindowEvent::Focused(true),
                    ..
                } => {
                    try_multiple_time!(window.window().set_cursor_state(winit::CursorState::Normal))
                        .ok_or_show(|e| format!("Failed to reset cursor: {}", e));
                    try_multiple_time!(window.window().set_cursor_state(winit::CursorState::Grab))
                        .ok_or_show(|e| format!("Failed to grab cursor: {}", e));
                }
                winit::Event::WindowEvent {
                    event: ::winit::WindowEvent::Closed,
                    ..
                } => {
                    break 'main_loop;
                }
                _ => (),
            }
            game_state = game_state.winit_event(ev, &mut world, &mut ui);
        }
        while let Some(ev) = gilrs.next_event() {
            gilrs.update(&ev);
            game_state = game_state.gilrs_event(ev.event, &mut world, &mut ui);
        }
        for (id, gamepad) in gilrs.gamepads() {
            game_state = game_state.gilrs_gamepad_state(id, gamepad, &mut world, &mut ui);
        }

        // Quit
        if game_state.quit() {
            break 'main_loop;
        }

        // Update
        let delta_time = last_update_instant.elapsed();
        last_update_instant = Instant::now();
        world.write_resource::<::resource::UpdateTime>().0 = delta_time
            .as_secs()
            .saturating_mul(1_000_000_000)
            .saturating_add(delta_time.subsec_nanos() as u64)
            as f32 / 1_000_000_000.0;

        update_dispatcher.dispatch(&mut world.res);
        game_state = game_state.update_draw_ui(&mut ui.set_widgets(), &ids, &mut world);
        world.add_resource(ui.draw().owned());

        // Maintain world and synchronize physic world
        world.maintain();
        {
            let mut physic_world = world.write_resource::<::resource::PhysicWorld>();
            for body in world.write::<::component::RigidBody>().retained() {
                physic_world.remove_rigid_body(body.handle());
            }
        }

        // Draw
        graphics.draw(&mut world, &window);

        // Sleep
        let elapsed = last_frame_instant.elapsed();
        if let Some(to_sleep) = frame_duration.checked_sub(elapsed) {
            thread::sleep(to_sleep);
        }
        last_frame_instant = Instant::now();
        fps_counter.tick();
    }
}
