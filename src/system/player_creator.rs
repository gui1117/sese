pub struct PlayerCreatorSystem;

// TODO: avec un temps avant de le respawn
impl<'a> ::specs::System<'a> for PlayerCreatorSystem {
    type SystemData = (
        ::specs::Fetch<'a, ::resource::Mode>,
        ::specs::Fetch<'a, ::resource::PlayersEntities>,
        ::specs::Fetch<'a, ::resource::PlayersControllers>,
        ::specs::Fetch<'a, ::specs::LazyUpdate>,
        ::specs::Fetch<'a, ::specs::EntitiesRes>,
    );

    fn run(
        &mut self,
        (
            mode,
            players_entities,
            players_controllers,
            lazy_update,
            entities,
        ): Self::SystemData,
    ) {
        for player in mode.number_of_player()..3 {
            assert!(players_entities[player].is_none());
            assert!(players_controllers[player].is_none());
        }

        for player in 0..mode.number_of_player() {
            if players_entities[player].map_or(true, |entity| entities.is_alive(entity)) {
                lazy_update.execute(move |world| {
                    let player_pos = ::na::Vector3::new(
                        -10,
                        -10+player as isize *2,
                        -10,
                    );
                    ::entity::create_player(::util::to_world(&player_pos, 1.0), world);
                });
            }
        }
    }
}
