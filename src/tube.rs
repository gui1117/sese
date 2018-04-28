use rand::distributions::{IndependentSample, Range};
use alga::linear::AffineTransformation;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, EnumIterator)]
pub enum TubeSize {
    T1,
    T2,
    T3,
}

impl TubeSize {
    pub fn size(&self) -> isize {
        match *self {
            TubeSize::T1 => 1,
            TubeSize::T2 => 2,
            TubeSize::T3 => 3,
        }
    }
    pub fn from_size(size: isize) -> Self {
        match size {
            1 => TubeSize::T1,
            2 => TubeSize::T2,
            3 => TubeSize::T3,
            _ => panic!("invalid tube size"),
        }
    }
}

#[derive(Debug)]
pub struct Tube {
    pub position: ::na::Isometry3<f32>,
    pub tube_size: TubeSize,
    pub size: f32,
}

pub fn build_column(position: ::na::Isometry3<f32>, size: isize) -> Vec<Tube> {
    let mut rng = ::rand::thread_rng();
    let mut tubes = vec![];
    let mut remaining_size = size;
    while remaining_size != 0 {
        let tube_size = Range::new(1, 3.min(remaining_size) + 1).ind_sample(&mut rng);
        let dl = (size - remaining_size) as f32 - size as f32 / 2.0 + tube_size as f32 / 2.0;
        let direction = position.rotation * ::na::Vector3::y();
        let translation = ::na::Translation::from_vector(direction*dl);
        let tube_position = position.append_translation(&translation);

        tubes.push(Tube {
            position: tube_position,
            tube_size: TubeSize::from_size(tube_size),
            size: tube_size as f32,
        });
        remaining_size -= tube_size;
    }
    tubes
}
