use nphysics::object::WorldObject;
use ncollide::events::ProximityEvent;
use ncollide::query::Proximity;
use specs::Join;

pub struct PhysicSystem;

impl<'a> ::specs::System<'a> for PhysicSystem {
    type SystemData = (
        ::specs::ReadStorage<'a, ::component::RocketControl>,
        ::specs::ReadStorage<'a, ::component::FlightControl>,
        ::specs::WriteStorage<'a, ::component::PhysicBody>,
        ::specs::WriteStorage<'a, ::component::Contactor>,
        ::specs::WriteStorage<'a, ::component::Proximitor>,
        ::specs::Fetch<'a, ::resource::UpdateTime>,
        ::specs::FetchMut<'a, ::resource::PhysicWorld>,
    );

    fn run(
        &mut self,
        (
            rocket_controls,
            flight_controls,
            mut bodies,
            mut contactors,
            mut proximitors,
            update_time,
            mut physic_world,
        ): Self::SystemData,
    ) {
        for (flight_control, body) in (&flight_controls, &mut bodies).join() {
            let body = body.get_mut(&mut physic_world);
            let orientation = body.position().rotation;

            let ang_vel = body.ang_vel();
            let lin_vel = body.lin_vel();
            body.set_ang_vel_internal(flight_control.ang_damping * ang_vel);
            body.set_lin_vel_internal(flight_control.lin_damping * lin_vel);

            body.clear_forces();
            body.append_ang_force(
                orientation * ::na::Vector3::y() * flight_control.y_direction
                    * flight_control.direction_force,
            );
            body.append_ang_force(
                orientation * ::na::Vector3::x() * flight_control.x_direction
                    * flight_control.direction_force,
            );
            let lin_force = flight_control.power * flight_control.power_force
                + flight_control.default_power_force;
            body.append_lin_force(orientation * ::na::Vector3::x() * lin_force);
        }

        for (rocket_control, body) in (&rocket_controls, &mut bodies).join() {
            let body = body.get_mut(&mut physic_world);

            let lin_vel = body.lin_vel();
            body.set_lin_vel_internal(rocket_control.lin_damping * lin_vel);

            body.clear_forces();
            body.append_lin_force(rocket_control.direction * rocket_control.force);
        }

        for contactor in (&mut contactors).join() {
            contactor.contacts.clear();
        }
        for proximitor in (&mut proximitors).join() {
            proximitor.intersections.clear();
        }

        let mut remaining_to_update = update_time.0;
        while remaining_to_update > ::CFG.physic_min_step_time {
            let step = remaining_to_update.min(::CFG.physic_max_step_time);
            remaining_to_update -= step;
            physic_world.step(step);

            for (co1, co2, mut contact) in physic_world.collision_world().contacts() {
                let (entity_1, entity_2) = match (&co1.data, &co2.data) {
                    (&WorldObject::RigidBody(w1), &WorldObject::RigidBody(w2)) => {
                        let e1 = physic_world.rigid_body(w1);
                        let e2 = physic_world.rigid_body(w2);
                        (
                            ::component::PhysicBody::entity(e1),
                            ::component::PhysicBody::entity(e2),
                        )
                    }
                    _ => unreachable!(),
                };

                if let Some(contactor) = contactors.get_mut(entity_1) {
                    contactor.contacts.push((entity_2, contact.clone()));
                }

                if let Some(contactor) = contactors.get_mut(entity_2) {
                    contact.flip();
                    contactor.contacts.push((entity_1, contact));
                }
            }

            for event in physic_world.collision_world().proximity_events() {
                if let &ProximityEvent {
                    co1,
                    co2,
                    new_status: Proximity::Intersecting,
                    ..
                } = event
                {
                    let co1 = physic_world
                        .collision_world()
                        .collision_object(co1)
                        .map(|c| &c.data);
                    let co2 = physic_world
                        .collision_world()
                        .collision_object(co2)
                        .map(|c| &c.data);

                    if let (Some(co1), Some(co2)) = (co1, co2) {
                        // we can't just get e1 and e2 and check for each if there is a proximitor
                        // because the rigid body of eX may be involve in a proximity even if the
                        // proximitor is associated to eX sensor
                        match (co1, co2) {
                            (&WorldObject::Sensor(w1), &WorldObject::RigidBody(w2)) => {
                                let e1 = ::component::PhysicSensor::entity(physic_world.sensor(w1));
                                let e2 = ::component::PhysicBody::entity(physic_world.rigid_body(w2));
                                if let Some(proximitor) = proximitors.get_mut(e1) {
                                    proximitor.intersections.push(e2);
                                }
                            }
                            (&WorldObject::RigidBody(w1), &WorldObject::Sensor(w2)) => {
                                let e1 = ::component::PhysicBody::entity(physic_world.rigid_body(w1));
                                let e2 = ::component::PhysicSensor::entity(physic_world.sensor(w2));
                                if let Some(proximitor) = proximitors.get_mut(e2) {
                                    proximitor.intersections.push(e1);
                                }
                            }
                            (&WorldObject::Sensor(w1), &WorldObject::Sensor(w2)) => {
                                let e1 = ::component::PhysicSensor::entity(physic_world.sensor(w1));
                                let e2 = ::component::PhysicSensor::entity(physic_world.sensor(w2));
                                println!("{:?} {:?}", e1, e2);
                                if let Some(proximitor) = proximitors.get_mut(e2) {
                                    proximitor.intersections.push(e1);
                                }
                                if let Some(proximitor) = proximitors.get_mut(e1) {
                                    proximitor.intersections.push(e2);
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            }
        }
    }
}
