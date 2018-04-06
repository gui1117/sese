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

    let entity = world
        .create_entity()
        .with(::component::Player)
        .with(::component::FlightControl {
            x_direction: 0.0,
            y_direction: 0.0,
            power: 0.0,
            ang_damping: ::CFG.flight_control_ang_damping,
            lin_damping: ::CFG.flight_control_lin_damping,
            power_force: ::CFG.flight_control_power_force,
            direction_force: ::CFG.flight_control_direction_force,
            default_power_force: ::CFG.flight_control_default_power_force,
        })
        .build();

    ::component::PhysicBody::add(
        entity,
        body,
        &mut world.write(),
        &mut world.write_resource(),
    );

    world.write_resource::<::resource::PlayersEntities>()[0] = Some(entity);
}

pub fn create_column(pos: ::na::Isometry3<f32>, maze_size: f32, world: &mut ::specs::World) {
    let shape = ::ncollide::shape::Cylinder::new(
        maze_size * 2_f32.sqrt() * ::CFG.column_size_factor / 2.0,
        ::CFG.column_radius,
    );
    let mut body = ::nphysics::object::RigidBody::new_static(shape, 0.0, 0.0);
    body.set_transformation(pos);

    let entity = world.create_entity().build();

    ::component::PhysicBody::add(
        entity,
        body,
        &mut world.write(),
        &mut world.write_resource(),
    );
}
