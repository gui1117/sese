use rand::distributions::{IndependentSample, Range};
use rand::{thread_rng, Rng};
use std::collections::HashSet;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Mul;

trait Pop {
    type Item;
    fn pop(&mut self) -> Option<Self::Item>;
}

impl<T: Eq + Hash + Clone> Pop for HashSet<T> {
    type Item = T;
    fn pop(&mut self) -> Option<Self::Item> {
        self.iter()
            .next()
            .map(|cell| cell.clone())
            .map(|cell| self.take(&cell).unwrap())
    }
}

struct Opening<D>
where
    D: ::na::Dim + ::na::DimName + Hash,
    D::Value: Mul<::typenum::UInt<::typenum::UTerm, ::typenum::B1>, Output = D::Value>
        + ::generic_array::ArrayLength<isize>,
{
    cell: ::na::VectorN<isize, D>,
    requires: Vec<::na::VectorN<isize, D>>,
    cost: isize,
}

pub struct Maze<D>
where
    D: ::na::Dim + ::na::DimName + Hash,
    D::Value: Mul<::typenum::UInt<::typenum::UTerm, ::typenum::B1>, Output = D::Value>
        + ::generic_array::ArrayLength<isize>,
{
    walls: HashSet<::na::VectorN<isize, D>>,
    size: ::na::VectorN<isize, D>,
    openings: Vec<Opening<D>>,
    neighbours: Vec<::na::VectorN<isize, D>>,
}

impl Maze<::na::U3> {
    /// take a random wall, set the larger not colored cuboid containing it to one color
    ///
    /// continue while some wall are not colored
    pub fn build_colors(&self) -> HashMap<::na::Vector3<isize>, usize> {
        let mut wall_random_list = self.walls.iter().cloned().collect::<Vec<_>>();
        let mut color = 0;
        thread_rng().shuffle(&mut wall_random_list);

        let mut colored = HashMap::new();
        for wall in wall_random_list {
            if colored.contains_key(&wall) {
                continue;
            }

            let mut expand = vec![wall];
            while let Some(wall) = expand.pop() {
                self.neighbours
                    .iter()
                    .map(|n| n + wall)
                    .filter(|n| self.walls.contains(n))
                    .filter(|n| !colored.contains_key(n))
                    .for_each(|n| expand.push(n));

                colored.insert(wall, color);
            }

            color += 1;
        }

        colored
    }
}

