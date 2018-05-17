use alga::linear::AffineTransformation;

#[repr(usize)]
pub enum Group {
    Target,
    Wall,
    Player,
    Rocket,
    Mine,
}

pub fn create_wall(pos: ::na::Vector3<f32>, _color: usize, world: &mut ::specs::World) {
    let shape = ::ncollide::shape::Cuboid3::new(::na::Vector3::from_element(0.5));
    let mut body = ::nphysics::object::RigidBody::new_static(shape, 0.0, 0.0);
    body.set_transformation(::na::Isometry3::new(pos, ::na::zero()));

    let mut group = ::nphysics::object::RigidBodyCollisionGroups::new_static();
    group.set_membership(&[Group::Wall as usize]);
    body.set_collision_groups(group);

    let entity = world.create_entity().build();

    ::component::PhysicBody::add(
        entity,
        body,
        &mut world.write(),
        &mut world.write_resource(),
    );
}

pub fn create_player(pos: ::na::Vector3<f32>, world: &::specs::World) {
    let shape = ::ncollide::shape::Ball::new(::CFG.ball_radius);
    let mut group = ::nphysics::object::RigidBodyCollisionGroups::new_dynamic();
    group.set_membership(&[Group::Player as usize]);

    let mut body = ::nphysics::object::RigidBody::new_dynamic(shape, 1.0, 0.0, 0.0);
    body.set_transformation(::na::Isometry3::new(pos, ::na::zero()));
    body.set_collision_groups(group);

    let entity = world.entities().create();
    world.write().insert(entity, ::component::Player);
    world.write().insert(entity, ::component::FlightControl {
        x_direction: 0.0,
        y_direction: 0.0,
        power: 0.0,
        ang_damping: ::CFG.flight_control_ang_damping,
        lin_damping: ::CFG.flight_control_lin_damping,
        power_force: ::CFG.flight_control_power_force,
        direction_force: ::CFG.flight_control_direction_force,
        default_power_force: ::CFG.flight_control_default_power_force,
    });

    ::component::PhysicBody::add(
        entity,
        body,
        &mut world.write(),
        &mut world.write_resource(),
    );

    world.write_resource::<::resource::PlayersEntities>()[0] = Some(entity);
}

pub fn create_tube(tube: &::tube::Tube, world: &mut ::specs::World) {
    let bodies = match tube.shape {
        ::tube::Shape::Line => {
            let half_extents = ::na::Vector3::new(::tube::RADIUS, 0.5, ::tube::RADIUS);
            let shape = ::ncollide::shape::Cuboid::new(half_extents);
            let mut body = ::nphysics::object::RigidBody::new_static(shape, 0.0, 0.0);
            body.set_transformation(tube.position);
            vec![body]
        }
        ::tube::Shape::Angle => {
            let half_extents = ::na::Vector3::new(::tube::RADIUS, 0.25, ::tube::RADIUS);
            let shape = ::ncollide::shape::Cuboid::new(half_extents);
            let mut body_0 = ::nphysics::object::RigidBody::new_static(shape, 0.0, 0.0);
            let y =
                (tube.position.rotation * ::na::Point::from_coordinates(::na::Vector3::y())).coords;
            let position = tube.position
                .append_translation(&::na::Translation::from_vector(-y * 0.25));
            body_0.set_transformation(position);

            let half_extents = ::na::Vector3::new(0.25, ::tube::RADIUS, ::tube::RADIUS);
            let shape = ::ncollide::shape::Cuboid::new(half_extents);
            let mut body_1 = ::nphysics::object::RigidBody::new_static(shape, 0.0, 0.0);
            let x =
                (tube.position.rotation * ::na::Point::from_coordinates(::na::Vector3::x())).coords;
            let position = tube.position
                .append_translation(&::na::Translation::from_vector(x * 0.25));
            body_1.set_transformation(position);

            vec![body_0, body_1]
        }
    };

    for mut body in bodies {
        let mut group = ::nphysics::object::RigidBodyCollisionGroups::new_static();
        group.set_membership(&[Group::Wall as usize]);
        body.set_collision_groups(group);

        let entity = world.create_entity().build();

        ::component::PhysicBody::add(
            entity,
            body,
            &mut world.write(),
            &mut world.write_resource(),
        );
    }
}

pub fn create_rocket(pos: ::na::Isometry3<f32>, world: &::specs::World) {
    let mut group = ::nphysics::object::RigidBodyCollisionGroups::new_dynamic();
    group.set_membership(&[Group::Rocket as usize]);
    let shape = ::ncollide::shape::Ball::new(::CFG.ball_radius);
    let mut body = ::nphysics::object::RigidBody::new_dynamic(shape, 1.0, 0.0, 0.0);
    body.set_collision_groups(group);
    body.set_transformation(pos);

    let entity = world.entities().create();
    world.write().insert(entity, ::component::PlayerKiller);
    world.write().insert(entity, ::component::Contactor::new());
    world.write().insert(entity, ::component::RocketControl);
    world.write().insert(entity, ::component::ClosestPlayer::new());

    ::component::PhysicBody::add(
        entity,
        body,
        &mut world.write(),
        &mut world.write_resource(),
    );
}

pub fn create_rocket_launcher(pos: ::na::Isometry3<f32>, world: &mut ::specs::World) {
    world.create_entity()
        .with(::component::RocketLauncher::new(pos))
        .build();
}

pub fn create_mine(pos: ::na::Vector3<f32>, world: &::specs::World) {
    let mut group = ::nphysics::object::RigidBodyCollisionGroups::new_dynamic();
    group.set_membership(&[Group::Mine as usize]);
    let shape = ::ncollide::shape::Ball::new(::CFG.ball_radius);
    let mut body = ::nphysics::object::RigidBody::new_dynamic(shape, 1.0, 0.0, 0.0);
    body.set_collision_groups(group);
    body.set_transformation(::na::Isometry3::new(pos, ::na::zero()));

    let entity = world.entities().create();
    world.write().insert(entity, ::component::PlayerKiller);
    world.write().insert(entity, ::component::Contactor::new());
    world.write().insert(entity, ::component::MineControl);
    world.write().insert(entity, ::component::ClosestPlayer::new());

    ::component::PhysicBody::add(
        entity,
        body,
        &mut world.write(),
        &mut world.write_resource(),
    );
}

pub fn create_target(pos: ::na::Vector3<f32>, world: &mut ::specs::World) {
    let mut group = ::nphysics::object::SensorCollisionGroups::new();
    group.set_membership(&[Group::Target as usize]);
    group.set_whitelist(&[Group::Target as usize]);

    let shape = ::ncollide::shape::Ball::new(::CFG.ball_radius);
    let mut sensor = ::nphysics::object::Sensor::new(shape, None);
    sensor.set_relative_position(::na::Isometry3::new(pos, ::na::zero()));
    sensor.set_collision_groups(group);

    let entity = world
        .create_entity()
        .with(::component::Target)
        .build();

    ::component::PhysicSensor::add(
        entity,
        sensor,
        &mut world.write(),
        &mut world.write_resource(),
    );
}
