extern crate alga;
extern crate app_dirs2;
#[macro_use]
extern crate derive_deref;
extern crate failure;
extern crate fps_counter;
extern crate generic_array;
extern crate gilrs;
#[macro_use]
extern crate lazy_static;
extern crate nalgebra as na;
extern crate ncollide;
extern crate nphysics3d as nphysics;
extern crate pathfinding;
extern crate png;
extern crate rand;
extern crate ron;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate show_message;
extern crate specs;
extern crate typenum;
#[macro_use]
extern crate vulkano;
#[macro_use]
extern crate vulkano_shader_derive;
extern crate vulkano_win;
extern crate winit;

mod component;
mod configuration;
pub mod maze;
mod resource;
#[macro_use]
mod util;
mod game_state;
mod graphics;
mod retained_storage;
mod level;
mod entity;

pub use configuration::CFG;

use retained_storage::Retained;
use show_message::OkOrShow;
use game_state::GameState;
use vulkano_win::VkSurfaceBuild;
use vulkano::instance::Instance;
use std::time::Duration;
use std::time::Instant;
use std::thread;
use specs::{DispatcherBuilder, World};

fn main() {
    ::std::env::set_var("WINIT_UNIX_BACKEND", "x11");
    let mut save = ::resource::Save::new();

    let mut gilrs = gilrs::Gilrs::new()
        .ok_or_show(|e| format!("Failed to initialize gilrs: {}\n\n{:#?}", e, e));

    let instance = {
        let extensions = vulkano_win::required_extensions();
        let info = app_info_from_cargo_toml!();
        Instance::new(Some(&info), &extensions, None).ok_or_show(|e| {
            format!("Failed to create Vulkan instance.\nPlease see if you graphic cards support Vulkan and if so update your drivers\n\n{}", e)
        })
    };

    let mut events_loop = winit::EventsLoop::new();
    let window = winit::WindowBuilder::new()
        .with_fullscreen(Some(events_loop.get_primary_monitor()))
        .build_vk_surface(&events_loop, instance.clone())
        .ok_or_show(|e| format!("Failed to build vulkan window: {}\n\n{:#?}", e, e));

    try_multiple_time!(window.window().set_cursor_state(winit::CursorState::Grab))
        .ok_or_show(|e| format!("Failed to grab cursor: {}", e));
    window.window().set_cursor(winit::MouseCursor::NoneCursor);

    let mut graphics = graphics::Graphics::new(&window, &mut save);

    let mut world = World::new();
    world.register::<::component::PhysicBody>();
    world.register::<::component::Player>();
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

    ::level::LevelBuilder {
        half_size: 3,
        x_shift: false,
        y_shift: false,
        z_shift: false,
        percent: 10.0,
    }.build(&mut world);

    'main_loop: loop {
        // Parse events
        let mut evs = vec![];
        events_loop.poll_events(|ev| {
            evs.push(ev);
        });
        for ev in evs {
            println!("{:?}", ev);
            match ev {
                // FIXME: this should be in winit I think
                winit::Event::WindowEvent {
                    event: winit::WindowEvent::Focused(true),
                    ..
                } => {
                    try_multiple_time!(window.window().set_cursor_state(winit::CursorState::Grab))
                        .ok_or_show(|e| format!("Failed to grab cursor: {}", e));
                }
                // FIXME: this should be in winit I think
                winit::Event::WindowEvent {
                    event: winit::WindowEvent::Focused(false),
                    ..
                } => {
                    try_multiple_time!(
                        window.window().set_cursor_state(winit::CursorState::Normal)
                    ).ok_or_show(|e| format!("Failed to grab cursor: {}", e));
                }
                winit::Event::WindowEvent {
                    event: ::winit::WindowEvent::Closed,
                    ..
                } => {
                    break 'main_loop;
                }
                _ => (),
            }
            game_state = game_state.winit_event(ev, &mut world);
        }
        while let Some(ev) = gilrs.next_event() {
            println!("{:?}", ev);
            gilrs.update(&ev);
            game_state = game_state.gilrs_event(ev.event, &mut world);
        }
        for (id, gamepad) in gilrs.gamepads() {
            game_state = game_state.gilrs_gamepad_state(id, gamepad, &mut world);
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

        // Maintain world and synchronize physic world
        world.maintain();
        {
            let mut physic_world = world.write_resource::<::resource::PhysicWorld>();
            for body in world.write::<::component::PhysicBody>().retained() {
                physic_world.remove_rigid_body(body.handle());
            }
        }

        // Draw
        game_state = graphics.draw(&mut world, &window, game_state);

        // Sleep
        let elapsed = last_frame_instant.elapsed();
        if let Some(to_sleep) = frame_duration.checked_sub(elapsed) {
            thread::sleep(to_sleep);
        }
        last_frame_instant = Instant::now();
        fps_counter.tick();
    }
}
