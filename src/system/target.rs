use specs::Join;
use ncollide::shape::Shape;

pub struct TargetSystem;

impl<'a> ::specs::System<'a> for TargetSystem {
    type SystemData = (
        ::specs::ReadStorage<'a, ::component::Player>,
        ::specs::ReadStorage<'a, ::component::PhysicBody>,
        ::specs::ReadStorage<'a, ::component::PhysicSensor>,
        ::specs::Fetch<'a, ::specs::LazyUpdate>,
        ::specs::Fetch<'a, ::resource::PhysicWorld>,
    );

    fn run(
        &mut self,
        (
            players,
            bodies,
            sensors,
            lazy_update,
            physic_world,
        ): Self::SystemData,
    ) {

        let mut wall_group = ::nphysics::object::SensorCollisionGroups::new();
        wall_group.set_membership(&[::entity::Group::Wall as usize]);
        wall_group.set_whitelist(&[::entity::Group::Wall as usize]);
        let mut group = ::nphysics::object::RigidBodyCollisionGroups::new_dynamic();
        group.set_membership(&[3]);

        let mut target_group = ::nphysics::object::SensorCollisionGroups::new();
        target_group.set_membership(&[::entity::Group::Target as usize]);
        target_group.set_whitelist(&[::entity::Group::Target as usize]);

        let (shape, position) = match players.join().count() {
            1 => {
                let shape = ::ncollide::shape::Ball::new(::CFG.ball_radius);
                let position = (&players, &bodies).join().next().unwrap().1.get(&physic_world).position();
                (shape, position)
            },
            2 => {
                // TODO:
                unimplemented!();
            },
            3 => {
                // TODO:
                unimplemented!();
            },
            _ => unreachable!(),
        };

        if physic_world.collision_world().interferences_with_aabb(&shape.aabb(position), wall_group.as_collision_groups())
            .filter(|co| ::ncollide::query::proximity(&co.position, &*co.shape, &position, &shape, 0.0)  == ::ncollide::query::Proximity::Intersecting)
            .next()
            .is_none()
        {
            let entities_to_kill = physic_world.collision_world().interferences_with_aabb(&shape.aabb(position), target_group.as_collision_groups())
                .filter(|co| ::ncollide::query::proximity(&co.position, &*co.shape, &position, &shape, 0.0)  == ::ncollide::query::Proximity::Intersecting)
                .map(|co| ::component::physic_world_object_entity(&co.data, &physic_world))
                .collect::<Vec<_>>();

            lazy_update.execute(move |world| {
                let mut physic_world = world.write_resource();
                let mut entities = world.entities();
                for entity in entities_to_kill {
                    assert!(world.read::<::component::PhysicBody>().get(entity).is_none());
                    world.read::<::component::PhysicSensor>()
                        .get(entity).unwrap()
                        .remove(&mut physic_world);
                    entities.delete(entity).unwrap();
                }
            });
        }
    }
}
