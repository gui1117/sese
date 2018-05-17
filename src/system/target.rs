use specs::Join;
use ncollide::shape::Shape;

pub struct TargetSystem;

impl<'a> ::specs::System<'a> for TargetSystem {
    type SystemData = (
        ::specs::ReadStorage<'a, ::component::Player>,
        ::specs::ReadStorage<'a, ::component::PhysicBody>,
        ::specs::Fetch<'a, ::resource::PhysicWorld>,
        ::specs::Fetch<'a, ::resource::Mode>,
        ::specs::Entities<'a>,
    );

    fn run(
        &mut self,
        (
            players,
            bodies,
            physic_world,
            mode,
            entities,
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

        if mode.number_of_player() != players.join().count() {
            return
        }

        let (shape, position) = match mode.number_of_player() {
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
            for entity in physic_world.collision_world().interferences_with_aabb(&shape.aabb(position), target_group.as_collision_groups())
                .filter(|co| ::ncollide::query::proximity(&co.position, &*co.shape, &position, &shape, 0.0)  == ::ncollide::query::Proximity::Intersecting)
                .map(|co| ::component::physic_world_object_entity(&co.data, &physic_world))
            {
                entities.delete(entity).unwrap();
            }
        }
    }
}
