use retained_storage::RetainedStorage;
use std::any::Any;

pub struct FlightControl {
    pub x_direction: f32,
    pub y_direction: f32,
    pub power: f32,
    pub ang_damping: f32,
    pub lin_damping: f32,
    pub power_force: f32,
    pub default_power_force: f32,
    pub direction_force: f32,
}
impl ::specs::Component for FlightControl {
    type Storage = ::specs::VecStorage<Self>;
}

#[derive(Default)]
pub struct Player;
impl ::specs::Component for Player {
    type Storage = ::specs::NullStorage<Self>;
}

// Rigid body handle and whereas it has been deleted
#[derive(Clone)]
pub struct PhysicBody {
    handle: usize,
}

impl ::specs::Component for PhysicBody {
    type Storage = RetainedStorage<Self, ::specs::VecStorage<Self>>;
}

impl PhysicBody {
    pub fn handle(&self) -> usize {
        self.handle
    }

    pub fn entity(body: &::nphysics::object::RigidBody<f32>) -> ::specs::Entity {
        let entity = body.user_data().unwrap();
        let entity = unsafe { ::std::mem::transmute::<&Box<_>, &Box<Any>>(entity) };
        entity.downcast_ref::<::specs::Entity>().unwrap().clone()
    }

    pub fn add<'a>(
        entity: ::specs::Entity,
        mut body: ::nphysics::object::RigidBody<f32>,
        bodies: &mut ::specs::WriteStorage<'a, ::component::PhysicBody>,
        physic_world: &mut ::specs::FetchMut<'a, ::resource::PhysicWorld>,
    ) {
        body.set_user_data(Some(Box::new(entity)));
        let bodyhandle = physic_world.add_rigid_body(body);
        bodies.insert(entity, PhysicBody { handle: bodyhandle });
    }

    #[inline]
    pub fn get<'a>(
        &'a self,
        physic_world: &'a ::resource::PhysicWorld,
    ) -> &'a ::nphysics::object::RigidBody<f32> {
        physic_world.rigid_body(self.handle)
    }

    #[inline]
    pub fn get_mut<'a>(
        &'a mut self,
        physic_world: &'a mut ::resource::PhysicWorld,
    ) -> &'a mut ::nphysics::object::RigidBody<f32> {
        physic_world.mut_rigid_body(self.handle)
    }
}

// Sensor handle and whereas it has been deleted
#[derive(Clone)]
pub struct PhysicSensor {
    handle: usize,
}

impl ::specs::Component for PhysicSensor {
    type Storage = RetainedStorage<Self, ::specs::VecStorage<Self>>;
}

#[allow(unused)]
impl PhysicSensor {
    pub fn entity(body: &::nphysics::object::Sensor<f32>) -> ::specs::Entity {
        let entity = body.user_data().unwrap();
        let entity = unsafe { ::std::mem::transmute::<&Box<_>, &Box<Any>>(entity) };
        entity.downcast_ref::<::specs::Entity>().unwrap().clone()
    }

    pub fn add<'a>(
        entity: ::specs::Entity,
        mut sensor: ::nphysics::object::Sensor<f32>,
        sensors: &mut ::specs::WriteStorage<'a, ::component::PhysicSensor>,
        physic_world: &mut ::specs::FetchMut<'a, ::resource::PhysicWorld>,
    ) {
        sensor.set_user_data(Some(Box::new(entity)));
        let sensorhandle = physic_world.add_sensor(sensor);
        sensors.insert(
            entity,
            PhysicSensor {
                handle: sensorhandle,
            },
        );
    }

    #[inline]
    pub fn get<'a>(
        &'a self,
        physic_world: &'a ::resource::PhysicWorld,
    ) -> &'a ::nphysics::object::Sensor<f32> {
        physic_world.sensor(self.handle)
    }

    #[inline]
    pub fn get_mut<'a>(
        &'a mut self,
        physic_world: &'a mut ::resource::PhysicWorld,
    ) -> &'a mut ::nphysics::object::Sensor<f32> {
        physic_world.mut_sensor(self.handle)
    }
}

pub type Contact = ::ncollide::query::Contact<::na::Point3<f32>>;

pub struct Contactor {
    pub contacts: Vec<(::specs::Entity, Contact)>,
}

impl ::specs::Component for Contactor {
    type Storage = ::specs::VecStorage<Self>;
}

impl Contactor {
    pub fn new() -> Self {
        Contactor { contacts: vec![] }
    }
}

pub struct Proximitor {
    pub intersections: Vec<::specs::Entity>,
}

impl ::specs::Component for Proximitor {
    type Storage = ::specs::VecStorage<Self>;
}

impl Proximitor {
    pub fn new() -> Self {
        Proximitor {
            intersections: vec![],
        }
    }
}
