use retained_storage::Retained;

pub trait WorldAction {
    fn safe_maintain(&mut self);
    fn reset_for_mode(&mut self);
}

// Maintain world and synchronize physic world
fn safe_maintain(world: &mut ::specs::World) {
    world.maintain();
    let mut physic_world = world.write_resource::<::resource::PhysicWorld>();
    for body in world.write::<::component::PhysicBody>().retained() {
        physic_world.remove_rigid_body(body.handle());
    }
    for sensor in world.write::<::component::PhysicBody>().retained() {
        physic_world.remove_sensor(sensor.handle());
    }
}

impl WorldAction for ::specs::World {
    fn safe_maintain(&mut self) {
        safe_maintain(self);
    }

    fn reset_for_mode(&mut self) {
        safe_maintain(self);
        {
            let mut players_controllers = self.write_resource::<::resource::PlayersControllers>();
            let mut players_entities = self.write_resource::<::resource::PlayersEntities>();
            let mode = self.read_resource::<::resource::Mode>();
            let entities = self.entities();

            for player in mode.number_of_player()..3 {
                players_controllers[player] = None;
                if let Some(entity) = players_entities[player].take() {
                    entities.delete(entity).unwrap();
                }
            }
        }
        safe_maintain(self);
    }
}
