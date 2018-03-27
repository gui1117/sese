use specs::{Join, World};
use conrod::Widget;

widget_ids! {
    pub struct Ids {
        master,
        left_col,
        middle_col,
        right_col,
        left_text,
        middle_text,
        right_text,
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
    fn update_draw_ui(self: Box<Self>, ui: &mut ::conrod::UiCell, ids: &Ids, world: &mut World) -> Box<GameState> {
        use conrod::{color, widget, Colorable, Positionable, Scalar, Sizeable, Widget};

        // Our `Canvas` tree, upon which we will place our text widgets.
        widget::Canvas::new().flow_right(&[
            (ids.left_col, widget::Canvas::new().color(color::BLACK)),
            (ids.middle_col, widget::Canvas::new().color(color::DARK_CHARCOAL)),
            (ids.right_col, widget::Canvas::new().color(color::CHARCOAL)),
        ]).set(ids.master, ui);

        const DEMO_TEXT: &'static str = "AAZERTYUIOPQSDFGHJKLMWXCVBN?.L";

        const PAD: Scalar = 20.0;

        widget::Text::new(DEMO_TEXT)
            .color(color::LIGHT_RED)
            .padded_w_of(ids.left_col, PAD)
            .mid_top_with_margin_on(ids.left_col, PAD)
            .left_justify()
            .line_spacing(10.0)
            .set(ids.left_text, ui);

        const AA: &'static str = "A\nB\nC\nD";

        widget::Text::new(AA)
            .color(color::LIGHT_GREEN)
            .padded_w_of(ids.middle_col, PAD)
            .middle_of(ids.middle_col)
            .center_justify()
            .line_spacing(2.5)
            .set(ids.middle_text, ui);

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
