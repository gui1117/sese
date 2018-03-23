extern crate three;
extern crate rand;
extern crate typenum;
extern crate pathfinding;
extern crate generic_array;
#[macro_use]
extern crate lazy_static;
extern crate ron;
extern crate nalgebra as na;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate conrod;

pub mod maze;
pub mod configuration;

pub use configuration::CFG;

fn main() {
    println!("Hello, world!");
}
