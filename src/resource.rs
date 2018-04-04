use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use app_dirs2::{app_root, AppDataType, AppInfo};
use show_message::OkOrShow;

pub type PhysicWorld = ::nphysics::world::World<f32>;
pub type PlayersGamepads = [Option<usize>; 3];
pub type PlayersEntities = [Option<::specs::Entity>; 3];

#[derive(Deref, DerefMut)]
pub struct UpdateTime(pub f32);

const APP_INFO: AppInfo = AppInfo {
    name: "HyperZen Training",
    author: "thiolliere",
};
const FILENAME: &str = "save.ron";

lazy_static! {
    static ref SAVE_PATH: PathBuf = {
        let mut path = app_root(AppDataType::UserConfig, &APP_INFO).unwrap();
        path.push(FILENAME);
        path
    };
}

#[derive(Deserialize, Serialize)]
pub struct Save {
    vulkan_device_uuid: Option<[u8; 16]>,
    fullscreen: bool,
}

impl Save {
    pub fn new() -> Self {
        File::open(SAVE_PATH.as_path())
            .ok()
            .and_then(|file| ::ron::de::from_reader(file).ok())
            .unwrap_or(Save {
                fullscreen: true,
                vulkan_device_uuid: None,
            })
    }

    pub fn vulkan_device_uuid(&self) -> &Option<[u8; 16]> {
        &self.vulkan_device_uuid
    }

    /// Return if changed
    pub fn set_vulkan_device_uuid_lazy(&mut self, uuid: &[u8; 16]) -> bool {
        if self.vulkan_device_uuid
            .map(|saved_uuid| *uuid != saved_uuid)
            .unwrap_or(true)
        {
            self.vulkan_device_uuid = Some(uuid.clone());
            self.save();
            true
        } else {
            false
        }
    }

    pub fn save(&self) {
        let string = ::ron::ser::to_string(&self).unwrap();
        let mut file = File::create(SAVE_PATH.as_path()).ok_or_show(|e| {
            format!(
                "Failed to create save file at {}: {}",
                SAVE_PATH.display(),
                e
            )
        });
        file.write_all(string.as_bytes()).ok_or_show(|e| {
            format!(
                "Failed to write to save file {}: {}",
                SAVE_PATH.display(),
                e
            )
        });
    }
}
