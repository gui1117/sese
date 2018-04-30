use specs::Join;

pub struct RocketSystem;

impl<'a> ::specs::System<'a> for RocketSystem {
    type SystemData = (
        ::specs::ReadStorage<'a, ::component::Player>,
        ::specs::ReadStorage<'a, ::component::PhysicBody>,
        ::specs::WriteStorage<'a, ::component::RocketControl>,
        ::specs::Fetch<'a, ::resource::PhysicWorld>,
    );

    fn run(
        &mut self,
        (
            players,
            bodies,
            mut rockets,
            physic_world,
        ): Self::SystemData,
    ) {
        for (rocket, body) in (&mut rockets, &bodies).join() {
            let position = body.get(&physic_world).position().translation.vector;
            rocket.direction = (&players, &bodies).join()
                .map(|(_, player_body)| {
                    player_body.get(&physic_world).position().translation.vector - position
                })
                .min_by_key(|v| (v.norm()*10000.0) as usize)
                .map(|v| v.normalize())
                .unwrap_or(::na::zero());
        }
    }
}
