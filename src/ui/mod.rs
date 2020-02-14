/// UI states for our game's push-down automaton

use crate::automaton::*;
use crate::game_logic::*;
use quicksilver::prelude::*;
use quicksilver::Future;
use serde::export::fmt::Debug;
use derivative::*;
use std::mem::take;

mod widgets;

use widgets::*;
use std::collections::HashMap;
use crate::game_objects::GameData;

pub const WINDOW_SIZE_W: f32 = 800.0;
pub const WINDOW_SIZE_H: f32 = 600.0;
const PLAYER_BOARD_FROM_TOP: f32 = 200.0;

#[derive(Debug, Default)]
pub struct LoadingState {
    board_state: BoardState,
}

impl LoadingState {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            board_state: BoardState::load_board("cards.json"),
        })
    }
}

impl AutomatonState for LoadingState {
    fn event(&mut self, event: GameEvent) -> Box<dyn AutomatonState> {
        Box::new(take(self))
    }

    fn update(&mut self) -> Box<dyn AutomatonState> {
        GameplayState::new_with_ui(take(self).board_state) // TODO async load
    }

    fn draw(&self, window: &mut Window) -> () {
        window.draw(&Circle::new((300, 300), 32), Col(Color::BLUE));
    }
}

// TODO: cache widgets?
#[derive(Derivative)]
#[derivative(Debug)]
pub struct TakeTurnState {
    gameplay_state: Box<GameplayState>,
    widgets: Vec<Box<dyn Widget>>,
    #[derivative(Debug = "ignore")]
    font: Font,
}

// TODO: load fonts in LoadingState
impl TakeTurnState {
    pub fn new(gameplay_state: Box<GameplayState>) -> Box<Self> {
        let font = Font::load("Roboto-Italic.ttf").wait().expect("Can't load font file");

        let card_size = Vector::new(CARD_WIDTH, CARD_HEIGHT);
        let card_width = card_size.x_comp();
        let h_gap = Vector::new(CARD_PAD_HORIZONTAL, 0);
        let hand_offset_top_left = Vector::new(180.0, 410.0);

        let mut widgets = Vec::new();
        widgets.push(Box::new(CardWidget::new(
            &"Draw pile\n\nClick to\nend turn".to_string(),
            Vector::new(UI_UNIT * 3.0, PLAYER_BOARD_FROM_TOP + (WINDOW_SIZE_H - PLAYER_BOARD_FROM_TOP) / 2.0 - CARD_HEIGHT / 2.0),
            card_size,
            &font,
            GameEvent::EndTurn,
        ),
        ) as Box<dyn Widget>);

        let hand = &gameplay_state.get_board().hand.cards;

        for (num, card) in hand.clone().drain(..).enumerate() {
            let name = card.name.clone();
            let action_text = format!("Card {} clicked", name);
            widgets.push(Box::new(CardWidget::new(
                &name,
                hand_offset_top_left + ((card_width + h_gap) * num as f32),
                card_size,
                &font,
                GameEvent::CardPicked(num),
            ),
            ) as Box<dyn Widget>);
        }

        Box::new(Self {
            gameplay_state,
            widgets,
            font,
        })
    }
}

// This is only a placeholder, to allow us to take() ourself from &mut Self
impl Default for TakeTurnState {
    fn default() -> Self {
        *Self::new(Box::new(GameplayState::default()))
    }
}

impl AutomatonState for TakeTurnState {
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
                let found = self.widgets.iter()
                    .map(|widg| { widg.maybe_activate() }) // translate to events (maybe all None)
                    .find(|event| { event.is_some() }) // maybe find first Some
                    .map(|some_event| { some_event.unwrap() }); // if some, unwrap
                match found {
                    Some(event) => self.gameplay_state.event(event),
                    None => Box::new(take(self))
                }
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
        let horizontal_divider = Line::new(
            Vector::new(0, PLAYER_BOARD_FROM_TOP),
            Vector::new(WINDOW_SIZE_W, PLAYER_BOARD_FROM_TOP),
        );
        window.draw(&horizontal_divider, Col(Color::from_rgba(100, 100, 100, 1.0)));

        for widget in &self.widgets {
            widget.draw(window).unwrap();
        }
    }
}