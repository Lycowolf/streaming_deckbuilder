/// UI states for our game's push-down automaton

use crate::automaton::*;
use quicksilver::prelude::*;

#[derive(Debug)]
pub struct LoadingState;

impl LoadingState {
    pub fn new() -> Box<Self> {
        Box::new(Self)
    }
}

impl AutomatonState for LoadingState {
    fn event(&self, event: GameEvent) -> ProcessingResult {
        match event {
            GameEvent::IO(Event::Key(Key::Return, ButtonState::Pressed)) => {
                let new_game = GameplayState::new();
                (StateAction::Replace(new_game), Some(GameEvent::Started))
            },
            _ => (StateAction::None, None)
        }
    }

    fn draw(&self, window: &mut Window, z_index: u32) -> () {
        window.draw_ex(&Circle::new((300, 300), 32), Col(Color::BLUE), Transform::IDENTITY, z_index);
    }
}

#[derive(Debug)]
pub struct GameplayState {
    timer: i32
}

impl GameplayState {
    pub fn new() -> Box<Self> {
        Box::new(Self {timer: 0})
    }
}

impl AutomatonState for GameplayState {
    fn event(&self, event: GameEvent) -> ProcessingResult {
        match event {
            GameEvent::IO(Event::Key(Key::Escape, ButtonState::Pressed)) => {
                (StateAction::Pop, None)
            },
            _ => (StateAction::None, None)
        }
    }

    fn update(&mut self) -> () {
        self.timer += 1;
    }

    fn draw(&self, window: &mut Window, z_index: u32) -> () {
        let rectangle = Rectangle::new((300, 300), (32, 32));
        window.draw_ex(&rectangle, Col(Color::RED), Transform::rotate(self.timer), z_index);
    }
}