use specs::World;
use gilrs::{Axis, Button, EventType};
use world_action::WorldAction;

pub trait GameState {
    fn update_draw_ui(self: Box<Self>, world: &mut World) -> Box<GameState>;
    fn winit_event(self: Box<Self>, event: ::winit::Event, world: &mut World) -> Box<GameState>;
    fn gilrs_event(
        self: Box<Self>,
        id: usize,
        event: ::gilrs::EventType,
        world: &mut World,
    ) -> Box<GameState>;
    fn gilrs_gamepad_state(
        self: Box<Self>,
        id: usize,
        gamepad: &::gilrs::Gamepad,
        world: &mut World,
    ) -> Box<GameState>;
    fn quit(&self) -> bool {
        false
    }
    fn paused(&self, world: &World) -> bool;
}

pub struct ValidateBuildedController {
    controller: Vec<::winit::VirtualKeyCode>,
    menu: ::menu::Menu<bool>,
    stacked_state: Box<GameState>,
}

impl ValidateBuildedController {
    fn new(controller: Vec<::winit::VirtualKeyCode>, stacked_state: Box<GameState>) -> Self {
        let menu = ::menu::MenuBuilder::new()
            .add_middle("OK".to_string(), true)
            .add_middle("Cancel".to_string(), false)
            .build();

        ValidateBuildedController {
            controller,
            menu,
            stacked_state,
        }
    }

    fn process_action(self: Box<Self>, action: bool, world: &mut World) -> Box<GameState> {
        match action {
            true => {
                let mut players_controllers = world.write_resource::<::resource::PlayersControllers>();
                let mode = world.read_resource::<::resource::Mode>();
                let free_player = players_controllers
                    .iter()
                    .take(mode.number_of_player())
                    .enumerate()
                    .find(|&(_, player_gamepad_id)| player_gamepad_id.is_none())
                    .map(|(player_number, _)| player_number);

                if let Some(free_player) = free_player {
                    players_controllers[free_player] = Some(::resource::Controller::new_keyboard(&self.controller));
                }
            },
            false => (),
        }
        self.stacked_state
    }
}

impl GameState for ValidateBuildedController {
    fn update_draw_ui(self: Box<Self>, world: &mut World) -> Box<GameState> {
        let mut text = world.write_resource::<::resource::Text>();
        let font = world.read_resource::<::resource::Font>();
        let joystick_description = ::resource::Control::iter_variants()
            .zip(self.controller.iter())
            .map(|(control, key)| { format!("{} ↔ {:?}", control, key) })
            .collect::<Vec<_>>();

        text.global = ::util::joystick_description_layout(joystick_description, '↔', self.menu.len(), &font);
        text.global.extend(self.menu.glyphs(&font));

        self
    }

    fn winit_event(mut self: Box<Self>, event: ::winit::Event, world: &mut World) -> Box<GameState> {
        let action = {
            let controllers = world.read_resource::<::resource::PlayersControllers>();
            let mut possible_next_controllers = controllers.clone();
            if let Some(c) = possible_next_controllers.iter_mut().find(|c| c.is_none()) {
                *c = Some(::resource::Controller::new_keyboard(&self.controller));
            } else {
                // No available controller
                return self.stacked_state;
            }
            self.menu.winit_event(event, None, &::resource::PlayersControllers(possible_next_controllers))
        };
        if let Some(action) = action {
            self.process_action(action, world)
        } else {
            self
        }
    }

    fn gilrs_event(
        mut self: Box<Self>,
        _id: usize,
        event: ::gilrs::EventType,
        world: &mut World,
    ) -> Box<GameState> {
        if let Some(action) = self.menu.gilrs_event(event) {
            self.process_action(action, world)
        } else {
            self
        }
    }

    fn gilrs_gamepad_state(
        self: Box<Self>,
        _id: usize,
        _gamepad: &::gilrs::Gamepad,
        _world: &mut World,
    ) -> Box<GameState> {
        self
    }

    fn paused(&self, _world: &World) -> bool {
        true
    }
}

