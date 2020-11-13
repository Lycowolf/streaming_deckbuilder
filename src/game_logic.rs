extern crate quicksilver;
extern crate json;

use serde_derive::*;
use quicksilver::prelude::*;

use crate::automaton::*;
use crate::ui::{TakeTurnState, TargetingState};
use crate::game_objects::*;
use crate::loading::Assets;
use crate::game_control::{Player, PlayerControl, GameControlState};
use crate::ai::AI;
use std::mem::take;
use quicksilver::graphics::PixelFormat;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BoardState {
    pub player: Player,
    pub hand: Box<CardContainer>,
    pub deck: Box<Deck>,
    // FIXME: Use something that have set of keys known beforehand; consider using RON (Rusty Object Notation) instead of JSON
    //  for its support of enums
    pub globals: Box<NumberMap>,
    pub turn: u16,
    pub stores: Box<Vec<Store>>,
    pub buildings: Box<CardContainer>,
    // FIXME: make this a vector, or a type that can be iterated
    pub kaiju_zone: Box<CardContainer>,
    pub ai: Option<Box<AI>>
}

impl BoardState {
    /*
    pub fn new() -> Box<Self> {
        Box::<Self>::new(Self::setup(None))
    }
    */

     fn play_card(&mut self, card: usize) -> Card {
        let played = self.hand.remove(card)
                        .expect(format!("WTF? Playing card not in hand? I should play card #{:?} when my gameplay state is: {:?}", card, self).as_str());

        println!("Played card {}", played.name);

        for effect in &played.on_play {
            self.evaluate_effect(effect, played.clone())
        }

        played
    }

    fn play_card_on_target(&mut self, card_idx: usize, target_zone: BoardZone, target_idx: usize) {
        let played = self.play_card(card_idx);
        let target_container = self.container_by_zone(target_zone);

        println!("   on target {}", target_idx);

        match played.target_effect {
            TargetEffect::None => { print!("*Sad trombone*"); },
            TargetEffect::Stun => { target_container.get(card_idx).stunned = true; }
            TargetEffect::Kill => { target_container.remove(target_idx); },
            TargetEffect::Bounce => {
                if let Some(target) = target_container.remove(target_idx) {
                    self.deck.add(target);
                }
            },
        }
    }

    pub fn draw_card(&mut self) -> bool {
        if self.hand.is_full() {
            return false;
        }

        match self.deck.draw() {
            None => false,
            Some(card) => {
                self.container_by_zone(card.draw_to).add(card);
                
                true},
        }
    }

    pub fn begin_turn(&mut self) {
        println!("Starting turn {}", self.turn);

        //for zone in [self.hand, self.kaiju_zone, self.buildings, self.store_fixed, self.store_trade]:
        // 
        for mut card in self.kaiju_zone.cards.clone() {
            card.reset()
        }


        // process on_begin
        for (_, card, effect) in self.buildings.all_effects(|c| &c.on_turn_start) {
            self.evaluate_effect(&effect, card)
        }

        // draw full hand
        while !self.hand.is_full() {
            if !self.draw_card() {
                break;
            }
        }
    }

    pub fn end_turn(&mut self) {
        println!("Ending turn {}", self.turn);
        println!();

        for (_, card, effect) in self.kaiju_zone.all_effects(|c| &c.on_strike) {
            if !card.stunned {
                self.evaluate_effect(&effect, card)
            }
        }

        for (_, card, effect) in self.buildings.all_effects(|c| &c.on_turn_end) {
            self.evaluate_effect(&effect, card)
        }

        self.globals.reset_all();

        // increase turn counter
        self.turn += 1;
    }

    pub fn evaluate_effect(&mut self, effect: &Effect, card: Card) {
        match effect {
            Effect::Break => {
                let cost = Cost{currency: Globals::Block, count: 1};
                if self.globals.can_afford(&cost) {
                    self.globals.pay(&cost)
                } else if self.buildings.cards.len() > 0 {
                    self.buildings.remove(0);
                }},
            Effect::BreakEverything => {
                while !self.buildings.empty() {
                    self.buildings.remove(0);
                }},
            Effect::BreakUnblockable => {self.buildings.remove(0);},
            Effect::Echo{msg} => println!("  {}", msg),
            Effect::Global{key, val} => self.globals.add(*key, *val),
            Effect::None => println!("  It does nothing"),
            Effect::Return => { self.deck.add(card) },
            Effect::ToBuildings => { self.buildings.add(card) },
        }
    }

    pub fn store_by_zone(&mut self, zone: BoardZone) -> &mut Store {
        self.stores.iter_mut()
            .find(|s| s.menu.zone == zone)
            .expect("Buy in unknown store")
    }

    pub fn update_availability(&mut self) {
        for store in self.stores.iter_mut() {
            for card in store.menu.cards.iter_mut() {
                card.available = self.globals.can_afford(&card.cost);
            }
        }

        for container in vec!(self.hand.as_mut(), self.buildings.as_mut(), self.kaiju_zone.as_mut()) {
            for card in container.cards.iter_mut() {
                card.available = true;
            }
        }
    }

    pub fn container_by_zone(&mut self, zone: BoardZone) -> &mut CardContainer {
        match zone {
            BoardZone::Buildings => self.buildings.as_mut(),
            BoardZone::Hand => self.hand.as_mut(),
            BoardZone::Kaiju => self.kaiju_zone.as_mut(),
            BoardZone::BuildStore => &mut self.store_by_zone(zone).menu,
            BoardZone::KaijuStore => &mut self.store_by_zone(zone).menu,
            BoardZone::None => { panic!("Do not access None zone.") }
        }
    }

