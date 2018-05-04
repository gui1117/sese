use std::fs::File;
use std::fmt;
use std::io::Write;
use std::path::PathBuf;
use app_dirs2::{app_root, AppDataType, AppInfo};
use show_message::OkOrShow;
use vulkano::pipeline::viewport::Viewport;

pub type PhysicWorld = ::nphysics::world::World<f32>;
#[derive(Deref, DerefMut)]
pub struct PlayersEntities(pub [Option<::specs::Entity>; 3]);
#[derive(Deref, DerefMut)]
pub struct PlayersControllers(pub [Option<Controller>; 3]);

impl PlayersControllers {
    pub fn convert_event(&self, event: ::winit::Event) -> Option<(usize, ::winit::ElementState, Control)> {
        match event {
            ::winit::Event::WindowEvent {
                event:
                    ::winit::WindowEvent::KeyboardInput {
                        input:
                            ::winit::KeyboardInput {
                                state,
                                virtual_keycode: Some(virtual_keycode),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                self.iter()
                    .enumerate()
                    .filter_map(|(player, c)| if let &Some(::resource::Controller::Keyboard(ref controls)) = c {
                        Some((player, controls))
                    } else {
                        None
                    })
                    .filter_map(|(player, controls)| {
                        controls.mapping.iter()
                            .find(|c| c.0 == virtual_keycode)
                            .map(|c| (player, state, c.1))
                    })
                    .next()
            }
            _ => None,
        }
    }
}

#[derive(Clone)]
pub enum Controller {
    Gamepad(usize),
    Keyboard(KeyboardController),
}

impl Controller {
    pub fn new_keyboard(mapping: Vec<(::winit::VirtualKeyCode, Control)>) -> Self {
        Controller::Keyboard(KeyboardController {
            mapping,
            pressed: vec![],
        })
    }
}

#[derive(Clone)]
pub struct KeyboardController {
    pub mapping: Vec<(::winit::VirtualKeyCode, Control)>,
    pub pressed: Vec<Control>,
}

#[derive(Clone, Copy, PartialEq, EnumIterator)]
pub enum Control {
    Up,
    Down,
    Left,
    Right,
    Boost,
    Menu,
}

impl fmt::Display for Control {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Control::Up => write!(f, "Up [↑]"),
            Control::Down => write!(f, "Down [↓]"),
            Control::Left => write!(f, "Left [←]"),
            Control::Right => write!(f, "Right [→]"),
            Control::Boost => write!(f, "Boost [SPACE]"),
            Control::Menu => write!(f, "Menu [ESCAPE]"),
        }
    }
}

#[derive(Deref, DerefMut)]
pub struct UpdateTime(pub f32);

#[derive(Default)]
pub struct Text {
    pub players: [Vec<::rusttype::PositionedGlyph<'static>>; 3],
    pub global: Vec<::rusttype::PositionedGlyph<'static>>,
}

#[derive(Deref, DerefMut)]
pub struct Font(pub ::rusttype::Font<'static>);

impl Font {
    pub fn new() -> Self {
        let font_data = include_bytes!("DejaVuSans.ttf");
        Font(::rusttype::Font::from_bytes(font_data as &[u8]).unwrap())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Mode1Player,
    Mode2Player,
    Mode3Player,
}
impl Mode {
    pub fn number_of_player(&self) -> usize {
        match *self {
            Mode::Mode1Player => 1,
            Mode::Mode2Player => 2,
            Mode::Mode3Player => 3,
        }
    }

    pub fn increase(&mut self) {
        match *self {
            Mode::Mode1Player => *self = Mode::Mode2Player,
            Mode::Mode2Player => *self = Mode::Mode3Player,
            Mode::Mode3Player => *self = Mode::Mode3Player,
        }
    }

    pub fn reduce(&mut self) {
        match *self {
            Mode::Mode1Player => *self = Mode::Mode1Player,
            Mode::Mode2Player => *self = Mode::Mode1Player,
            Mode::Mode3Player => *self = Mode::Mode2Player,
        }
    }

    pub fn viewport_for_player(&self, player: usize, dimensions: [u32; 2]) -> Viewport {
        let mut viewport = match *self {
            Mode::Mode1Player => {
                assert!(player == 0);
                Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [1.0, 1.0],
                    depth_range: 0.0..1.0,
                }
            },
            Mode::Mode2Player => {
                match player {
                    0 => Viewport {
                        origin: [0.0, 0.0],
                        dimensions: [1.0, 0.5],
                        depth_range: 0.0..1.0,
                    },
                    1 => Viewport {
                        origin: [0.0, 0.5],
                        dimensions: [1.0, 0.5],
                        depth_range: 0.0..1.0,
                    },
                    _ => unreachable!(),
                }
            },
            Mode::Mode3Player => {
                match player {
                    0 => Viewport {
                        origin: [0.0, 0.0],
                        dimensions: [0.5, 0.5],
                        depth_range: 0.0..1.0,
                    },
                    1 => Viewport {
                        origin: [0.5, 0.0],
                        dimensions: [0.5, 0.5],
                        depth_range: 0.0..1.0,
                    },
                    2 => Viewport {
                        origin: [0.0, 0.5],
                        dimensions: [1.0, 0.5],
                        depth_range: 0.0..1.0,
                    },
                    _ => unreachable!(),
                }
            },
        };
        viewport.origin[0] *= dimensions[0] as f32;
        viewport.origin[1] *= dimensions[1] as f32;
        viewport.dimensions[0] *= dimensions[0] as f32;
        viewport.dimensions[1] *= dimensions[1] as f32;
        viewport
    }
}

#[derive(Deref, DerefMut)]
pub struct Tiles(pub Vec<::tile::Tile>);

#[derive(Deref, DerefMut)]
pub struct Tubes(pub Vec<::tube::Tube>);

const APP_INFO: AppInfo = AppInfo {
    name: "SESE",
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
