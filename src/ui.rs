/// UI states for our game's push-down automaton

use crate::automaton::*;
use crate::game_logic::*;
use quicksilver::prelude::*;

#[derive(Debug)]
pub struct LoadingState {
    timer: u32
}

impl LoadingState {
    pub fn new() -> Box<Self> {
        Box::new(Self{ timer: 1 })
    }
}

impl AutomatonState for LoadingState {
    fn event(&self, board_state: &mut Option<BoardState>, event: GameEvent) -> ProcessingResult {
        match event {
            GameEvent::IO(Event::Key(Key::Return, ButtonState::Pressed)) | GameEvent::Timeout => {
                let new_game = Box::new(GameplayState::new());
                (StateAction::Replace(new_game), Some(GameEvent::Started))
            },
            _ => (StateAction::None, None)
        }
    }

    fn draw(&self, board_state: &Option<BoardState>, window: &mut Window, z_index: u32) -> () {
        // TODO draw boardstate.global
        // TODO draw hand
        window.draw_ex(&Circle::new((300, 300), 32), Col(Color::BLUE), Transform::IDENTITY, z_index);
    }

    fn update(&mut self, board_state: &mut Option<BoardState>) -> Option<GameEvent> {
        // TODO async load
        *board_state = Some(BoardState::setup(Some("test_deck")));


        if self.timer > 0 {
            self.timer -= 1;
        }

        if self.timer % 60 == 0 {
            println!("Seconds remaining: {}", self.timer / 60)
        }

        if self.timer == 0 {
            Some(GameEvent::Timeout)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct TakeTurnState {
    timer: i32
}

impl TakeTurnState {
    pub fn new() -> Box<Self> {
        Box::new(Self {timer: 0})
    }
}

impl AutomatonState for TakeTurnState {
    fn event(&self, board_state: &mut Option<BoardState>, event: GameEvent) -> ProcessingResult {
        match event {
            GameEvent::IO(Event::Key(Key::Escape, ButtonState::Released)) => {
                (StateAction::Pop, Some(GameEvent::GameEnded))
            },
            _ => (StateAction::None, None)
        }
    }

    fn update(&mut self, board_state: &mut Option<BoardState>) -> Option<GameEvent> {
        self.timer += 1;
        None
    }

    fn draw(&self, board_state: &Option<BoardState>, window: &mut Window, z_index: u32) -> () {
        let rectangle = Rectangle::new((300, 300), (32, 32));
        window.draw_ex(&rectangle, Col(Color::RED), Transform::rotate(self.timer), z_index);
    }
}