impl<D> Maze<D>
where
    D: ::na::Dim + ::na::DimName + Hash,
    D::Value: Mul<::typenum::UInt<::typenum::UTerm, ::typenum::B1>, Output = D::Value>
        + ::generic_array::ArrayLength<isize>,
{
    pub fn new_empty() -> Self {
        Maze {
            walls: HashSet::new(),
            size: ::na::zero(),
            openings: Self::openings(),
            neighbours: Self::neighbours(),
        }
    }

    pub fn new_rectangle(size: ::na::VectorN<isize, D>) -> Self {
        Maze {
            walls: HashSet::new(),
            size: size,
            openings: Self::openings(),
            neighbours: Self::neighbours(),
        }
    }

    /// Generate partial reverse randomized_kruskal
    /// `https://en.wikipedia.org/wiki/Maze_generation_algorithm#Randomized_Kruskal`
    pub fn new_kruskal(
        size: ::na::VectorN<isize, D>,
        percent: f64,
        bug: ::na::VectorN<isize, D>,
    ) -> Self {
        struct GridCell {
            wall: bool,
            group: usize,
        }

        for size in size.iter() {
            assert_eq!(size.wrapping_rem(2), 1);
        }

        let mut grid = HashMap::new();

        for (i, cell) in Self::iterate_area(&size).iter().enumerate() {
            grid.insert(
                cell.clone(),
                GridCell {
                    wall: false,
                    group: i,
                },
            );
        }

        let mut walls: Vec<Vec<::na::VectorN<isize, D>>> = Vec::new();
        let mut x_wall = vec![]; // 1x3x3 wall centered on 0
        let mut y_wall = vec![]; // 3x1x3 wall centered on 0
        let mut z_wall = vec![]; // 3x3x1 wall centered on 0

        match D::dim() {
            2 => {
                for i in -1..2 {
                    x_wall.push(Self::new_vec2(0, i));
                    y_wall.push(Self::new_vec2(i, 0));
                }

                for x in 1..size[0] / 2 + 1 {
                    for y in 1..size[1] / 2 + 1 {
                        if y != size[1] / 2 {
                            walls.push(
                                x_wall
                                    .iter()
                                    .map(|c| c + Self::new_vec2(x * 2 - 1 + bug[0], y * 2))
                                    .collect(),
                            );
                        }
                        if x != size[0] / 2 {
                            walls.push(
                                y_wall
                                    .iter()
                                    .map(|c| c + Self::new_vec2(x * 2, y * 2 - 1 + bug[1]))
                                    .collect(),
                            );
                        }
                    }
                }
            }
            3 => {
                for i in -1..2 {
                    for j in -1..2 {
                        x_wall.push(Self::new_vec3(0, i, j));
                        y_wall.push(Self::new_vec3(i, 0, j));
                        z_wall.push(Self::new_vec3(i, j, 0));
                    }
                }

                for x in 1..size[0] / 2 + 1 {
                    for y in 1..size[1] / 2 + 1 {
                        for z in 1..size[2] / 2 + 1 {
                            let x_end = x == size[0] / 2;
                            let y_end = y == size[1] / 2;
                            let z_end = z == size[2] / 2;
                            if !y_end && !z_end {
                                walls.push(
                                    x_wall
                                        .iter()
                                        .map(|c| {
                                            c + Self::new_vec3(x * 2 - 1 + bug[0], y * 2, z * 2)
                                        })
                                        .collect(),
                                );
                            }
                            if !x_end && !z_end {
                                walls.push(
                                    y_wall
                                        .iter()
                                        .map(|c| {
                                            c + Self::new_vec3(x * 2, y * 2 - 1 + bug[1], z * 2)
                                        })
                                        .collect(),
                                );
                            }
                            if !x_end && !y_end {
                                walls.push(
                                    z_wall
                                        .iter()
                                        .map(|c| {
                                            c + Self::new_vec3(x * 2, y * 2, z * 2 - 1 + bug[2])
                                        })
                                        .collect(),
                                );
                            }
                        }
                    }
                }
            }
            _ => unimplemented!(),
        }

        let mut rng = ::rand::thread_rng();

        let stop = ((walls.len() as f64) * (1. - percent / 100.)) as usize;

        while walls.len() > stop {
            let i = ::rand::distributions::Range::new(0, walls.len()).ind_sample(&mut rng);
            let wall = walls.swap_remove(i);

            let mut groups = HashSet::new();
            for cell in &wall {
                groups.insert(grid[cell].group);
            }
            let one_group = grid[&wall[0]].group; // a random group in the set

            if groups.len() > 2 {
                for cell in &wall {
                    grid.get_mut(cell).unwrap().wall = true
                }
                for cell in grid.values_mut() {
                    if groups.contains(&cell.group) {
                        cell.group = one_group;
                    }
                }
            }
        }

        let mut walls = HashSet::new();
        for (key, value) in grid {
            if value.wall {
                walls.insert(key);
            }
        }

        Maze {
            size,
            walls,
            neighbours: Self::neighbours(),
            openings: Self::openings(),
        }
    }

    pub fn is_cuboid(&self) -> bool {
        for &s in self.size.iter() {
            if s != self.size[0] {
                return false;
            }
        }
        true
    }

    pub fn check(&self) {
        let zero = ::na::VectorN::<isize, D>::from_iterator((0..1).cycle());
        for wall in self.walls.iter() {
            assert!(wall < &self.size);
            assert!(wall >= &zero);
        }
    }

    /// Remove the circle of the maze
    pub fn reduce(&mut self, size: isize) {
        assert!(size > 0);
        for &s in self.size.iter() {
            assert!(s >= size * 2);
        }
        let dl = size * ::na::VectorN::<isize, D>::from_iterator((1..2).cycle());
        let mut new_walls = HashSet::new();
        for wall in self.walls.iter() {
            if wall >= &dl && wall < &(self.size.clone() - dl.clone()) {
                new_walls.insert(wall - dl.clone());
            }
        }
        self.walls = new_walls;
        self.size -= dl * 2;
    }

    /// Extend the maze with empty cell
    pub fn extend(&mut self, size: isize) {
        let dl = size * ::na::VectorN::<isize, D>::from_iterator((1..2).cycle());
        let mut new_walls = HashSet::new();
        for wall in self.walls.iter() {
            new_walls.insert(wall + dl.clone());
        }
        self.walls = new_walls;
        self.size += dl * 2;
    }

    /// Create a wall that circle the maze
    pub fn circle(&mut self) {
        for cell in self.iterate_maze() {
            for i in 0..D::dim() {
                if cell[i] == 0 || cell[i] == self.size[i] - 1 {
                    self.walls.insert(cell.clone());
                }
            }
        }
    }

    /// Filter allowed entry.
    /// Return cell and its opening.
    /// The vector returned may contains less than nbr cell if it can't dig further.
    pub fn dig_cells<F>(
        &mut self,
        nbr: usize,
        filter: F,
    ) -> Vec<(::na::VectorN<isize, D>, ::na::VectorN<isize, D>)>
    where
        F: Fn(&::na::VectorN<isize, D>) -> bool,
    {
        let mut res = vec![];
        let mut rng = ::rand::thread_rng();
        let mut candidates = self.iterate_maze();
        candidates.retain(|cell| filter(cell));

        for _ in 0..nbr {
            candidates.retain(|cell| {
                self.neighbours
                    .iter()
                    .map(|n| n + cell)
                    .filter(|n| !self.walls.contains(n))
                    .count() == 1 && !self.is_on_border(cell)
            });
            if candidates.is_empty() {
                return res;
            }
            let choosen = Range::new(0, candidates.len()).ind_sample(&mut rng);
            let cell = candidates.swap_remove(choosen);
            self.walls.remove(&cell);
            let opening = self.neighbours
                .iter()
                .map(|n| n + cell.clone())
                .filter(|n| !self.walls.contains(n))
                .next()
                .unwrap();
            res.push((cell, opening));
        }
        res
    }

    /// Compute the largest zone and fill all other zone
    ///
    /// Return whereas change have been made
    pub fn fill_smallests(&mut self) -> bool {
        let mut zones = self.compute_zones(|maze, cell| !maze.walls.contains(cell));
        if zones.is_empty() {
            return false;
        }
        let (_, max_id) = zones.iter().enumerate().fold(
            (-1, None),
            |(max_len, max_id), (id, zone)| {
                let len = zone.len() as isize;
                if len >= max_len {
                    (len, Some(id))
                } else {
                    (max_len, max_id)
                }
            },
        );
        zones.remove(max_id.unwrap());

        let mut changes = false;
        zones.iter().flat_map(|zone| zone.iter()).for_each(|pos| {
            changes = true;
            self.walls.insert(pos.clone());
        });
        changes
    }

    pub fn fill_dead_rooms(&mut self) -> bool {
        let mut changes = false;
        let rooms = self.compute_dead_room_zones();
        for pos in rooms.iter().flat_map(|z| z) {
            changes = true;
            self.walls.insert(pos.clone());
        }
        changes
    }

    pub fn fill_dead_corridors(&mut self) -> bool {
        let mut changes = false;
        loop {
            let mut corridors = self.compute_corridor_zones();
            corridors.retain(|corridor| {
                corridor.iter().any(|cell| {
                    let neighbours_wall =
                        self.neighbours.iter().map(|n| n + cell).fold(0, |acc, n| {
                            if self.walls.contains(&n) {
                                acc + 1
                            } else {
                                acc
                            }
                        });
                    neighbours_wall >= self.neighbours.len() - 1
                })
            });
            if corridors.len() == 0 {
                break;
            }
            for pos in corridors.iter().flat_map(|z| z) {
                changes = true;
                self.walls.insert(pos.clone());
            }
        }
        changes
    }

    pub fn is_neighbouring_corridor(&self, cell: &::na::VectorN<isize, D>) -> bool {
        self.neighbours
            .iter()
            .map(|n| n + cell)
            .any(|n| self.is_corridor(&n))
    }

    pub fn is_neighbouring_wall(&self, cell: &::na::VectorN<isize, D>) -> bool {
        self.neighbours
            .iter()
            .map(|n| n + cell)
            .any(|n| self.walls.contains(&n))
    }

    pub fn is_corridor(&self, cell: &::na::VectorN<isize, D>) -> bool {
        !self.walls.contains(cell)
            && self.openings
                .iter()
                .filter(|opening| {
                    opening
                        .requires
                        .iter()
                        .all(|o| !self.walls.contains(&(cell.clone() + o)))
                })
                .count() <= 2
    }

    fn is_on_border(&self, v: &::na::VectorN<isize, D>) -> bool {
        let one = ::na::VectorN::<isize, D>::from_iterator((1..2).cycle());
        !(v >= &one && v + one < self.size)
    }

    fn iterate_maze(&self) -> Vec<::na::VectorN<isize, D>> {
        Self::iterate_area(&self.size)
    }

    fn iterate_area(size: &::na::VectorN<isize, D>) -> Vec<::na::VectorN<isize, D>> {
        let mut res = vec![];

        match D::dim() {
            2 => for x in 0..size[0] {
                for y in 0..size[1] {
                    res.push(Self::new_vec2(x, y));
                }
            },
            3 => for x in 0..size[0] {
                for y in 0..size[1] {
                    for z in 0..size[2] {
                        res.push(Self::new_vec3(x, y, z));
                    }
                }
            },
            _ => unimplemented!(),
        }
        res
    }

    /// Filter(openings) -> if we keep the cell
    pub fn compute_zones<F>(&self, filter: F) -> Vec<HashSet<::na::VectorN<isize, D>>>
    where
        F: Fn(&Self, &::na::VectorN<isize, D>) -> bool,
    {
        let mut unvisited = HashSet::new();
        for cell in self.iterate_maze() {
            unvisited.insert(cell);
        }

        let mut to_visit = HashSet::new();
        let mut zones = Vec::new();

        while let Some(cell) = unvisited.pop() {
            let mut zone = HashSet::new();
            to_visit.insert(cell);

            while let Some(cell) = to_visit.pop() {
                if !filter(&self, &cell) {
                    continue;
                }

                for neighbour in self.neighbours.iter().map(|n| n + cell.clone()) {
                    if unvisited.contains(&neighbour) {
                        to_visit.insert(neighbour);
                    }
                }

                unvisited.remove(&cell);
                assert!(zone.insert(cell));
            }
            if !zone.is_empty() {
                zones.push(zone);
            }
        }

        zones
    }

    /// room zones without neighbouring corridors
    pub fn compute_inner_room_zones(&self) -> Vec<HashSet<::na::VectorN<isize, D>>> {
        let mut rooms = self.compute_room_zones();
        for room in &mut rooms {
            room.retain(|cell| !self.is_neighbouring_corridor(cell));
        }
        rooms
    }

    pub fn compute_room_zones(&self) -> Vec<HashSet<::na::VectorN<isize, D>>> {
        self.compute_zones(|maze, cell| {
            !maze.walls.contains(cell)
                && maze.openings
                    .iter()
                    .filter(|opening| {
                        opening
                            .requires
                            .iter()
                            .all(|o| !self.walls.contains(&(cell.clone() + o)))
                    })
                    .count() > 2
        })
    }

    pub fn compute_dead_room_zones(&self) -> Vec<HashSet<::na::VectorN<isize, D>>> {
        let mut rooms = self.compute_room_zones();
        rooms.retain(|room| {
            let superset = room.iter().fold(HashSet::new(), |mut acc, cell| {
                self.neighbours
                    .iter()
                    .map(|n| n + cell)
                    .filter(|n| !self.walls.contains(n))
                    .for_each(|n| {
                        acc.insert(n);
                    });
                acc
            });
            superset.difference(room).count() == 1
        });
        rooms
    }

    pub fn compute_corridor_zones(&self) -> Vec<HashSet<::na::VectorN<isize, D>>> {
        self.compute_zones(|maze, cell| maze.is_corridor(cell))
    }

    /// Return all dead room with its entry corridor
    pub fn compute_dead_room_and_corridor_zones(&self) -> Vec<HashSet<::na::VectorN<isize, D>>> {
        let mut rooms = self.compute_dead_room_zones();
        let corridors = self.compute_corridor_zones();
        for room in &mut rooms {
            let opening = {
                let superset = room.iter().fold(HashSet::new(), |mut acc, cell| {
                    self.neighbours
                        .iter()
                        .map(|n| n + cell)
                        .filter(|n| !self.walls.contains(n))
                        .for_each(|n| {
                            acc.insert(n);
                        });
                    acc
                });
                superset.difference(&room).next().unwrap().clone()
            };
            for cell in corridors
                .iter()
                .find(|corridor| corridor.contains(&opening))
                .unwrap()
                .clone()
            {
                room.insert(cell);
            }
        }
        rooms
    }

    fn new_vec2(x: isize, y: isize) -> ::na::VectorN<isize, D> {
        let mut v = ::na::VectorN::<isize, D>::zeros();
        v[0] = x;
        v[1] = y;
        v
    }

    fn new_vec3(x: isize, y: isize, z: isize) -> ::na::VectorN<isize, D> {
        let mut v = ::na::VectorN::<isize, D>::zeros();
        v[0] = x;
        v[1] = y;
        v[2] = z;
        v
    }

    pub fn find_path(
        &self,
        pos: ::na::VectorN<isize, D>,
        goal: ::na::VectorN<isize, D>,
    ) -> Option<Vec<::na::VectorN<isize, D>>> {
        ::pathfinding::astar::astar(
            &pos,
            |cell| {
                let mut res = vec![];
                for opening in self.openings.iter() {
                    if opening
                        .requires
                        .iter()
                        .all(|o| !self.walls.contains(&(o + cell.clone())))
                    {
                        res.push((opening.cell.clone() + cell, opening.cost));
                    }
                }
                res
            },
            |cell| {
                let mut min = (cell[0] - goal[0]).abs();
                for i in 1..D::dim() {
                    min = min.min((cell[i] - goal[i]).abs());
                }
                min * 10
            },
            |cell| *cell == goal,
        ).map(|p| p.0)
    }

    pub fn random_free(&self) -> ::na::VectorN<isize, D> {
        assert!(self.size.iter().all(|&s| s != 0));

        let ranges: Vec<_> = self.size.iter().map(|&s| Range::new(0, s)).collect();
        let mut rng = ::rand::thread_rng();

        let mut vec =
            ::na::VectorN::<isize, D>::from_iterator(ranges.iter().map(|r| r.ind_sample(&mut rng)));
        while self.walls.contains(&vec) {
            vec = ::na::VectorN::<isize, D>::from_iterator(
                ranges.iter().map(|r| r.ind_sample(&mut rng)),
            );
        }
        vec
    }

    pub fn free_in_square(
        &self,
        center: ::na::VectorN<isize, D>,
        radius: isize,
    ) -> Vec<::na::VectorN<isize, D>> {
        let mut res = vec![];

        let clip_start = center
            .iter()
            .map(|c| (c - radius).max(0))
            .collect::<Vec<_>>();

        let clip_end = center
            .iter()
            .zip(self.size.iter())
            .map(|(c, s)| (c + radius).min(s - 1))
            .collect::<Vec<_>>();

        match D::dim() {
            2 => {
                for y in clip_start[1]..clip_end[1] + 1 {
                    for &x in [clip_start[0], clip_end[0]].iter() {
                        let vec = Self::new_vec2(x, y);
                        if !self.walls.contains(&vec) {
                            res.push(vec);
                        }
                    }
                }
                for x in clip_start[0]..clip_end[0] + 1 {
                    for &y in [clip_start[1], clip_end[1]].iter() {
                        let vec = Self::new_vec2(x, y);
                        if !self.walls.contains(&vec) {
                            res.push(vec);
                        }
                    }
                }
            }
            3 => {
                for x in clip_start[0]..clip_end[0] + 1 {
                    for y in clip_start[1]..clip_end[1] + 1 {
                        for &z in [clip_start[2], clip_end[2]].iter() {
                            let vec = Self::new_vec3(x, y, z);
                            if !self.walls.contains(&vec) {
                                res.push(vec);
                            }
                        }
                    }
                }
                for y in clip_start[1]..clip_end[1] + 1 {
                    for z in clip_start[2]..clip_end[2] + 1 {
                        for &x in [clip_start[0], clip_end[0]].iter() {
                            let vec = Self::new_vec3(x, y, z);
                            if !self.walls.contains(&vec) {
                                res.push(vec);
                            }
                        }
                    }
                }
                for x in clip_start[0]..clip_end[0] + 1 {
                    for z in clip_start[2]..clip_end[2] + 1 {
                        for &y in [clip_start[1], clip_end[1]].iter() {
                            let vec = Self::new_vec3(x, y, z);
                            if !self.walls.contains(&vec) {
                                res.push(vec);
                            }
                        }
                    }
                }
            }
            _ => unimplemented!(),
        }
        res
    }

    fn neighbours() -> Vec<::na::VectorN<isize, D>> {
        match D::dim() {
            2 => vec![
                Self::new_vec2(-1, 0),
                Self::new_vec2(1, 0),
                Self::new_vec2(0, -1),
                Self::new_vec2(0, 1),
            ],
            3 => vec![
                Self::new_vec3(-1, 0, 0),
                Self::new_vec3(1, 0, 0),
                Self::new_vec3(0, -1, 0),
                Self::new_vec3(0, 1, 0),
                Self::new_vec3(0, 0, -1),
                Self::new_vec3(0, 0, 1),
            ],
            _ => unimplemented!(),
        }
    }

    fn openings() -> Vec<Opening<D>> {
        match D::dim() {
            2 => vec![
                Opening {
                    cell: Self::new_vec2(-1, 0),
                    cost: 10,
                    requires: vec![Self::new_vec2(-1, 0)],
                },
                Opening {
                    cell: Self::new_vec2(1, 0),
                    cost: 10,
                    requires: vec![Self::new_vec2(1, 0)],
                },
                Opening {
                    cell: Self::new_vec2(0, -1),
                    cost: 10,
                    requires: vec![Self::new_vec2(0, -1)],
                },
                Opening {
                    cell: Self::new_vec2(0, 1),
                    cost: 10,
                    requires: vec![Self::new_vec2(0, 1)],
                },
                Opening {
                    cell: Self::new_vec2(-1, -1),
                    cost: 15,
                    requires: vec![
                        Self::new_vec2(-1, 0),
                        Self::new_vec2(0, -1),
                        Self::new_vec2(-1, -1),
                    ],
                },
                Opening {
                    cell: Self::new_vec2(-1, 1),
                    cost: 15,
                    requires: vec![
                        Self::new_vec2(-1, 0),
                        Self::new_vec2(0, 1),
                        Self::new_vec2(-1, 1),
                    ],
                },
                Opening {
                    cell: Self::new_vec2(1, -1),
                    cost: 15,
                    requires: vec![
                        Self::new_vec2(1, 0),
                        Self::new_vec2(0, -1),
                        Self::new_vec2(1, -1),
                    ],
                },
                Opening {
                    cell: Self::new_vec2(1, 1),
                    cost: 15,
                    requires: vec![
                        Self::new_vec2(1, 0),
                        Self::new_vec2(0, 1),
                        Self::new_vec2(1, 1),
                    ],
                },
            ],
            3 => vec![
                Opening {
                    cell: Self::new_vec3(-1, 0, 0),
                    cost: 10,
                    requires: vec![Self::new_vec3(-1, 0, 0)],
                },
                Opening {
                    cell: Self::new_vec3(1, 0, 0),
                    cost: 10,
                    requires: vec![Self::new_vec3(1, 0, 0)],
                },
                Opening {
                    cell: Self::new_vec3(0, -1, 0),
                    cost: 10,
                    requires: vec![Self::new_vec3(0, -1, 0)],
                },
                Opening {
                    cell: Self::new_vec3(0, 1, 0),
                    cost: 10,
                    requires: vec![Self::new_vec3(0, 1, 0)],
                },
                Opening {
                    cell: Self::new_vec3(0, 0, -1),
                    cost: 10,
                    requires: vec![Self::new_vec3(0, 0, -1)],
                },
                Opening {
                    cell: Self::new_vec3(0, 0, 1),
                    cost: 10,
                    requires: vec![Self::new_vec3(0, 0, 1)],
                },
                Opening {
                    cell: Self::new_vec3(-1, -1, 0),
                    cost: 15,
                    requires: vec![
                        Self::new_vec3(-1, 0, 0),
                        Self::new_vec3(0, -1, 0),
                        Self::new_vec3(-1, -1, 0),
                    ],
                },
                Opening {
                    cell: Self::new_vec3(-1, 1, 0),
                    cost: 15,
                    requires: vec![
                        Self::new_vec3(-1, 0, 0),
                        Self::new_vec3(0, 1, 0),
                        Self::new_vec3(-1, 1, 0),
                    ],
                },
                Opening {
                    cell: Self::new_vec3(1, -1, 0),
                    cost: 15,
                    requires: vec![
                        Self::new_vec3(1, 0, 0),
                        Self::new_vec3(0, -1, 0),
                        Self::new_vec3(1, -1, 0),
                    ],
                },
                Opening {
                    cell: Self::new_vec3(1, 1, 0),
                    cost: 15,
                    requires: vec![
                        Self::new_vec3(1, 0, 0),
                        Self::new_vec3(0, 1, 0),
                        Self::new_vec3(1, 1, 0),
                    ],
                },
                Opening {
                    cell: Self::new_vec3(0, -1, -1),
                    cost: 15,
                    requires: vec![
                        Self::new_vec3(0, -1, 0),
                        Self::new_vec3(0, 0, -1),
                        Self::new_vec3(0, -1, -1),
                    ],
                },
                Opening {
                    cell: Self::new_vec3(0, -1, 1),
                    cost: 15,
                    requires: vec![
                        Self::new_vec3(0, -1, 0),
                        Self::new_vec3(0, 0, 1),
                        Self::new_vec3(0, -1, 1),
                    ],
                },
                Opening {
                    cell: Self::new_vec3(0, 1, -1),
                    cost: 15,
                    requires: vec![
                        Self::new_vec3(0, 1, 0),
                        Self::new_vec3(0, 0, -1),
                        Self::new_vec3(0, 1, -1),
                    ],
                },
                Opening {
                    cell: Self::new_vec3(0, 1, 1),
                    cost: 15,
                    requires: vec![
                        Self::new_vec3(0, 1, 0),
                        Self::new_vec3(0, 0, 1),
                        Self::new_vec3(0, 1, 1),
                    ],
                },
                Opening {
                    cell: Self::new_vec3(-1, 0, -1),
                    cost: 15,
                    requires: vec![
                        Self::new_vec3(-1, 0, 0),
                        Self::new_vec3(0, 0, -1),
                        Self::new_vec3(-1, 0, -1),
                    ],
                },
                Opening {
                    cell: Self::new_vec3(-1, 0, 1),
                    cost: 15,
                    requires: vec![
                        Self::new_vec3(-1, 0, 0),
                        Self::new_vec3(0, 0, 1),
                        Self::new_vec3(-1, 0, 1),
                    ],
                },
                Opening {
                    cell: Self::new_vec3(1, 0, -1),
                    cost: 15,
                    requires: vec![
                        Self::new_vec3(1, 0, 0),
                        Self::new_vec3(0, 0, -1),
                        Self::new_vec3(1, 0, -1),
                    ],
                },
                Opening {
                    cell: Self::new_vec3(1, 0, 1),
                    cost: 15,
                    requires: vec![
                        Self::new_vec3(1, 0, 0),
                        Self::new_vec3(0, 0, 1),
                        Self::new_vec3(1, 0, 1),
                    ],
                },
                Opening {
                    cell: Self::new_vec3(-1, -1, -1),
                    cost: 17,
                    requires: vec![
                        Self::new_vec3(-1, 0, 0),
                        Self::new_vec3(0, -1, 0),
                        Self::new_vec3(-1, -1, 0),
                        Self::new_vec3(-1, 0, -1),
                        Self::new_vec3(0, -1, -1),
                        Self::new_vec3(-1, -1, -1),
                        Self::new_vec3(-1, -1, -1),
                    ],
                },
                Opening {
                    cell: Self::new_vec3(1, 1, 1),
                    cost: 17,
                    requires: vec![
                        Self::new_vec3(1, 0, 0),
                        Self::new_vec3(0, 1, 0),
                        Self::new_vec3(1, 1, 0),
                        Self::new_vec3(1, 0, 1),
                        Self::new_vec3(0, 1, 1),
                        Self::new_vec3(1, 1, 1),
                        Self::new_vec3(1, 1, 1),
                    ],
                },
                Opening {
                    cell: Self::new_vec3(-1, -1, 1),
                    cost: 17,
                    requires: vec![
                        Self::new_vec3(-1, 0, 0),
                        Self::new_vec3(0, -1, 0),
                        Self::new_vec3(-1, -1, 0),
                        Self::new_vec3(-1, 0, 1),
                        Self::new_vec3(0, -1, 1),
                        Self::new_vec3(-1, -1, 1),
                        Self::new_vec3(-1, -1, 1),
                    ],
                },
                Opening {
                    cell: Self::new_vec3(-1, 1, -1),
                    cost: 17,
                    requires: vec![
                        Self::new_vec3(-1, 0, 0),
                        Self::new_vec3(0, 1, 0),
                        Self::new_vec3(-1, 1, 0),
                        Self::new_vec3(-1, 0, -1),
                        Self::new_vec3(0, 1, -1),
                        Self::new_vec3(-1, 1, -1),
                        Self::new_vec3(-1, 1, -1),
                    ],
                },
                Opening {
                    cell: Self::new_vec3(1, -1, -1),
                    cost: 17,
                    requires: vec![
                        Self::new_vec3(1, 0, 0),
                        Self::new_vec3(0, -1, 0),
                        Self::new_vec3(1, -1, 0),
                        Self::new_vec3(1, 0, -1),
                        Self::new_vec3(0, -1, -1),
                        Self::new_vec3(1, -1, -1),
                        Self::new_vec3(1, -1, -1),
                    ],
                },
                Opening {
                    cell: Self::new_vec3(-1, 1, 1),
                    cost: 17,
                    requires: vec![
                        Self::new_vec3(-1, 0, 0),
                        Self::new_vec3(0, 1, 0),
                        Self::new_vec3(-1, 1, 0),
                        Self::new_vec3(-1, 0, 1),
                        Self::new_vec3(0, 1, 1),
                        Self::new_vec3(-1, 1, 1),
                        Self::new_vec3(-1, 1, 1),
                    ],
                },
                Opening {
                    cell: Self::new_vec3(1, 1, -1),
                    cost: 17,
                    requires: vec![
                        Self::new_vec3(1, 0, 0),
                        Self::new_vec3(0, 1, 0),
                        Self::new_vec3(1, 1, 0),
                        Self::new_vec3(1, 0, -1),
                        Self::new_vec3(0, 1, -1),
                        Self::new_vec3(1, 1, -1),
                        Self::new_vec3(1, 1, -1),
                    ],
                },
                Opening {
                    cell: Self::new_vec3(1, -1, 1),
                    cost: 17,
                    requires: vec![
                        Self::new_vec3(1, 0, 0),
                        Self::new_vec3(0, -1, 0),
                        Self::new_vec3(1, -1, 0),
                        Self::new_vec3(1, 0, 1),
                        Self::new_vec3(0, -1, 1),
                        Self::new_vec3(1, -1, 1),
                        Self::new_vec3(1, -1, 1),
                    ],
                },
            ],
            _ => unimplemented!(),
        }
    }
}

