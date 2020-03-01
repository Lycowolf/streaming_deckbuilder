extern crate quicksilver;
extern crate json;

use serde_derive::*;
use quicksilver::prelude::*;

use crate::automaton::*;
use crate::ui::TakeTurnState;
use crate::game_objects::*;
use std::mem::take;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BoardState {
    pub hand: Box<CardContainer>,
    pub deck: Box<Deck>,
    // FIXME: Use something that have set of keys known beforehand; consider using RON (Rusty Object Notation) instead of JSON
    //  for its support of enums
    pub globals: Box<NumberMap>,
    pub turn: u16,
    pub stores: Box<Vec<Store>>,
    pub buildings: Box<CardContainer>, // FIXME: make this a vector, or a type that can be iterated
    pub kaiju_zone: Box<CardContainer>
}

impl BoardState {
    /*
    pub fn new() -> Box<Self> {
        Box::<Self>::new(Self::setup(None))
    }
    */

     fn play_card(&mut self, card: usize) {
        if !(0..self.hand.cards.len()).contains(&card) {
            panic!("WTF? Playing card not in hand? I should play card #{:?} when my gameplay state is: {:?}", card, self);
        }
        let played = self.hand.cards.remove(card);

        println!("Played card {}", played.name);

        let mut returning = false;

        for effect in &played.on_play {
            self.evaluate_effect(effect, played.clone())
        }
    }

    pub fn draw_card(&mut self) -> bool {
        if self.hand.is_full() {
            return false
        }

        match self.deck.draw() {
            None => false,
            Some(card) => {
                match card.draw_to {
                    DrawTo::Hand => {
                        println!("Drawn card: {}", card.name);
                        self.hand.cards.push(card);
                    },
                    DrawTo::Kaiju => {
                        println!("Raaar! Kaiju came: {}", card.name);
                        self.kaiju_zone.add(card);
                    }
                };
                
                true},
        }
    }

    pub fn begin_turn(&mut self) {
        println!("Starting turn {}", self.turn);

        // process on_begin
        for (_, card, effect) in self.buildings.all_effects(|c| &c.on_turn_begin) {
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
            self.evaluate_effect(&effect, card)
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
            Effect::Echo{msg} => println!("  {}", msg),
            Effect::Global{key, val} => self.globals.add(*key, *val),
            Effect::None => println!("  It does nothing"),
            Effect::Return => { self.deck.add(card) },
            Effect::ToBuildings => { self.buildings.add(card) },
            Effect::Break => { self.buildings.cards.remove(0); }
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
    }

}

// impl Default for BoardState {
//     fn default() -> Self {
//         print!("Default needed");
//         //Self::load_board("cards.json")
//     }
// }

#[derive(Debug, Default)]
pub struct GameplayState {
    board: BoardState
}

impl GameplayState {
    pub fn new(mut board: BoardState) -> Self {
        board.begin_turn();
        println!("Created a new board: {:?}", board);
        Self { board }
    }

    pub fn new_with_ui(mut board: BoardState) -> Box<TakeTurnState> {
        let gameplay_state = Box::new(Self::new(board));
        println!("Wrapping this gameplay state: {:?}", gameplay_state);
        TakeTurnState::new(gameplay_state)
    }

    pub fn get_board(&self) -> &BoardState {
        &self.board
    }

    // Performs all operations needed before switching to TakeTurnState
    fn take_turn(&mut self) -> Box<TakeTurnState> {
        self.board.update_availability();
        TakeTurnState::new(Box::new(take(self)))
    }
}

impl AutomatonState for GameplayState {
    fn event(&mut self, event: GameEvent) -> Box<dyn AutomatonState> {
        println!("GameplayState received event: {:?}", event);

        match event {
            GameEvent::CardPicked(card) => {
                self.board.play_card(card);
                self.take_turn()
            } 
            //GameEvent::CardTargeted => (StateAction::None, None),
            GameEvent::CardBought(zone, card_idx) => {

                // TODO extract as method to GameBoard? I may want to use it in computing is_available, too
                let can_afford = {
                    let store = self.board.store_by_zone(zone);
                    let card_cost = store.menu.cards[card_idx].cost.clone();
                    self.board.globals.can_afford(&card_cost)
                };

                if can_afford {
                    let store = self.board.store_by_zone(zone);
                    let card = store.buy_card(card_idx);

                    self.board.globals.pay(&card.cost);
                    self.board.deck.add(card.clone());
                } else {
                    println!("Cannot buy, relevant global value too low (i.e. you do not have enough cash)")
                }
                
                self.take_turn()
            }
            GameEvent::EndTurn => {
                self.board.end_turn();
                self.board.begin_turn();
                self.take_turn()
            },
            GameEvent::GameEnded => Box::new(GameEndedState{}),
            _ => {
                panic!("This state can't handle event {:?}", event)
            }
        }
    }

    fn update(&mut self) -> Box<dyn AutomatonState> {
        Box::new(take(self))
    }
}
