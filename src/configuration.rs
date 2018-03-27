use std::fs::File;
use std::path::PathBuf;

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

    pub glyph_scale_tolerance: f32,
    pub glyph_position_tolerance: f32,
    pub glyph_width: u32,
    pub glyph_height: u32,

    pub font_file: String,
}

impl Configuration {
    fn check(&self) {
    }
}
