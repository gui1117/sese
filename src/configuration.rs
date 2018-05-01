use std::fs::File;

const FILENAME: &str = "configuration.ron";

lazy_static! {
    pub static ref CFG: Configuration = {
        let file = File::open(FILENAME).unwrap();
        let c: Configuration = ::ron::de::from_reader(file).unwrap();
        c.check();
        c
    };
}

#[derive(Serialize, Deserialize)]
pub struct Configuration {
    pub fps: usize,

    pub unlocal_texture_size: u32,
    pub unlocal_texture_layers: u32,

    pub physic_min_step_time: f32,
    pub physic_max_step_time: f32,

    pub flight_control_ang_damping: f32,
    pub flight_control_lin_damping: f32,
    pub flight_control_power_force: f32,
    pub flight_control_direction_force: f32,
    pub flight_control_default_power_force: f32,

    pub rocket_control_lin_damping: f32,
    pub rocket_control_force: f32,

    pub ball_radius: f32,
    pub rocket_launcher_timer: f32,

    pub mine_control_max_force: f32,
    pub mine_control_coef_force: f32,
}

impl Configuration {
    fn check(&self) {}
}
