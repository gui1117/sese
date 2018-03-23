use std::fs::File;

const FILENAME: &str = "cfg.ron";

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
}

impl Configuration {
    fn check(&self) {
    }
}
