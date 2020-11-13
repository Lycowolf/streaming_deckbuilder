/// UI states for our game's push-down automaton

use crate::automaton::*;
use crate::game_logic::*;
use quicksilver::prelude::*;
use quicksilver::lifecycle::{Event, Window};
use serde::export::fmt::Debug;
use std::collections::HashMap;

use super::widgets::*;
use crate::game_objects::{BoardZone, Globals}; //, GameData, Card, Effect, };

// pub const WINDOW_SIZE_W: f32 = 1280.0;
// pub const WINDOW_SIZE_H: f32 = 800.0;
const PLAYER_BOARD_FROM_TOP: f32 = 300.0;
const BASE_Z_INDEX: f32 = 1.0; // widgets will be layered starting with this Z

#[derive(Debug)]
pub struct BoardDisplay {
    widgets: Vec<Box<dyn Widget>>,
    window_w: f32,
    window_h: f32
}

impl BoardDisplay {
    pub fn new(gameplay_state: &GameplayState, handlers: HashMap<BoardZone, CardHandler>, window_w: f32, window_h: f32) -> Box<Self> {
        let assets = gameplay_state.get_assets();
        let mut widgets = Vec::new();

        // Nametag
        widgets.push(Box::new(Button::new(
            format!("Me: {}", gameplay_state.get_board().player.name),
            Vector::new(UI_UNIT * 5.0, UI_UNIT * 5.0),
            BASE_Z_INDEX,
            &assets,
            None,
        ),
        ) as Box<dyn Widget>);
        widgets.push(Box::new(Button::new(
            format!("Foe: {}", gameplay_state.get_opponent().player.name),
            Vector::new(UI_UNIT * 5.0, UI_UNIT * 10.0),
            BASE_Z_INDEX,
            &assets,
            None,
        ),
        ) as Box<dyn Widget>);

        // Next turn button
        widgets.push(Box::new(Button::new(
            format!("End turn\ndeck: {}", gameplay_state.get_board().deck.len()),
            Vector::new(UI_UNIT * 7.0, UI_UNIT * 45.0),
            BASE_Z_INDEX,
            &assets,
            Some(GameEvent::EndTurn),
        ),
        ) as Box<dyn Widget>);

        // Hand
        let hand_zone = CardZone::<CardFull>::from_container(&gameplay_state.get_board().hand,
                                                             Vector::new(13.0 * UI_UNIT, 35.0 * UI_UNIT),
                                                             ZoneDirection::Horizontal,
                                                             BASE_Z_INDEX,
                                                             &assets,
                                                             &handlers);
        widgets.push(Box::new(hand_zone));

        // TODO: refactor stores: store by name is weird

        // Stores
        let base_store_position = Vector::new(UI_UNIT, PLAYER_BOARD_FROM_TOP);
        for (num, store) in gameplay_state.get_board().stores.iter().enumerate() {
            let shop_zone = CardZone::<CardIcon>::from_container(&store.menu,
                                                                 base_store_position + Vector::new(0, UI_UNIT * 4.0 * num as f32), // 4U widget height + 1U padding + 1U gap
                                                                 ZoneDirection::Horizontal,
                                                                 BASE_Z_INDEX,
                                                                 &assets,
                                                                 &handlers);
            widgets.push(Box::new(shop_zone));
        }

        // buildings
        let base_playzone_position = Vector::new(60.0 * UI_UNIT, PLAYER_BOARD_FROM_TOP);

        let build_zone = CardZone::<CardIcon>::from_container(&gameplay_state.get_board().buildings,
                                                              base_playzone_position,
                                                              ZoneDirection::Vertical,
                                                              BASE_Z_INDEX,
                                                              &assets,
                                                              &handlers);
        widgets.push(Box::new(build_zone));

        // kaiju_zone
        let kaiju_position = base_playzone_position + Vector::new(UI_UNIT * 9.0, 0); // 7U widget height + 1U padding + 1U gap
        let kaiju_zone = CardZone::<CardIcon>::from_container(&gameplay_state.get_board().kaiju_zone,
                                                              kaiju_position,
                                                              ZoneDirection::Vertical,
                                                              BASE_Z_INDEX,
                                                              &assets,
                                                              &handlers);
        widgets.push(Box::new(kaiju_zone));

        let base_numbers_position = Vector::new(4.0 * UI_UNIT, PLAYER_BOARD_FROM_TOP + 12.0 * UI_UNIT);

        for (num, currency) in Globals::in_game().iter().enumerate() {
            let value = gameplay_state.get_board().globals.get(*currency);
            widgets.push(Box::new(Button::new(
                format!("{:?}\n {}", currency, value),
                base_numbers_position + Vector::new(UI_UNIT * 5.0, 0) * num as f32,
                BASE_Z_INDEX,
                &assets,
                None,
            )));
        }

        Box::new(Self {
            widgets,
            window_w,
            window_h
        })
    }

    pub fn handle_io(&mut self, event: Event) -> Option<GameEvent> {
        match event {
            // TODO: generalize to arbitrary window sizes
            Event::MouseMoved(position) => {
                for w in &mut self.widgets {
                    w.update_hovered(position);
                }
                None
            }
            Event::MouseButton(MouseButton::Left, ButtonState::Released) => {
                self.widgets.iter()
                    .map(|widg| { widg.maybe_activate() }) // translate to events (maybe all None)
                    .find(|event| { event.is_some() }) // maybe find first Some
                    .map(|some_event| { some_event.unwrap() }) // if some, unwrap
            }
            _ => None
        }
    }

    pub fn update(&mut self) {
        ()
    }

    // TODO: make widgets draw to Surface, and arrange the Surfaces
    pub fn draw(&self, window: &mut Window) -> () {
        let horizontal_divider = Line::new(
            Vector::new(0, PLAYER_BOARD_FROM_TOP),
            Vector::new(self.window_w, PLAYER_BOARD_FROM_TOP),
        );
        window.draw(&horizontal_divider, Col(Color::from_rgba(100, 100, 100, 1.0)));

        for widget in &self.widgets {
            widget.draw(window).unwrap();
        }
    }
}

// This is only a placeholder, to allow us to take() ourselves from &mut Self
impl Default for BoardDisplay {
    fn default() -> Self {
        Self {
            widgets: Vec::new(),
            window_w: 0.0,
            window_h: 0.0
        }
    }
}

