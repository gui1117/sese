use specs::{Join, World};

pub trait GameState {
    // TODO: Return bool = if next state gui must be set ?
    fn update_draw_ui(self: Box<Self>, ui: &::imgui::Ui, world: &mut World) -> Box<GameState>;
    fn winit_event(
        self: Box<Self>,
        event: ::winit::Event,
        world: &mut World,
    ) -> Box<GameState>;
    fn gilrs_event(
        self: Box<Self>,
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
    fn update_draw_ui(self: Box<Self>, ui: &::imgui::Ui, world: &mut World) -> Box<GameState> {
        self
    }

    fn winit_event(
        self: Box<Self>,
        _event: ::winit::Event,
        _world: &mut World,
    ) -> Box<GameState> {
        self
    }

    fn gilrs_event(
        self: Box<Self>,
        _event: ::gilrs::EventType,
        _world: &mut World,
    ) -> Box<GameState> {
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
