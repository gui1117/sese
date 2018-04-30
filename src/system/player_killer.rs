use specs::Join;

pub struct PlayerKillerSystem;

impl<'a> ::specs::System<'a> for PlayerKillerSystem {
    type SystemData = (
        ::specs::ReadStorage<'a, ::component::Player>,
        ::specs::ReadStorage<'a, ::component::Contactor>,
        ::specs::ReadStorage<'a, ::component::PlayerKiller>,
        ::specs::Entities<'a>,
    );

    fn run(
        &mut self,
        (
            players,
            contactors,
            player_killers,
            entities,
        ): Self::SystemData,
    ) {
        for (_, contactor, entity) in (&player_killers, &contactors, &*entities).join() {
            if !contactor.contacts.is_empty() {
                for player in contactor.contacts.iter()
                    .map(|&(entity, _)| entity)
                    .filter(|&entity| players.get(entity).is_some())
                    .collect::<Vec<_>>()
                {
                    entities.delete(player).unwrap();
                }
                entities.delete(entity).unwrap();
            }
        }
    }
}
