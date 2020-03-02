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
use crate::game_objects::{GameData, Globals, Card, Effect, BoardZone};
use crate::loading::load_board;

pub const WINDOW_SIZE_W: f32 = 1280.0;
pub const WINDOW_SIZE_H: f32 = 800.0;
const PLAYER_BOARD_FROM_TOP: f32 = 300.0;
const FONT_FILE: &'static str = "Teko-Regular.ttf";

#[derive(Debug, Default)]
pub struct LoadingState {
    board_state: BoardState,
}

impl LoadingState {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            board_state: load_board("cards.json"),
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
        let font = Font::load(FONT_FILE).wait().expect("Can't load font file");
        let mut widgets = Vec::new();

        // Next turn button
        widgets.push(Box::new(Button::new(
            "End\nturn".to_string(),
            Vector::new(UI_UNIT * 7.0, UI_UNIT * 45.0),
            &font,
            Some(GameEvent::EndTurn),
        ),
        ) as Box<dyn Widget>);

        // Hand
        let hand_zone = CardZone::<CardFull>::from_container(&gameplay_state.get_board().hand,
                                                             Vector::new(13.0 * UI_UNIT, 35.0 * UI_UNIT),
                                                             ZoneDirection::Horizontal, 
                                                             &font,
                                                             |idx, card, name| Some(GameEvent::CardPicked(idx)));
        widgets.push(Box::new(hand_zone));

        // TODO: refactor stores: store by name is weird

        // Stores
        let mut base_store_position = Vector::new(UI_UNIT, PLAYER_BOARD_FROM_TOP);
        for (num, store) in gameplay_state.get_board().stores.iter().enumerate() {
            let shop_zone = CardZone::<CardIcon>::from_container(&store.menu,
                                                                 base_store_position + Vector::new(0, UI_UNIT * 4.0 * num as f32), // 4U widget height + 1U padding + 1U gap
                                                                 ZoneDirection::Horizontal,
                                                                 &font,
                                                                 |idx, card, name| Some(GameEvent::CardBought(name, idx)));
            widgets.push(Box::new(shop_zone));
        }

        // buildings
        let mut base_playzone_position = Vector::new(60.0 * UI_UNIT, PLAYER_BOARD_FROM_TOP);

        let build_zone = CardZone::<CardIcon>::from_container(&gameplay_state.get_board().buildings,
                                                              base_playzone_position,
                                                              ZoneDirection::Vertical,
                                                              &font,
                                                              |idx, card, name| None);
        widgets.push(Box::new(build_zone));

        // kaiju_zone
        let kaiju_position = base_playzone_position + Vector::new(UI_UNIT * 9.0, 0); // 7U widget height + 1U padding + 1U gap
        let kaiju_zone = CardZone::<CardIcon>::from_container(&gameplay_state.get_board().kaiju_zone,
                                                              kaiju_position,
                                                              ZoneDirection::Vertical,
                                                              &font,
                                                              |idx, card, zone_id| None);
        widgets.push(Box::new(kaiju_zone));

        let base_numbers_position = Vector::new(4.0 * UI_UNIT, PLAYER_BOARD_FROM_TOP + 15.0 * UI_UNIT);

        for (num, currency) in Globals::in_game().iter().enumerate() {
            let value = gameplay_state.get_board().globals.get(*currency);
            widgets.push(Box::new(Button::new(
                format!("{:?}\n {}", currency, value),
                base_numbers_position + Vector::new(UI_UNIT * 5.0, 0) * num as f32,
                &font,
                None
            )));
        }

        Box::new(Self {
            gameplay_state,
            widgets,
            font,
        })
    }
}

// This is only a placeholder, to allow us to take() ourselves from &mut Self
impl Default for TakeTurnState {
    fn default() -> Self {
        Self {
            gameplay_state: Box::new(GameplayState::default()),
            widgets: Vec::new(),
            font: Font::load(FONT_FILE).wait().expect("Can't load font file"), // TODO: use preloaded font (or make it optional)
        }
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