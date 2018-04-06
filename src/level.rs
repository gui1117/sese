pub struct LevelBuilder {
    pub half_size: usize,
    pub x_shift: bool,
    pub y_shift: bool,
    pub z_shift: bool,
    pub percent: f64,
}

impl LevelBuilder {
    pub fn build(&self, world: &mut ::specs::World) {
        // let maze = {
        //     let size = ::na::Vector3::new(
        //         (self.half_size*2+1) as isize,
        //         (self.half_size*2+1) as isize,
        //         (self.half_size*2+1) as isize,
        //     );
        //     let bug = ::na::Vector3::new(
        //         if self.x_shift { 1 } else { 0 },
        //         if self.y_shift { 1 } else { 0 },
        //         if self.z_shift { 1 } else { 0 },
        //     );

        //     let mut maze = ::maze::Maze::new_kruskal(size, self.percent, bug);
        //     maze.reduce(1);
        //     maze.circle();
        //     maze.fill_smallests();
        //     while maze.fill_dead_corridors() {}
        //     maze
        // };

        // let colors = maze.build_colors();
        // for (wall, color) in colors {
        //     ::entity::create_wall(::util::to_world(&wall, 1.0), color, world);
        // }

        // let player_distance = 3;
        // let player_pos = ::na::Vector3::new(
        //     - player_distance,
        //     (self.half_size+1) as isize,
        //     (self.half_size+1) as isize,
        // );
        // ::entity::create_player(::util::to_world(&player_pos, 1.0), world);

        ::entity::create_wall(::util::to_world(&::na::zero(), 1.0), 0, world);

        let player_distance = 3;
        let player_pos = ::na::Vector3::new(-player_distance, 0, 0);
        ::entity::create_player(::util::to_world(&player_pos, 1.0), world);
    }
}
