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
    store_trade: Box<Store>
}

impl BoardState {
    /*
    pub fn new() -> Box<Self> {
        Box::<Self>::new(Self::setup(None))
    }
    */

    pub fn load_board(filename: &str) -> BoardState {
        
        let deck_node_name = "test_deck";
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

        print!("Loading done");

        BoardState {
            turn: 1,
            hand: Box::new(hand),
            deck: draw_deck,
            globals: NumberMap::new(),
            store_fixed: Box::new(build_store),
            store_trade: Box::new(kaiju_store)
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
            match effect {
                Effect::Echo{msg} => println!("  {}", msg),
                Effect::Global{key, val} => self.globals.add(&key, *val),
                Effect::None => println!("  It does nothing"),
                Effect::Return => { returning = true; }
            }
        }

        if returning {
            self.deck.add(played)
        }
    }

    pub fn draw_card(&mut self) -> bool {
        if self.hand.is_full() {
            return false
        }

        match self.deck.draw() {
            Some(card) => {
                println!("Drawn card: {}", card.name);
                self.hand.cards.push(card);
                true
                },
            None => false
        }
    }

    pub fn begin_turn(&mut self) {
        println!("Starting turn {}", self.turn);
        // process on_begin

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

        // proc on_end

        // increase turn counter
        self.turn += 1;
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

    fn find_index_in_store(&self, store: &Store, card: &Card) -> Option<usize> {
        store.menu.iter().position(|c| card.eq(c))
    }

    fn find_card_in_stores(&mut self, card: &Card) -> (&mut Store, usize) {
        match self.find_index_in_store(&self.store_fixed, &card) {
            Some(idx) => { return (self.store_fixed.as_mut(), idx); },
            None => ()
        };

        match self.find_index_in_store(&self.store_trade, &card) {
            Some(idx) => { return (self.store_trade.as_mut(), idx); },
            None => ()
        };

        panic!("Card not found in stores;")
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
            GameEvent::CardBought(card_idx) => {
                let store = self.board.store_fixed.as_mut();

                if let Some(card) = store.menu.get(card_idx) {
                    self.board.globals.add(&card.cost_currency, -card.cost);
                    self.board.deck.add(card.clone());

                    if let StoreType::Drafted{size: _, from_deck: _} = store.store_type {
                        store.menu.remove(card_idx);
                        store.refill();
                    }
                }

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