pub struct BuildController {
    current_input: ::resource::Control,
    controller: Vec<::winit::VirtualKeyCode>,
    stacked_state: Box<GameState>,
}

impl BuildController {
    pub fn new(stacked_state: Box<GameState>) -> Self {
        BuildController {
            current_input: ::resource::Control::iter_variants().next().unwrap(),
            controller: vec![],
            stacked_state,
        }
    }
}

impl GameState for BuildController {
    fn update_draw_ui(self: Box<Self>, world: &mut World) -> Box<GameState> {
        let mut text = world.write_resource::<::resource::Text>();
        let font = world.read_resource::<::resource::Font>();
        text.global = ::util::menu_layout(vec![format!("Press key for {}", self.current_input)], None, &font);
        self
    }

    fn winit_event(mut self: Box<Self>, event: ::winit::Event, _world: &mut World) -> Box<GameState> {
        match event {
            ::winit::Event::WindowEvent {
                event:
                    ::winit::WindowEvent::KeyboardInput {
                        input:
                            ::winit::KeyboardInput {
                                state: ::winit::ElementState::Pressed,
                                virtual_keycode: Some(virtual_keycode),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                self.controller.push(virtual_keycode);
                if let Some(control) =  ::resource::Control::iter_variants().skip_while(|v| *v != self.current_input).skip(1).next() {
                    Box::new(BuildController {
                        current_input: control,
                        controller: self.controller.clone(),
                        stacked_state: self.stacked_state,
                    }) as Box<_>
                } else {
                    Box::new(ValidateBuildedController::new(self.controller.clone(), self.stacked_state)) as Box<_>
                }
            }
            _ => self,
        }
    }

    fn gilrs_event(
        self: Box<Self>,
        _id: usize,
        _event: ::gilrs::EventType,
        _world: &mut World,
    ) -> Box<GameState> {
        println!("{:?}", _event);
        self
    }

    fn gilrs_gamepad_state(
        self: Box<Self>,
        _id: usize,
        _gamepad: &::gilrs::Gamepad,
        _world: &mut World,
    ) -> Box<GameState> {
        self
    }

    fn paused(&self, _world: &World) -> bool {
        true
    }
}

pub struct Game {
    players_menus: [Option<::menu::Menu<GameMenuAction>>; 3],
    space_return: [bool; 2],
}

impl Game {
    pub fn new() -> Self {
        Game {
            players_menus: [
                Some(Game::create_menu()),
                Some(Game::create_menu()),
                Some(Game::create_menu()),
            ],
            space_return: [false; 2],
        }
    }

    fn create_menu() -> ::menu::Menu<GameMenuAction> {
        use self::GameMenuAction::*;
        ::menu::MenuBuilder::new()
            .add_middle("Resume".to_string(), Resume)
            .add_middle("Main".to_string(), MainMenu)
            .build()
    }

    fn process_action(mut self: Box<Self>, player: usize, action: GameMenuAction, world: &mut World) -> Box<GameState> {
        use self::GameMenuAction::*;
        match action {
            Resume => self.players_menus[player] = None,
            MainMenu => return Box::new(GlobalMenu::new(world)) as Box<_>,
        }
        self
    }
}

// TODO: add tolerance
//       add disconnect device
#[derive(Clone, Copy)]
enum GameMenuAction {
    Resume,
    MainMenu,
}

impl GameState for Game {
    fn update_draw_ui(self: Box<Self>, world: &mut World) -> Box<GameState> {
        let mut text = world.write_resource::<::resource::Text>();
        let font = world.read_resource::<::resource::Font>();
        let mode = world.read_resource::<::resource::Mode>();
        let players_controllers = world.read_resource::<::resource::PlayersControllers>();

        text.global = vec![];

        let number_of_player = world.read_resource::<::resource::Mode>().number_of_player();
        let number_of_controllers = players_controllers.iter().filter(|g| g.is_some()).count();

        for (player, menu) in self.players_menus.iter().enumerate().take(mode.number_of_player()) {
            if players_controllers[player].is_none() {
                let lines = vec![
                    "Press [START] to join with a gamepad".to_string(),
                    "Press [SPACE]+[RETURN] to configure a keyboard".to_string(),
                ];
                text.players[player] = ::util::menu_layout(lines, None, &font);
            } else if let &Some(ref menu) = menu {
                text.players[player] = menu.glyphs(&font);
            } else if number_of_player != number_of_controllers {
                text.players[player] = ::util::menu_layout(vec!["Waiting for other players".to_string()], None, &font);
            } else {
                text.players[player] = vec![];
            }
        }

        self
    }

    fn winit_event(mut self: Box<Self>, event: ::winit::Event, world: &mut World) -> Box<GameState> {
        match event {
            ::winit::Event::WindowEvent {
                event:
                    ::winit::WindowEvent::KeyboardInput {
                        input:
                            ::winit::KeyboardInput {
                                state,
                                virtual_keycode: Some(::winit::VirtualKeyCode::Space),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                self.space_return[0] = state == ::winit::ElementState::Pressed;
            },
            ::winit::Event::WindowEvent {
                event:
                    ::winit::WindowEvent::KeyboardInput {
                        input:
                            ::winit::KeyboardInput {
                                state,
                                virtual_keycode: Some(::winit::VirtualKeyCode::Return),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                self.space_return[1] = state == ::winit::ElementState::Pressed;
            },
            _ => (),
        }
        if self.space_return.iter().all(|&s| s) {
            self.space_return = [false; 2];
            return Box::new(BuildController::new(self)) as Box<_>;
        }

        let (player, state, control) = {
            let players_controllers = world.read_resource::<::resource::PlayersControllers>();
            let option = players_controllers.convert_event(event.clone());
            if option.is_none() {
                return self;
            }
            option.unwrap()
        };

        if let ::resource::Control::Menu = control {
            if ::winit::ElementState::Pressed == state {
                if self.players_menus[player].is_some() {
                    self.players_menus[player] = None;
                } else {
                    self.players_menus[player] = Some(Game::create_menu());
                }
            }
            self
        } else if self.players_menus[player].is_some() {
            let mut player_action = None;
            if ::winit::ElementState::Pressed == state {
                if let Some(ref mut menu) = self.players_menus[player] {
                    if let Some(action) = menu.control_event(control) {
                        player_action = Some((player, action));
                    }
                }
            }
            if let Some((player, action)) = player_action {
                self.process_action(player, action, world)
            } else {
                self
            }
        } else {
            let mut players_controllers = world.write_resource::<::resource::PlayersControllers>();

            // Update controller state
            let controller = if let Some(::resource::Controller::Keyboard(ref mut controller)) = players_controllers[player] {
                controller
            } else {
                unreachable!();
            };

            controller.pressed.retain(|&c| c != control);
            if state == ::winit::ElementState::Pressed {
                controller.pressed.push(control);
            }

            // Update control
            let players_entities = world.read_resource::<::resource::PlayersEntities>();
            let mut flight_controls = world.write::<::component::FlightControl>();

            let flight_control = players_entities[player].and_then(|entity| flight_controls.get_mut(entity));
            if let Some(flight_control) = flight_control {
                let boost = controller.pressed.contains(&::resource::Control::Boost) as i32;
                let up = controller.pressed.contains(&::resource::Control::Up) as i32;
                let down = controller.pressed.contains(&::resource::Control::Down) as i32;
                let right = controller.pressed.contains(&::resource::Control::Right) as i32;
                let left = controller.pressed.contains(&::resource::Control::Left) as i32;
                flight_control.power = boost as f32;
                flight_control.y_direction = (up - down) as f32;
                flight_control.x_direction = (right-left) as f32;
            }
            self
        }
    }

    fn gilrs_event(
        self: Box<Self>,
        id: usize,
        event: ::gilrs::EventType,
        world: &mut World,
    ) -> Box<GameState> {
        // TODO: Menu actions
        // Game actions
        let mut players_controllers = world.write_resource::<::resource::PlayersControllers>();

        match event {
            EventType::Disconnected => {
                for player_controller in players_controllers.iter_mut() {
                    if let Some(::resource::Controller::Gamepad(player_controller_id)) = *player_controller {
                        if player_controller_id == id {
                            *player_controller = None;
                        }
                    }
                }
                return self;
            }
            _ => (),
        }

        let mode = world.read_resource::<::resource::Mode>();
        let players_entities = world.read_resource::<::resource::PlayersEntities>();
        let mut flight_controls = world.write::<::component::FlightControl>();

        let player = if let ::resource::Mode::Mode1Player = *mode {
            Some(0)
        } else {
            players_controllers
                .iter()
                .enumerate()
                .find(|&(_, player_controller)| {
                    if let Some(::resource::Controller::Gamepad(player_controller_id)) = *player_controller {
                        player_controller_id == id
                    } else {
                        false
                    }
                })
                .map(|(player_number, _)| player_number)
        };

        let free_player = players_controllers
            .iter()
            .take(mode.number_of_player())
            .enumerate()
            .find(|&(_, player_gamepad_id)| player_gamepad_id.is_none())
            .map(|(player_number, _)| player_number);

        if let Some(player) = player {
            let flight_control =
                players_entities[player].and_then(|entity| flight_controls.get_mut(entity));

            if let Some(flight_control) = flight_control {
                match event {
                    EventType::AxisChanged(Axis::LeftStickX, value, _)
                    | EventType::AxisChanged(Axis::RightStickX, value, _) => {
                        flight_control.x_direction = value;
                    }
                    EventType::AxisChanged(Axis::LeftStickY, value, _)
                    | EventType::AxisChanged(Axis::RightStickY, value, _) => {
                        flight_control.y_direction = value;
                    }
                    EventType::ButtonChanged(Button::LeftTrigger2, value, _)
                    | EventType::ButtonChanged(Button::RightTrigger2, value, _) => {
                        flight_control.power = value;
                    }
                    _ => (),
                }
            }
        } else if let Some(free_player) = free_player {
            match event {
                EventType::ButtonPressed(Button::Start, _) => {
                    players_controllers[free_player] = Some(::resource::Controller::Gamepad(id));
                }
                _ => (),
            }
        }
        self
    }

    fn gilrs_gamepad_state(
        self: Box<Self>,
        _id: usize,
        _gamepad: &::gilrs::Gamepad,
        _world: &mut World,
    ) -> Box<GameState> {
        self
    }

    fn paused(&self, world: &World) -> bool {
        let number_of_player = world.read_resource::<::resource::Mode>().number_of_player();
        let number_of_gamepads = world.read_resource::<::resource::PlayersControllers>().iter().filter(|g| g.is_some()).count();
        number_of_player != 1 && number_of_player != number_of_gamepads
    }
}

struct GlobalMenu {
    menu: ::menu::Menu<GlobalMenuAction>,
}

impl GlobalMenu {
    pub fn new(world: &::specs::World) -> Self {
        use self::GlobalMenuAction::*;

        let players = world.read_resource::<::resource::Mode>().number_of_player();

        let menu = ::menu::MenuBuilder::new()
            .add_left_right(format!("Players: {}", players), ReducePlayers, IncreasePlayers)
            .add_middle("New map".to_string(), NewMap)
            .build();

        GlobalMenu {
            menu,
        }
    }

    fn process_action(mut self: Box<Self>, action: GlobalMenuAction, world: &mut World) -> Box<GameState> {
        use self::GlobalMenuAction::*;
        match action {
            ReducePlayers => {
                {
                    let mut mode = world.write_resource::<::resource::Mode>();
                    mode.reduce();
                    let players = mode.number_of_player();
                    self.menu.reset_name(0, format!("Players: {}", players));
                }
                world.reset_for_mode();
                self
            },
            IncreasePlayers => {
                {
                    let mut mode = world.write_resource::<::resource::Mode>();
                    mode.increase();
                    let players = mode.number_of_player();
                    self.menu.reset_name(0, format!("Players: {}", players));
                }
                world.reset_for_mode();
                self
            },
            NewMap => Box::new(NewMapMenu::new(world)) as Box<_>,
        }
    }
}

#[derive(Clone, Copy)]
enum GlobalMenuAction {
    ReducePlayers,
    IncreasePlayers,
    NewMap,
}

impl GameState for GlobalMenu {
    fn update_draw_ui(self: Box<Self>, world: &mut World) -> Box<GameState> {
        let mut text = world.write_resource::<::resource::Text>();
        let font = world.read_resource::<::resource::Font>();
        text.global = self.menu.glyphs(&font);

        self
    }

    fn winit_event(mut self: Box<Self>, event: ::winit::Event, world: &mut World) -> Box<GameState> {
        let action = {
            let controllers = world.read_resource::<::resource::PlayersControllers>();
            self.menu.winit_event(event, None, &controllers)
        };
        if let Some(action) = action {
            self.process_action(action, world)
        } else {
            self
        }
    }

    fn gilrs_event(
        mut self: Box<Self>,
        _id: usize,
        event: ::gilrs::EventType,
        world: &mut World,
    ) -> Box<GameState> {
        if let Some(action) = self.menu.gilrs_event(event) {
            self.process_action(action, world)
        } else {
            self
        }
    }

    fn gilrs_gamepad_state(
        self: Box<Self>,
        _id: usize,
        _gamepad: &::gilrs::Gamepad,
        _world: &mut World,
    ) -> Box<GameState> {
        self
    }

    fn paused(&self, _world: &World) -> bool {
        true
    }
}

struct NewMapMenu {
    menu: ::menu::Menu<NewMapMenuAction>,
}

impl NewMapMenu {
    pub fn new(_world: &::specs::World) -> Self {
        use self::NewMapMenuAction::*;
        let menu = ::menu::MenuBuilder::new()
            .add_middle("Play".to_string(), Play)
            .build();

        NewMapMenu {
            menu,
        }
    }

    fn process_action(self: Box<Self>, action: NewMapMenuAction, world: &mut World) -> Box<GameState> {
        use self::NewMapMenuAction::*;

        match action {
            Play => {
                ::level::LevelBuilder {
                    half_size: 9,
                    x_shift: false,
                    y_shift: false,
                    z_shift: false,
                    percent: 5.0,
                    unit: 1.0,
                    columns: 0,
                    rocket_launcher: 1,
                    mine: 1,
                    target: 1,
                }.build(world);
                Box::new(Game::new()) as Box<GameState>
            }
        }
    }
}

#[derive(Clone, Copy)]
enum NewMapMenuAction {
    Play,
}

impl GameState for NewMapMenu {
    fn update_draw_ui(self: Box<Self>, world: &mut World) -> Box<GameState> {
        let mut text = world.write_resource::<::resource::Text>();
        let font = world.read_resource::<::resource::Font>();
        text.global = self.menu.glyphs(&font);

        self
    }

    fn winit_event(mut self: Box<Self>, event: ::winit::Event, world: &mut World) -> Box<GameState> {
        let action = {
            let controllers = world.read_resource::<::resource::PlayersControllers>();
            self.menu.winit_event(event, None, &controllers)
        };
        if let Some(action) = action {
            self.process_action(action, world)
        } else {
            self
        }
    }

    fn gilrs_event(
        mut self: Box<Self>,
        _id: usize,
        event: ::gilrs::EventType,
        world: &mut World,
    ) -> Box<GameState> {
        if let Some(action) = self.menu.gilrs_event(event) {
            self.process_action(action, world)
        } else {
            self
        }
    }

    fn gilrs_gamepad_state(
        self: Box<Self>,
        _id: usize,
        _gamepad: &::gilrs::Gamepad,
        _world: &mut World,
    ) -> Box<GameState> {
        self
    }

    fn paused(&self, _world: &World) -> bool {
        true
    }
}
