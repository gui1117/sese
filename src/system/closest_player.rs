use specs::Join;

pub struct ClosestPlayerSystem;

impl<'a> ::specs::System<'a> for ClosestPlayerSystem {
    type SystemData = (
        ::specs::ReadStorage<'a, ::component::Player>,
        ::specs::ReadStorage<'a, ::component::PhysicBody>,
        ::specs::WriteStorage<'a, ::component::ClosestPlayer>,
        ::specs::Fetch<'a, ::resource::PhysicWorld>,
    );

    fn run(
        &mut self,
        (
            players,
            bodies,
            mut closest_players,
            physic_world,
        ): Self::SystemData,
    ) {
        for (closest_player, body) in (&mut closest_players, &bodies).join() {
            let position = body.get(&physic_world).position().translation.vector;
            closest_player.vector = (&players, &bodies).join()
                .map(|(_, player_body)| {
                    player_body.get(&physic_world).position().translation.vector - position
                })
                .min_by_key(|v| (v.norm()*10000.0) as usize);
        }
    }
}
