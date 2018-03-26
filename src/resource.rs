pub type PhysicWorld = ::nphysics::world::World<f32>;

#[derive(Deref, DerefMut)]
pub struct UpdateTime(pub f32);
pub use conrod::render::OwnedPrimitives;
