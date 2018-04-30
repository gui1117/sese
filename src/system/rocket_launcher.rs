use specs::Join;

pub struct RocketLauncherSystem;

impl<'a> ::specs::System<'a> for RocketLauncherSystem {
    type SystemData = (
        ::specs::WriteStorage<'a, ::component::RocketLauncher>,
        ::specs::Fetch<'a, ::resource::UpdateTime>,
        ::specs::Fetch<'a, ::specs::LazyUpdate>,
    );

    fn run(
        &mut self,
        (
            mut rocket_launchers,
            update_time,
            lazy_update,
        ): Self::SystemData,
    ) {
        for rocket_launcher in (&mut rocket_launchers).join() {
            rocket_launcher.timer -= update_time.0;
            if rocket_launcher.timer <= 0.0 {
                let position = rocket_launcher.position;
                lazy_update.execute(move |world| {
                    ::entity::create_rocket(position, world);
                });
                rocket_launcher.timer = ::CFG.rocket_launcher_timer;
            }
        }
    }
}
