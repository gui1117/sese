use specs::{Join, World};

widget_ids! {
    pub struct Ids {
    }
}

pub trait GameState {
    // TODO: Return bool = if next state gui must be set ?
    fn update_draw_ui(self: Box<Self>, ui: &mut ::conrod::UiCell, ids: &Ids, world: &mut World) -> Box<GameState>;
    fn winit_event(
        self: Box<Self>,
        event: ::winit::Event,
        world: &mut World,
        ui: &mut ::conrod::Ui
    ) -> Box<GameState>;
    fn gilrs_event(
        self: Box<Self>,
        event: ::gilrs::EventType,
        world: &mut World,
        ui: &mut ::conrod::Ui
    ) -> Box<GameState>;
    fn gilrs_gamepad_state(
        self: Box<Self>,
        id: usize,
        gamepad: &::gilrs::Gamepad,
        world: &mut World,
        ui: &mut ::conrod::Ui
    ) -> Box<GameState>;
    fn quit(&self) -> bool {
        false
    }
    fn paused(&self) -> bool;
}

pub struct Game;

impl GameState for Game {
    fn update_draw_ui(self: Box<Self>, _ui: &mut ::conrod::UiCell, _ids: &Ids, _world: &mut World) -> Box<GameState> {
        self
    }
    fn winit_event(
        self: Box<Self>,
        _event: ::winit::Event,
        _world: &mut World,
        _ui: &mut ::conrod::Ui
    ) -> Box<GameState> {
        self
    }

    fn gilrs_event(
        self: Box<Self>,
        _event: ::gilrs::EventType,
        _world: &mut World,
        _ui: &mut ::conrod::Ui
    ) -> Box<GameState> {
        self
    }

    fn gilrs_gamepad_state(
        self: Box<Self>,
        _id: usize,
        gamepad: &::gilrs::Gamepad,
        world: &mut World,
        _ui: &mut ::conrod::Ui
    ) -> Box<GameState> {
        self
    }

    fn paused(&self) -> bool {
        false
    }
}
