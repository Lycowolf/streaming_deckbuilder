/// UI states for our game's push-down automaton

use crate::automaton::*;
use crate::game_control::*;
use quicksilver::prelude::*;
use serde::export::fmt::Debug;
use derivative::*;
use std::mem::take;

use super::widgets::*;
use crate::game_objects::{GameData, Globals, Card, Effect, BoardZone};
use crate::game_logic::BoardState;
use crate::loading::Assets;

pub const WINDOW_SIZE_W: f32 = 1280.0;
pub const WINDOW_SIZE_H: f32 = 800.0;
const PLAYER_BOARD_FROM_TOP: f32 = 300.0;
const BASE_Z_INDEX: f32 = 1.0; // widgets will be layered starting with this Z

// TODO: cache widgets?
#[derive(Derivative)]
#[derivative(Debug)]
pub struct GameEndState {
    boards: Vec<BoardState>,
    widgets: Vec<Box<dyn Widget>>,
}

// TODO: load fonts in LoadingState
impl GameEndState {
    pub fn new(boards: Vec<BoardState>, assets: Assets) -> Box<Self> {
        let mut widgets = Vec::new();

        // Exit game
        let loser = boards.iter()
            .find(|b| b.is_defeated())
            .expect("Someone had to lose")
            .player
            .clone();

        widgets.push(Box::new(Button::new(
            format!("It is over, {} lost", loser.name),
            Vector::new(UI_UNIT * 5.0, UI_UNIT * 5.0),
            BASE_Z_INDEX,
            &assets,
            Some(GameEvent::GameEnded),
        ),
        ) as Box<dyn Widget>);
        println!("It is over");

        Box::new(Self {
            boards,
            widgets,
        })
    }
}

// This is only a placeholder, to allow us to take() ourselves from &mut Self
impl Default for GameEndState {
    fn default() -> Self {
        Self {
            boards: Vec::new(),
            widgets: Vec::new(),
        }
    }
}

impl AutomatonState for GameEndState {
    fn event(&mut self, event: GameEvent) -> Box<dyn AutomatonState> {
        match event {
            // TODO: generalize to arbitrary window sizes
            GameEvent::IO(Event::MouseMoved(position)) => {
                for mut w in &mut self.widgets {
                   w.update_hovered(position);
                }
                Box::new(take(self))
            }
            GameEvent::IO(Event::MouseButton(MouseButton::Left, ButtonState::Released)) => {
                Box::new(take(self))
            }
            GameEvent::IO(Event::Key(Key::Escape, ButtonState::Released)) => {
                Box::new(GameEndedState {})
            }
            _ => Box::new(take(self))
        }
    }

    fn update(&mut self) -> Box<dyn AutomatonState> {
        Box::new(take(self))
    }

    // TODO: make widgets draw to Surface, and arrange the Surfaces
    fn draw(&self, window: &mut Window) -> () {
        // TODO Draw result

        for widget in &self.widgets {
            widget.draw(window).unwrap();
        }
    }
}