    pub fn is_defeated(&self) -> bool {
        self.buildings.empty()
    }
}

#[derive(Debug, Default)]
pub struct GameplayState {
    controller: Box<GameControlState>,
    // board: BoardState,
    // opponent_board: BoardState
    board_idx: usize,
    opo_idx: usize
}

impl GameplayState {
    pub fn new(controller: Box<GameControlState>, board_idx: usize, opo_idx: usize) -> Box<Self> { //board: BoardState, opponent_board: BoardState) -> Box<Self> {
        //Box::new(Self{controller, board, opponent_board })
        Box::new(Self{controller, board_idx, opo_idx })
    }

    pub fn new_with_ui(controller: Box<GameControlState>, board_idx: usize, opo_idx: usize) -> Box<dyn AutomatonState> {
       let mut gameplay_state = Box::new(Self::new(controller, board_idx, opo_idx));
       println!("Wrapping this gameplay state: {:?}", gameplay_state);
       gameplay_state.event(GameEvent::StartTurn)
    }

    pub fn get_board(&self) -> &BoardState {
        //&self.board
        &self.controller.get_board(self.board_idx)
    }

    pub fn get_board_mut(&mut self) -> &mut BoardState {
        //&self.board
        self.controller.get_board_mut(self.board_idx)
    }

    pub fn get_opponent(&self) -> &BoardState {
        //&self.opponent_board
        &self.controller.get_board(self.opo_idx)
    }

    pub fn get_opponent_mut(&mut self) -> &mut BoardState {
        //&self.opponent_board
        self.controller.get_board_mut(self.opo_idx)
    }

    pub fn get_assets(&self) -> &Assets {
        self.controller.get_assets()
    }

    // Performs all operations needed before switching control
    // either to player by going to TakeTurnState,
    // or AI by calling self.event with event obtained from AI object
    fn take_turn(&mut self) -> Box<dyn AutomatonState> {
        self.get_board_mut().update_availability();

        match self.get_board().player.control {
            PlayerControl::Human => TakeTurnState::new(Box::new(take(self))),
            PlayerControl::AI => {
                let board = self.get_board();
                let ai = board.ai.as_ref().expect("AI for AI player not loaded");
                let intent = ai.select_card(board);
                self.event(intent)
            } 
        }
    }
}

impl AutomatonState for GameplayState {
    fn event(&mut self, event: GameEvent) -> Box<dyn AutomatonState> {
        println!("GameplayState received event: {:?}", event);

        match event {
            GameEvent::StartTurn => {
                self.get_board_mut().begin_turn();
                self.take_turn()
            }
            GameEvent::CardPicked(card_idx) => {

                // interception
                let tags = self.get_board().hand.cards[card_idx].tags.clone();
                let interference = self.get_board_mut().kaiju_zone.cards.iter_mut()
                    .filter_map(|k| if !k.stunned &&
                                       k.intercepts_left > 0 &&
                                       tags.contains(&k.intercept?.tag) {
                                            Some(k)
                                        } else {
                                            None
                                        })
                    .next();
                
                if let Some(mut annoyance) = interference {
                    annoyance.intercepts_left -= 1;
                    self.get_board_mut().hand.remove(card_idx);

                    return self.take_turn()
                }

                // play the card
                let card_target = self.get_board().hand.cards[card_idx].target_zone;
                match card_target {
                    BoardZone::None => {
                        self.get_board_mut().play_card(card_idx);
                        self.take_turn()
                    },
                    _ => {
                        self.get_board_mut().update_availability();
                        match self.get_board().player.control {
                            PlayerControl::Human => TargetingState::new(Box::new(take(self)), BoardZone::Hand, card_idx, card_target),
                            PlayerControl::AI => {
                                let board = self.get_board_mut();
                                let ai = board.ai.as_ref().expect("AI for AI player not loaded");
                                let intent = ai.target_card(board, card_idx, card_target);
                                self.event(intent)
                            } 
                        }
                        
                    }
                }
            }, 
            GameEvent::CardTargeted(card_zone, card_idx, target_zone, target_idx) => {
                if target_zone != BoardZone::None {
                    self.get_board_mut().play_card_on_target(card_idx, target_zone, target_idx);
                };
                self.take_turn()
            },
            GameEvent::CardBought(zone, card_idx) => {

                // TODO extract as method to GameBoard? I may want to use it in computing is_available, too
                let can_afford = {
                    let store = self.get_board_mut().store_by_zone(zone);
                    let card_cost = store.menu.cards[card_idx].cost.clone();
                    self.get_board().globals.can_afford(&card_cost)
                };

                if can_afford {
                    let store = self.get_board_mut().store_by_zone(zone);
                    let card = store.buy_card(card_idx);

                    self.get_board_mut().globals.pay(&card.cost);

                    if card.give_to_enemy {
                        self.get_opponent_mut().deck.add(card.clone());
                    } else {
                        self.get_board_mut().deck.add(card.clone());
                    }
                } else {
                    println!("Cannot buy, relevant global value too low (i.e. you do not have enough cash)")
                }

                self.take_turn()
            }
            GameEvent::EndTurn => {
                self.get_board_mut().end_turn();
                self.controller.event(GameEvent::EndTurn)
            }
            GameEvent::GameEnded => Box::new(GameEndedState {}),
            _ => {
                panic!("This state can't handle event {:?}", event)
            }
        }
    }

    fn update(&mut self) -> Box<dyn AutomatonState> {
        Box::new(take(self))
    }
}
