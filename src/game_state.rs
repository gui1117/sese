use specs::World;
use gilrs::{Axis, Button, EventType};

pub trait GameState {
    // TODO: Return bool = if next state gui must be set ?
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
    fn paused(&self) -> bool;
}

pub struct Game;

impl GameState for Game {
    fn update_draw_ui(self: Box<Self>, _world: &mut World) -> Box<GameState> {
        self
    }

    fn winit_event(self: Box<Self>, _event: ::winit::Event, _world: &mut World) -> Box<GameState> {
        self
    }

    fn gilrs_event(
        self: Box<Self>,
        id: usize,
        event: ::gilrs::EventType,
        world: &mut World,
    ) -> Box<GameState> {
        let players_entities = world.read_resource::<::resource::PlayersEntities>();
        let mut players_gamepads = world.write_resource::<::resource::PlayersGamepads>();
        let mut flight_controls = world.write::<::component::FlightControl>();

        let player = players_gamepads
            .iter()
            .enumerate()
            .find(|&(_, player_gamepad_id)| {
                player_gamepad_id.map(|p_id| p_id == id).unwrap_or(false)
            })
            .map(|(player_number, _)| player_number);

        let free_player = players_gamepads
            .iter()
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
                    players_gamepads[free_player] = Some(id);
                }
                _ => (),
            }
        }
        self
    }

    fn gilrs_gamepad_state(
        self: Box<Self>,
        _id: usize,
        gamepad: &::gilrs::Gamepad,
        world: &mut World,
    ) -> Box<GameState> {
        self
    }

    fn paused(&self) -> bool {
        false
    }
}
