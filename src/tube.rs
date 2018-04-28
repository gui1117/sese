use rand::distributions::{IndependentSample, Range};
use rand::{thread_rng, Rng};
use alga::linear::AffineTransformation;
use std::collections::HashSet;
use itertools::Itertools;
use std::f32::consts::{FRAC_PI_2, PI};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, EnumIterator)]
pub enum Shape {
    /// Line along y axis
    Line,
    /// Angle from y axis to x axis
    Angle,
}

impl Shape {
    pub fn obj(&self) -> String {
        match *self {
            Shape::Line => include_str!("line.obj").to_string(),
            Shape::Angle => include_str!("angle.obj").to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Tube {
    pub position: ::na::Isometry3<f32>,
    pub shape: Shape,
}

pub fn generate_paths(extra_paths: usize, maze: &::maze::Maze<::na::U3>) -> Vec<Vec<::na::Vector3<isize>>> {
    let mut maze = maze.clone();
    maze.extend(2);
    maze.circle();

    let mut wall_parts = maze.compute_zones(|maze, cell| {
        maze.walls.contains(cell)
    });
    wall_parts.retain(|part| !part.iter().any(|&cell| cell == ::na::zero()));
    thread_rng().shuffle(&mut wall_parts);

    let mut wall_parts_neighbours = wall_parts.iter()
        .map(|walls| {
            let mut neighbours = walls.iter()
                // tuple with neighbour and origin
                .flat_map(|wall| maze.neighbours.iter().map(|n| (n+wall, wall)).collect::<Vec<_>>())
                .collect::<HashSet<_>>();
            let mut neighbours = neighbours.drain().collect::<Vec<_>>();
            thread_rng().shuffle(&mut neighbours);
            neighbours
        })
        .collect::<Vec<_>>();

    let parts = wall_parts_neighbours.len();
    let mut paths = vec![];
    for i in (0..parts).cycle().take(parts+extra_paths) {
        let path = {
            let ref start_part = wall_parts_neighbours[i];
            let ref end_part = wall_parts_neighbours[(i+1) % parts];

            if start_part.is_empty() || end_part.is_empty() {
                continue
            }

            let ref start = start_part[0];
            let ref end = end_part[0];

            if let Some(mut path) = maze.find_path_direct(start.0, end.0) {
                path.insert(0, *start.1);
                path.push(*end.1);
                path
            } else {
                continue
            }
        };

        for &cell in &path {
            maze.walls.insert(cell);
        }
        for part in &mut wall_parts_neighbours {
            part.retain(|cell| !maze.walls.contains(&cell.0))
        }

        paths.push(path);
    }

    // shift 2 because maze has been extended
    for path in &mut paths {
        for cell in path {
            *cell -= ::na::Vector3::new(2, 2, 2);
        }
    }

    paths
}

pub fn build_tubes(extra_tubes: usize, maze: &::maze::Maze<::na::U3>) -> Vec<Tube> {
    let paths = generate_paths(extra_tubes, maze);
    let mut tubes = vec![];
    for (start, tube, end) in paths.iter().flat_map(|path| path.iter().tuple_windows()) {
        let v = end - start;
        let translation = ::na::Translation::from_vector(::util::to_world(tube, 1.0));

        if v.iter().any(|&coord| coord.abs() == 2) {
            // Line
            let rotation = ::na::UnitQuaternion::rotation_between(
                &::na::Vector3::y(),
                &::na::Vector3::new(v[0] as f32, v[1] as f32, v[2] as f32),
            ).unwrap_or(::na::one());

            tubes.push(Tube {
                position: ::na::Isometry3::from_parts(translation, rotation),
                shape: Shape::Line,
            });
        } else {
            // Angle
            let yv = tube - start;
            let first_rotation = if yv[0].abs() == 1 {
                ::na::UnitQuaternion::from_axis_angle(&::na::Vector3::z_axis(), - yv[0].signum() as f32 * FRAC_PI_2)
            } else if yv[1] == 1 {
                ::na::one()
            } else if yv[1] == -1 {
                ::na::UnitQuaternion::from_axis_angle(&::na::Vector3::z_axis(), PI)
            } else if yv[2].abs() == 1 {
                ::na::UnitQuaternion::from_axis_angle(&::na::Vector3::x_axis(), yv[2].signum() as f32 * FRAC_PI_2)
            } else {
                unreachable!();
            };

            let xv = end - tube;
            let xv_float = ::na::Vector3::from_iterator(xv.iter().map(|&c| c as f32));
            // TODO: if the inverse cost too much to compute we can cache it as there is only 5 rotation
            let xv_float_trans = (first_rotation.inverse() * ::na::Point::from_coordinates(xv_float)).coords;
            let xv_trans = ::na::Vector3::from_iterator(xv_float_trans.iter().map(|c| c.round() as isize));

            let second_rotation = if xv_trans[0] == 1 {
                ::na::one()
            } else if xv_trans[0] == -1 {
                ::na::UnitQuaternion::from_axis_angle(&::na::Vector3::y_axis(), PI)
            } else if xv_trans[2].abs() == 1 {
                ::na::UnitQuaternion::from_axis_angle(&::na::Vector3::y_axis(), - xv_trans[2].signum() as f32 * FRAC_PI_2)
            } else {
                unreachable!();
            };

            let rotation = first_rotation * second_rotation;

            tubes.push(Tube {
                position: ::na::Isometry3::from_parts(translation, rotation),
                shape: Shape::Angle,
            });
        }
    }
    tubes
}