impl ::std::fmt::Display for Maze<::na::U2> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        write!(f, "\n")?;
        for y in 0..self.size[1] {
            for x in 0..self.size[0] {
                if self.walls.contains(&::na::Vector2::new(x, y)) {
                    write!(f, "#")?;
                } else {
                    write!(f, " ")?;
                }
            }
            write!(f, "\n")?;
        }
        write!(f, "\n")
    }
}

// pub fn find_path(
//     &self,
//     pos: ::na::Vector3<f32>,
//     goal: ::na::Vector3<f32>,
// ) -> Option<Vec<::na::Vector3<f32>>> {
//     self.inner_find_path(self.to_inner(&pos), self.to_inner(&goal))
//         .map(|path| path.iter().map(|cell| self.to_world(cell)).collect())
// }

// #[inline]
// fn to_inner(&self, coords: &::na::Vector3<f32>) -> ::na::VectorN<isize, D> {
//     ::na::VectorN::<isize, D>::from_iterator(coords.iter().map(|&c| (c / self.scale) as isize))
// }

// #[inline]
// fn to_world(&self, coords: &::na::VectorN<isize, D>) -> ::na::Vector3<f32> {
//     let mut outer = ::na::Vector3::new(self.scale, self.scale, self.scale) * 0.5;
//     for i in 0..D::dim() {
//         outer[i] += (coords[i] as f32) * self.scale;
//     }
//     outer
// }
