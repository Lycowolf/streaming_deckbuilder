/// UI states for our game's push-down automaton

use crate::automaton::*;
use crate::game_logic::*;
use std::collections::HashMap;
use quicksilver::prelude::*;
use derivative::*;
use std::mem::take;

mod widgets;
mod board_display;

use widgets::*;
use crate::game_objects::{GameData, Globals, Card, Effect, BoardZone};
use board_display::BoardDisplay;

pub const WINDOW_SIZE_W: f32 = 1280.0;
pub const WINDOW_SIZE_H: f32 = 800.0;

// TODO: cache widgets?
#[derive(Derivative)]
#[derivative(Debug)]
pub struct TakeTurnState {
    gameplay_state: Box<GameplayState>,
    display: Box<BoardDisplay>,
}

// TODO: load fonts in LoadingState
impl TakeTurnState {
    pub fn new(gameplay_state: Box<GameplayState>) -> Box<Self> {

        let mut handler_dict = HashMap::<BoardZone, CardHandler>::new();

        handler_dict.insert(BoardZone::Hand, Box::new(|idx, _card, _zone| Some(GameEvent::CardPicked(idx))));

        for (_, store) in gameplay_state.get_board().stores.iter().enumerate() {
            handler_dict.insert(store.menu.zone, Box::new(|idx, _card, zone| Some(GameEvent::CardBought(zone, idx))));
        }

        let display = BoardDisplay::new(&gameplay_state, handler_dict, WINDOW_SIZE_W, WINDOW_SIZE_H);

        Box::new(Self {
            gameplay_state,
            display,
        })
    }
}

// This is only a placeholder, to allow us to take() ourselves from &mut Self
impl Default for TakeTurnState {
    fn default() -> Self {
        Self {
            gameplay_state: Box::new(GameplayState::default()),
            display: Box::new(BoardDisplay::default()),
        }
    }
}

impl AutomatonState for TakeTurnState {
    fn event(&mut self, event: GameEvent) -> Box<dyn AutomatonState> {
        match event {
            GameEvent::IO(Event::Key(Key::Escape, ButtonState::Released)) => {
                Box::new(GameEndedState {})
            }
            GameEvent::IO(io) => {
                match self.display.handle_io(io) {
                    Some(event) => self.gameplay_state.event(event),
                    None => Box::new(take(self))
                }
            }
            _ => Box::new(take(self))
        }
    }

    fn update(&mut self) -> Box<dyn AutomatonState> {
        self.display.update();
        Box::new(take(self))
    }

    // TODO: make widgets draw to Surface, and arrange the Surfaces
    fn draw(&self, window: &mut Window) -> () {
        self.display.draw(window)
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct TargetingState {
    gameplay_state: Box<GameplayState>,
    display: Box<BoardDisplay>,

    #[derivative(Debug = "ignore")]

    acting_card_source: BoardZone,
    acting_card_idx: usize,
    target_zone: BoardZone
}

// TODO: load fonts in LoadingState
impl TargetingState {
    pub fn new(gameplay_state: Box<GameplayState>, acting_card_source: BoardZone, acting_card_idx: usize, target_zone: BoardZone) -> Box<Self> {

        let mut handler_dict = HashMap::<BoardZone, CardHandler>::new();
        handler_dict.insert(target_zone, Box::new(move  |idx, card, zone| Some(GameEvent::CardTargeted(acting_card_source, acting_card_idx, zone, idx))));
    
        let display = BoardDisplay::new(&gameplay_state, handler_dict, WINDOW_SIZE_W, WINDOW_SIZE_H);
    
        Box::new(Self {
            gameplay_state,
            display,
            acting_card_source,
            acting_card_idx,
            target_zone
        })
    }

    fn response_event(&self, target: Option<usize>) -> GameEvent {
        match target {
            Some(idx) => GameEvent::CardTargeted(self.acting_card_source,
                                                 self.acting_card_idx,
                                                 self.target_zone,
                                                 idx),
            None => GameEvent::CardTargeted(self.acting_card_source,
                                            self.acting_card_idx,
                                            BoardZone::None,
                                            0)
        }
    }

    fn target_selected(&mut self, target: Option<usize>) -> Box<dyn AutomatonState> {
        let event = self.response_event(target);

        self.gameplay_state.event(event)
    }
}

// This is only a placeholder, to allow us to take() ourselves from &mut Self
impl Default for TargetingState {
    fn default() -> Self {
        Self {
            gameplay_state: Box::new(GameplayState::default()),
            display: Box::new(BoardDisplay::default()),
            acting_card_source: BoardZone::None,
            acting_card_idx: 0,
            target_zone: BoardZone::None,
        }
    }
}

impl AutomatonState for TargetingState {
    fn event(&mut self, event: GameEvent) -> Box<dyn AutomatonState> {
        match event {
            GameEvent::IO(Event::Key(Key::Escape, ButtonState::Released)) => {
                Box::new(GameEndedState {})
            }
            GameEvent::IO(Event::MouseButton(MouseButton::Right, ButtonState::Released)) => {
                // Cancel targetting
                let event = GameEvent::CardTargeted(BoardZone::None, 0, BoardZone::None, 0);
                self.gameplay_state.event(event) 
            }
            GameEvent::IO(io) => {
                match self.display.handle_io(io) {
                    Some(event) => self.gameplay_state.event(event),
                    None => Box::new(take(self))
                }
            }
            _ => Box::new(take(self))
        }
    }

    fn update(&mut self) -> Box<dyn AutomatonState> {
        self.display.update();
        Box::new(take(self))
    }

    // TODO: make widgets draw to Surface, and arrange the Surfaces
    fn draw(&self, window: &mut Window) -> () {
        self.display.draw(window)
    }
}