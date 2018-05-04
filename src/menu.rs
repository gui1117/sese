pub struct Menu<A> {
    buttons: Vec<Button<A>>,
    cursor: usize,
}

impl<A: Copy> Menu<A> {
    fn new(buttons: Vec<Button<A>>) -> Self {
        assert!(buttons.len() != 0);
        Menu {
            buttons,
            cursor: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.buttons.len()
    }

    pub fn glyphs(&self, font: &::rusttype::Font<'static>) -> Vec<::rusttype::PositionedGlyph<'static>> {
        let texts = self.buttons.iter()
            .map(|b| b.name.clone())
            .collect::<Vec<_>>();

        ::util::menu_layout(texts, Some(self.cursor), font)
    }

    pub fn reset_name(&mut self, button: usize, name: String) {
        self.buttons[button].name = name;
    }

    pub fn up(&mut self) {
        self.cursor -= 1;
        self.cursor %= self.buttons.len();
    }

    pub fn down(&mut self) {
        self.cursor += 1;
        self.cursor %= self.buttons.len();
    }

    pub fn left(&mut self) -> A {
        self.buttons[self.cursor].left
    }

    pub fn right(&mut self) -> A {
        self.buttons[self.cursor].right
    }

    pub fn control_event(&mut self, event: ::resource::Control) -> Option<A> {
        match event {
            ::resource::Control::Up => {
                self.up();
                None
            },
            ::resource::Control::Down => {
                self.down();
                None
            }
            ::resource::Control::Left => Some(self.left()),
            ::resource::Control::Right => Some(self.right()),
            _ => None,
        }
    }

    pub fn winit_event(&mut self, event: ::winit::Event, player: Option<usize>, controllers: &::resource::PlayersControllers) -> Option<A> {
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
                let control = match virtual_keycode {
                    ::winit::VirtualKeyCode::Up => Some(::resource::Control::Up),
                    ::winit::VirtualKeyCode::Down => Some(::resource::Control::Down),
                    ::winit::VirtualKeyCode::Left => Some(::resource::Control::Left),
                    ::winit::VirtualKeyCode::Right => Some(::resource::Control::Right),
                    _ => {
                        controllers.iter()
                            .enumerate()
                            .filter(|&(i, _)| if let Some(player) = player {
                                i == player
                            } else {
                                true
                            })
                            .filter_map(|(_, c)| if let &Some(::resource::Controller::Keyboard(ref controls)) = c {
                                Some(controls)
                            } else {
                                None
                            })
                            .flat_map(|c| c.mapping.iter())
                            .find(|c| c.0 == virtual_keycode)
                            .map(|c| c.1)
                    }
                };

                if let Some(control) = control {
                    self.control_event(control)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn gilrs_event(&mut self, event: ::gilrs::EventType) -> Option<A> {
        match event {
            ::gilrs::EventType::AxisChanged(::gilrs::Axis::LeftStickX, value, _)
            | ::gilrs::EventType::AxisChanged(::gilrs::Axis::RightStickX, value, _) => {
                if value == 1.0 {
                    Some(self.right())
                } else if value == -1.0 {
                    Some(self.left())
                } else {
                    None
                }
            }
            ::gilrs::EventType::AxisChanged(::gilrs::Axis::LeftStickY, value, _)
            | ::gilrs::EventType::AxisChanged(::gilrs::Axis::RightStickY, value, _) => {
                if value == 1.0 {
                    self.up();
                } else if value == -1.0 {
                    self.down();
                }
                None
            }
            ::gilrs::EventType::ButtonPressed(::gilrs::Button::South, _)
            | ::gilrs::EventType::ButtonPressed(::gilrs::Button::DPadDown, _) => {
                self.down();
                None
            }
            ::gilrs::EventType::ButtonPressed(::gilrs::Button::North, _)
            | ::gilrs::EventType::ButtonPressed(::gilrs::Button::DPadUp, _) => {
                self.up();
                None
            }
            ::gilrs::EventType::ButtonPressed(::gilrs::Button::East, _)
            | ::gilrs::EventType::ButtonPressed(::gilrs::Button::DPadRight, _) => {
                Some(self.right())
            }
            ::gilrs::EventType::ButtonPressed(::gilrs::Button::West, _)
            | ::gilrs::EventType::ButtonPressed(::gilrs::Button::DPadLeft, _) => {
                Some(self.left())
            }
            _ => None,
        }
    }
}

struct Button<A> {
    name: String,
    left: A,
    right: A,
}

pub struct MenuBuilder<A> {
    buttons: Vec<Button<A>>,
}

impl<A: Copy> MenuBuilder<A> {
    pub fn new() -> Self {
        MenuBuilder {
            buttons: vec![],
        }
    }

    pub fn add_left_right(mut self, name: String, left: A, right: A) -> Self {
        self.buttons.push(Button {
            name,
            left,
            right,
        });
        self
    }

    pub fn add_middle(mut self, name: String, middle: A) -> Self {
        self.buttons.push(Button {
            name,
            left: middle,
            right: middle,
        });
        self
    }

    pub fn build(self) -> Menu<A> {
        Menu::new(self.buttons)
    }
}
