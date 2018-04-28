use rand::{thread_rng, Rng};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, EnumIterator)]
pub enum TileSize {
    T1x1,
    T1x2,
    T2x1,
    T2x2,
    T2x3,
    T3x2,
    T3x3,
}

impl<'a> TileSize {
    pub fn size(&self) -> (isize, isize) {
        match *self {
            TileSize::T1x1 => (1, 1),
            TileSize::T1x2 => (1, 2),
            TileSize::T2x1 => (2, 1),
            TileSize::T2x2 => (2, 2),
            TileSize::T2x3 => (2, 3),
            TileSize::T3x2 => (3, 2),
            TileSize::T3x3 => (3, 3),
        }
    }

    pub fn width(&self) -> isize {
        self.size().0
    }

    pub fn height(&self) -> isize {
        self.size().1
    }

    fn from_size(width: isize, height: isize) -> Self {
        match (width, height) {
            (1, 1) => TileSize::T1x1,
            (2, 1) => TileSize::T2x1,
            (1, 2) => TileSize::T1x2,
            (2, 2) => TileSize::T2x2,
            (3, 2) => TileSize::T3x2,
            (2, 3) => TileSize::T2x3,
            (3, 3) => TileSize::T3x3,
            _ => panic!("invalid tile size"),
        }
    }
}

#[derive(Debug)]
pub struct Tile {
    pub position: ::na::Isometry3<f32>,
    pub size: TileSize,
    pub width: f32,
    pub height: f32,
}

