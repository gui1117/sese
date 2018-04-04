pub fn create_wall(pos: ::na::Vector3<f32>, _color: usize, world: &mut ::specs::World) {
    let shape = ::ncollide::shape::Cuboid3::new(::na::Vector3::from_element(1.0));
    let mut body = ::nphysics::object::RigidBody::new_static(shape, 0.0, 0.0);
    body.set_transformation(::na::Isometry3::new(pos, ::na::zero()));

    let entity = world.create_entity().build();

    ::component::PhysicBody::add(
        entity,
        body,
        &mut world.write(),
        &mut world.write_resource(),
    );
}

pub fn create_player(pos: ::na::Vector3<f32>, world: &mut ::specs::World) {
    let shape = ::ncollide::shape::Cuboid3::new(::na::Vector3::from_element(0.1));
    let mut body = ::nphysics::object::RigidBody::new_dynamic(shape, 10000.0, 0.0, 0.0);
    body.set_transformation(::na::Isometry3::new(pos, ::na::zero()));

    let entity = world.create_entity().with(::component::Player).build();

    ::component::PhysicBody::add(
        entity,
        body,
        &mut world.write(),
        &mut world.write_resource(),
    );
}
