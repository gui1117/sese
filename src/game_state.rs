use specs::{Join, World};
use conrod::Widget;

widget_ids! {
    pub struct Ids {
        triangles
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
        // let rect = ui.rect_of(ui.window).unwrap();
        // let (l, r, b, t) = rect.l_r_b_t();
        // let (c1, c2, c3) = (::conrod::color::RED.to_rgb(), ::conrod::color::GREEN.to_rgb(), ::conrod::color::BLUE.to_rgb());

        // let triangles = [
        //     ::conrod::widget::primitive::shape::triangles::Triangle([([l, b], c1), ([l, t], c2), ([r, t], c3)]),
        //     ::conrod::widget::primitive::shape::triangles::Triangle([([r, t], c1), ([r, b], c2), ([l, b], c3)]),
        // ];

        // ::conrod::widget::Triangles::multi_color(triangles.iter().cloned())
        //     .with_bounding_rect(rect)
        //     .set(ids.triangles, ui);

        ::conrod::widget::Rectangle::fill_with([100.0, 100.0], ::conrod::color::Color::Rgba(1.0, 0.0, 1.0, 0.5))
            .set(ids.triangles, ui);

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