/// Take a random cell in a face insert one largest tile on it. Continue to cover all faces
pub fn build_maze(maze: &::maze::Maze<::na::U3>) -> Vec<Tile> {
    #[derive(Hash, PartialEq, Eq, Clone)]
    struct Face {
        normal: ::na::Vector3<isize>,
        position: ::na::Vector3<isize>,
    }

    impl Face {
        fn shift(&self, shift: ::na::Vector3<isize>) -> Face {
            Face {
                normal: self.normal,
                position: self.position + shift,
            }
        }
    }

    fn one_largest(face: &Face, faces: &HashSet<Face>) -> Vec<Face> {
        let mut rng = ::rand::thread_rng();
        let mut tiles = all_9(face, faces);
        if !tiles.is_empty() {
            rng.shuffle(&mut tiles);
            return tiles.swap_remove(0);
        }
        let mut tiles = all_6(face, faces);
        if !tiles.is_empty() {
            rng.shuffle(&mut tiles);
            return tiles.swap_remove(0);
        }
        let mut tiles = all_4(face, faces);
        if !tiles.is_empty() {
            rng.shuffle(&mut tiles);
            return tiles.swap_remove(0);
        }
        let mut tiles = all_2(face, faces);
        if !tiles.is_empty() {
            rng.shuffle(&mut tiles);
            return tiles.swap_remove(0);
        }
        let mut tiles = all_1(face, faces);
        if !tiles.is_empty() {
            rng.shuffle(&mut tiles);
            return tiles.swap_remove(0);
        }
        unreachable!();
    }

    fn all_1(face: &Face, faces: &HashSet<Face>) -> Vec<Vec<Face>> {
        if faces.contains(&face) {
            vec![vec![face.clone()]]
        } else {
            vec![]
        }
    }

    fn all_2(face: &Face, faces: &HashSet<Face>) -> Vec<Vec<Face>> {
        let (left, down) = left_down(face.normal);
        vec![
            down_1x2(face, faces),
            down_1x2(&face.shift(down), faces),
            left_2x1(face, faces),
            left_2x1(&face.shift(left), faces),
        ].iter()
            .cloned()
            .filter_map(|tile| tile)
            .collect::<Vec<_>>()
    }

    fn all_4(face: &Face, faces: &HashSet<Face>) -> Vec<Vec<Face>> {
        let (left, down) = left_down(face.normal);
        vec![
            down_left_2x2(face, faces),
            down_left_2x2(&face.shift(left), faces),
            down_left_2x2(&face.shift(down), faces),
            down_left_2x2(&face.shift(down + left), faces),
        ].iter()
            .cloned()
            .filter_map(|tile| tile)
            .collect::<Vec<_>>()
    }

    fn all_6(face: &Face, faces: &HashSet<Face>) -> Vec<Vec<Face>> {
        let (left, down) = left_down(face.normal);
        vec![
            down_3x2(face, faces),
            down_3x2(&face.shift(down), faces),
            left_2x3(face, faces),
            left_2x3(&face.shift(left), faces),
        ].iter()
            .cloned()
            .filter_map(|tile| tile)
            .collect::<Vec<_>>()
    }

    fn all_9(face: &Face, faces: &HashSet<Face>) -> Vec<Vec<Face>> {
        let (left, down) = left_down(face.normal);
        vec![
            centered_3x3(face, faces),
            centered_3x3(&face.shift(-left), faces),
            centered_3x3(&face.shift(left), faces),
            centered_3x3(&face.shift(-down), faces),
            centered_3x3(&face.shift(down), faces),
            centered_3x3(&face.shift(-left - down), faces),
            centered_3x3(&face.shift(left - down), faces),
            centered_3x3(&face.shift(-left + down), faces),
            centered_3x3(&face.shift(left + down), faces),
        ].iter()
            .cloned()
            .filter_map(|tile| tile)
            .collect::<Vec<_>>()
    }

    fn down_1x2(face: &Face, faces: &HashSet<Face>) -> Option<Vec<Face>> {
        let (_, down) = left_down(face.normal);
        let tiles = [face.position, face.position - down]
            .iter()
            .map(|position| Face {
                normal: face.normal,
                position: *position,
            })
            .collect::<Vec<_>>();

        if tiles.iter().all(|face| faces.contains(face)) {
            Some(tiles)
        } else {
            None
        }
    }

    fn left_2x1(face: &Face, faces: &HashSet<Face>) -> Option<Vec<Face>> {
        let (left, _) = left_down(face.normal);
        let tiles = [face.position, face.position - left]
            .iter()
            .map(|position| Face {
                normal: face.normal,
                position: *position,
            })
            .collect::<Vec<_>>();

        if tiles.iter().all(|face| faces.contains(face)) {
            Some(tiles)
        } else {
            None
        }
    }

    fn down_left_2x2(face: &Face, faces: &HashSet<Face>) -> Option<Vec<Face>> {
        let (left, down) = left_down(face.normal);
        let tiles = [
            face.position,
            face.position - left,
            face.position - left - down,
            face.position - down,
        ].iter()
            .map(|position| Face {
                normal: face.normal,
                position: *position,
            })
            .collect::<Vec<_>>();

        if tiles.iter().all(|face| faces.contains(face)) {
            Some(tiles)
        } else {
            None
        }
    }

    fn left_2x3(face: &Face, faces: &HashSet<Face>) -> Option<Vec<Face>> {
        let (left, down) = left_down(face.normal);
        let tiles = [
            face.position,
            face.position + down,
            face.position - down,
            face.position - left,
            face.position - left + down,
            face.position - left - down,
        ].iter()
            .map(|position| Face {
                normal: face.normal,
                position: *position,
            })
            .collect::<Vec<_>>();

        if tiles.iter().all(|face| faces.contains(face)) {
            Some(tiles)
        } else {
            None
        }
    }

    fn down_3x2(face: &Face, faces: &HashSet<Face>) -> Option<Vec<Face>> {
        let (left, down) = left_down(face.normal);
        let tiles = [
            face.position,
            face.position + left,
            face.position - left,
            face.position + down,
            face.position + down + left,
            face.position + down - left,
        ].iter()
            .map(|position| Face {
                normal: face.normal,
                position: *position,
            })
            .collect::<Vec<_>>();

        if tiles.iter().all(|face| faces.contains(face)) {
            Some(tiles)
        } else {
            None
        }
    }

    fn centered_3x3(face: &Face, faces: &HashSet<Face>) -> Option<Vec<Face>> {
        let (left, down) = left_down(face.normal);
        let tiles = [
            face.position,
            face.position + left,
            face.position - left,
            face.position + down,
            face.position - down,
            face.position + left + down,
            face.position - left + down,
            face.position + left - down,
            face.position - left - down,
        ].iter()
            .map(|position| Face {
                normal: face.normal,
                position: *position,
            })
            .collect::<Vec<_>>();

        if tiles.iter().all(|face| faces.contains(face)) {
            Some(tiles)
        } else {
            None
        }
    }

    fn left_down(normal: ::na::Vector3<isize>) -> (::na::Vector3<isize>, ::na::Vector3<isize>) {
        if normal == ::na::Vector3::new(1, 0, 0) {
            (::na::Vector3::new(0, -1, 0), ::na::Vector3::new(0, 0, -1))
        } else if normal == ::na::Vector3::new(0, 1, 0) {
            (::na::Vector3::new(1, 0, 0), ::na::Vector3::new(0, 0, -1))
        } else if normal == ::na::Vector3::new(0, 0, 1) {
            (::na::Vector3::new(-1, 0, 0), ::na::Vector3::new(0, -1, 0))
        } else if normal == ::na::Vector3::new(-1, 0, 0) {
            let (left, down) = left_down(-normal);
            (-left, -down)
        } else if normal == ::na::Vector3::new(0, -1, 0) {
            let (left, down) = left_down(-normal);
            (-left, -down)
        } else if normal == ::na::Vector3::new(0, 0, -1) {
            let (left, down) = left_down(-normal);
            (-left, -down)
        } else {
            unreachable!();
        }
    }

    fn orientation(normal: ::na::Vector3<isize>) -> ::na::UnitQuaternion<f32> {
        if normal == ::na::Vector3::new(1, 0, 0) {
            ::na::UnitQuaternion::from_axis_angle(
                &::na::Unit::new_normalize(::na::Vector3::new(1.0, 0.0, 0.0)),
                ::std::f32::consts::FRAC_PI_2,
            )
                * ::na::UnitQuaternion::from_axis_angle(
                    &::na::Unit::new_normalize(::na::Vector3::new(0.0, 1.0, 0.0)),
                    ::std::f32::consts::FRAC_PI_2,
                )
        } else if normal == ::na::Vector3::new(0, 1, 0) {
            ::na::UnitQuaternion::from_axis_angle(
                &::na::Unit::new_normalize(::na::Vector3::new(1.0, 0.0, 0.0)),
                ::std::f32::consts::FRAC_PI_2,
            )
        } else if normal == ::na::Vector3::new(0, 0, 1) {
            ::na::one()
        } else if normal == ::na::Vector3::new(-1, 0, 0) {
            -orientation(-normal)
        } else if normal == ::na::Vector3::new(0, -1, 0) {
            -orientation(-normal)
        } else if normal == ::na::Vector3::new(0, 0, -1) {
            -orientation(-normal)
        } else {
            unreachable!();
        }
    }

    let mut faces = maze.walls
        .iter()
        .flat_map(|cell| {
            [
                ::na::Vector3::new(-1, 0, 0),
                ::na::Vector3::new(1, 0, 0),
                ::na::Vector3::new(0, -1, 0),
                ::na::Vector3::new(0, 1, 0),
                ::na::Vector3::new(0, 0, -1),
                ::na::Vector3::new(0, 0, 1),
            ].iter()
                .filter(|normal| !maze.walls.contains(&(cell + *normal)))
                .map(|normal| Face {
                    normal: *normal,
                    position: *cell,
                })
                .collect::<Vec<_>>()
        })
        .collect::<HashSet<_>>();

    let mut tiles = vec![];
    let mut face_random_list = faces.iter().cloned().collect::<Vec<_>>();
    thread_rng().shuffle(&mut face_random_list);
    for face in face_random_list {
        if faces.contains(&face) {
            let largest_tile = one_largest(&face, &faces);

            let mut x_min = ::std::isize::MAX;
            let mut x_max = ::std::isize::MIN;
            let mut y_min = ::std::isize::MAX;
            let mut y_max = ::std::isize::MIN;
            let mut z_min = ::std::isize::MAX;
            let mut z_max = ::std::isize::MIN;

            let normal = largest_tile[0].normal;
            let (left, down) = left_down(normal);

            for face in largest_tile {
                faces.remove(&face);
                x_min = x_min.min(face.position[0]);
                x_max = x_max.max(face.position[0]);
                y_min = y_min.min(face.position[1]);
                y_max = y_max.max(face.position[1]);
                z_min = z_min.min(face.position[2]);
                z_max = z_max.max(face.position[2]);
            }

            let size =
                ::na::Vector3::new(x_max - x_min + 1, y_max - y_min + 1, z_max - z_min + 1);

            let translation = ::na::Vector3::new(
                x_min as f32 + size[0] as f32 / 2.0 + normal[0] as f32 / 2.0,
                y_min as f32 + size[1] as f32 / 2.0 + normal[1] as f32 / 2.0,
                z_min as f32 + size[2] as f32 / 2.0 + normal[2] as f32 / 2.0,
            );

            let position = ::na::Isometry3::from_parts(
                ::na::Translation::from_vector(translation),
                orientation(normal),
            );

            let width = (left.component_mul(&size)).iter().sum::<isize>().abs();
            let height = (down.component_mul(&size)).iter().sum::<isize>().abs();

            tiles.push(Tile {
                position: position,
                size: TileSize::from_size(width, height),
                width: width as f32,
                height: height as f32,
            });
        }
    }

    tiles
}
