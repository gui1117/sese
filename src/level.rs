pub struct LevelBuilder {
    pub half_size: usize,
    pub x_shift: bool,
    pub y_shift: bool,
    pub z_shift: bool,
    pub percent: f64,
    pub columns: usize,
    pub unit: f32,
}

impl LevelBuilder {
    pub fn build(&self, world: &mut ::specs::World) {
        // Build maze
        let mut maze = {
            let size = ::na::Vector3::new(
                (self.half_size * 2 + 1) as isize,
                (self.half_size * 2 + 1) as isize,
                (self.half_size * 2 + 1) as isize,
            );
            let bug = ::na::Vector3::new(
                if self.x_shift { 1 } else { 0 },
                if self.y_shift { 1 } else { 0 },
                if self.z_shift { 1 } else { 0 },
            );

            let mut maze = ::maze::Maze::new_kruskal(size, self.percent, bug);
            maze.reduce(1);
            maze.circle();
            maze.fill_smallests();
            while maze.fill_dead_corridors() {}
            maze.reduce(1);
            maze
        };

        let colors = maze.build_colors();
        for (wall, color) in colors {
            ::entity::create_wall(::util::to_world(&wall, self.unit), color, world);
        }

        let mut tiles = ::tile::build_maze(&maze);
        for tile in &mut tiles {
            tile.position.translation.vector *= self.unit;
            tile.width *= self.unit;
            tile.height *= self.unit;
        }
        world.add_resource(::resource::Tiles(tiles));

        // Build tubes
        let mut tubes = ::tube::build_tubes(self.columns, &mut maze);
        for tube in &mut tubes {
            tube.position.translation.vector *= self.unit;
            ::entity::create_tube(tube, world);
        }
        world.add_resource(::resource::Tubes(tubes));

        for _ in 0..10 {
            let pos = maze.random_free();
            maze.walls.insert(pos);
            ::entity::create_target(::util::to_world(&pos, self.unit), world);
        }

        // Build monsters
        let player_distance = 3;
        let player_pos = ::na::Vector3::new(
            -player_distance,
            (self.half_size + 1) as isize,
            (self.half_size + 1) as isize,
        );
        ::entity::create_player(::util::to_world(&player_pos, self.unit), world);
    }
}
