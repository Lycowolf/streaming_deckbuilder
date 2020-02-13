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
    pub hand: Box<Hand>,
    deck: Box<Deck>,
    pub globals: Box<NumberMap>,
    turn: u16,
    store_fixed: Box<Store>,
    store_trade: Box<Store>,
    buildings: Box<Buildings>,
    kaiju_zone: Box<Vec<Card>>
}

impl BoardState {
    /*
    pub fn new() -> Box<Self> {
        Box::<Self>::new(Self::setup(None))
    }
    */

    pub fn load_board(filename: &str) -> BoardState {
        
        let deck_node_name = "test_deck";
        let buildings_node = "starter_buildings";
        let store_node = "build_store";
        let trade_row = "kaiju_store";
        let hand_size = 5;


        let file = load_file(filename)
            .wait()
            .expect("file should open read only");
        let json: serde_json::Value = serde_json::from_slice(file.as_slice())
            .expect("file should be proper JSON");
        
        let card_node = { json.get("cards")
                            .expect("file should have \"cards\" node")
                            .clone()
                        };
        let factory: CardFactory = serde_json::from_value(card_node)
                .expect("Malformed card list");

        let deck_node  = { json.get(deck_node_name)
                            .expect(format!("file should have \"{}\" node", deck_node_name).as_str())
                            .clone()
                        };

        let draw_deck = Deck::from_json(deck_node, &factory);

        //let bs_node = { json.get("build_store").expect("build_store node not found").clone() };
        let build_store = Store::from_json(&json, store_node, &factory);

        //let ks_node = { json.get("kaiju_store").expect("kaiju_store node not found").clone() };
        let kaiju_store = Store::from_json(&json, trade_row, &factory);

        let hand = Hand { size: hand_size, cards: Vec::<Card>::with_capacity(hand_size)};

        let buildings = Buildings::from_jsom(&json, buildings_node, &factory);

        print!("Loading done");

        BoardState {
            turn: 1,
            hand: Box::new(hand),
            deck: draw_deck,
            globals: NumberMap::new(),
            store_fixed: Box::new(build_store),
            store_trade: Box::new(kaiju_store),
            buildings: Box::new(buildings),
            kaiju_zone: Box::new(Vec::<Card>::new())
        }
    }

     fn play_card(&mut self, card: usize) {
        if !(0..self.hand.cards.len()).contains(&card) {
            panic!("WTF? Playing card not in hand? I should play card #{:?} when my gameplay state is: {:?}", card, self);
        }
        let played = self.hand.cards.remove(card);

        println!("Played card {}", played.name);

        let mut returning = false;

        for effect in &played.on_play {
            self.evaluate_effect(effect, &played)
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
                        self.kaiju_zone.push(card);
                    }
                };
                
                true},
        }
    }

    pub fn begin_turn(&mut self) {
        println!("Starting turn {}", self.turn);

        // process on_begin
        for building in self.buildings.list.iter() {
            for eff in building.on_turn_end.iter() {
                //self.evaluate_effect(eff, building)
            }
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

        for kaiju in self.kaiju_zone.iter() {
            for eff in kaiju.on_strike.iter() {
                //self.evaluate_effect(eff, kaiju)
            }
        }

        for building in self.buildings.list.iter() {
            for eff in building.on_turn_end.iter() {
                //self.evaluate_effect(eff, building)
            }
        }
        
        self.globals.reset_all();

        // increase turn counter
        self.turn += 1;
    }

    pub fn evaluate_effect(&mut self, effect: &Effect, card: &Card) {
        match effect {
            Effect::Echo{msg} => println!("  {}", msg),
            Effect::Global{key, val} => self.globals.add(&key, *val),
            Effect::None => println!("  It does nothing"),
            Effect::Return => { self.deck.add(card.clone()) },
            Effect::ToBuildings => { self.buildings.add(card.clone()) },
            Effect::Break => { self.buildings.break_one() }
        }
    }

    pub fn report_hand(&self) {
        println!("In hand, I have:");
        for card in self.hand.cards.iter() {
            println!(" - {}", card.name)
        }
        println!();
    }

    pub fn report(&self) {
        self.globals.report();
        println!();
        self.report_hand();
    }

    pub fn store_by_name(&mut self, name: &str) -> &mut Store {
        if name == self.store_fixed.name {
            self.store_fixed.as_mut()
        } else if name == self.store_trade.name {
            self.store_trade.as_mut()
        } else {
            panic!("Buy in unknown store")
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
}

impl AutomatonState for GameplayState {
    fn event(&mut self, event: GameEvent) -> Box<dyn AutomatonState> {
        println!("GameState received event: {:?}", event);

        match event {
            GameEvent::CardPicked(card) => {
                self.board.play_card(card);
                TakeTurnState::new(Box::new(take(self)))
            } 
            //GameEvent::CardTargeted => (StateAction::None, None),
            GameEvent::CardBought(store_name, card_idx) => {

                let store = self.board.store_by_name(&store_name);
                let card = store.buy_card(card_idx);

                self.board.globals.add(&card.cost_currency, -card.cost);
                self.board.deck.add(card.clone());

                TakeTurnState::new(Box::new(take(self)))
            }
            GameEvent::EndTurn => {
                self.board.end_turn();
                self.board.begin_turn();
                TakeTurnState::new(Box::new(take(self)))
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
