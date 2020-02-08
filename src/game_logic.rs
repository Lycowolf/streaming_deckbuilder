extern crate quicksilver;
extern crate json;

use quicksilver::prelude::*;
use std::collections::VecDeque;
use std::collections::HashMap;
use serde_derive::*;
use itertools::Itertools; 

use crate::automaton::*;
use crate::ui::TakeTurnState;
use crate::automaton::GameEvent::Started;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "effect")]
enum Effect {
    Echo{msg: String},
    Global{key: String, val: i16},
    Return,
    None
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card {
    pub name: String,
    on_play: Vec<Effect>,
    cost: i16,
    cost_currency: String
}

type CardFactory = HashMap<String, Card>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Hand {
    pub size: usize,
    pub cards: Vec<Card>
}

impl Hand {
    pub fn is_full(&self) -> bool {
        self.cards.len() == self.size
    }

    pub fn get(&self, idx: usize) -> Card {
        self.cards[idx].clone()
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Deck {
    cards: VecDeque<Card>
}

impl Deck {
    fn new() -> Box<Self> {
        Box::new(Self {cards: VecDeque::new() } )
    }

    fn from_json(deck_node: serde_json::value::Value, card_factory: &CardFactory) -> Box<Self> {
        let data: HashMap<String, u16> =  serde_json::from_value(deck_node)
                                                .expect("Malformed deck list");

        let mut new_deck = Deck::new();
        for (key, num) in data.iter() {
            for _ in 0..*num {
                if card_factory.contains_key(key) {
                    let card = card_factory.get(key).unwrap().clone();
                    new_deck.add(card);
                }
            }
        };

        new_deck
    }

    // FIXME: load as asset
    fn load_deck(filename: &str, node_name: &str) -> Result<Box<Deck>> {
        
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

        let deck_node  = { json.get(node_name)
                            .expect(format!("file should have \"{}\" node", node_name).as_str())
                            .clone()
                        };

        let new_deck = Deck::from_json(deck_node, &factory);

        Ok(new_deck)
    }

    fn draw(&mut self) -> Option<Card> {
        self.cards.pop_front()
    }

    fn add(&mut self, new_card: Card) {
        self.cards.push_back(new_card)
    }

    fn shuffle(&self) {
        unimplemented!
        ()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NumberMap {
    changed: HashMap<String, i16>
}

impl NumberMap {
    fn new() -> Box<Self> {
        Box::<NumberMap>::new(Self { changed: HashMap::<String, i16>::new()})
    }

    fn get(&self, key: &str) -> i16 {
        match self.changed.get(key) {
            Some(val) => *val,
            None => 0
        }
    }

    fn add(&mut self, key: &str, change: i16) {
        let val = self.changed.entry(key.to_string()).or_insert(0);
        *val += change;
    }

    fn reset(&mut self, key: &str) {
        self.changed.remove(key);
    }

    fn reset_all(&mut self) {
        self.changed.clear();
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StoreType {
    Fixed{items: Vec<String>},
    Drafted{size: i8, from_deck: String}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Store {
    pub store_type: StoreType,
    pub menu: Vec<Card>,
    deck: Option<Box<Deck>>
}

impl Store {
/*
    fn from_json(json: serde_json::value::Value, factory: &CardFactory) -> Store {
        let store_type : StoreType = serde_json::from_value(json).expect("Malformed store description");

        let mut store = Store { store_type: store_type, menu: Vec::<Card>::new() };
        store.populate(factory);

        store
    }
*/
    fn from_json(json: &serde_json::value::Value, node: &str, factory: &CardFactory) -> Store {
        let source_node = json.get(node).expect(format!("store node {} not found", node).as_str()).clone();

        let store_type : StoreType = serde_json::from_value(source_node).expect("Malformed store description");

        let mut store = Store { store_type: store_type, menu: Vec::<Card>::new(), deck: None };
        store.populate(factory, &json);

        store
    }

    fn populate(&mut self, card_factory: &CardFactory, json: &serde_json::value::Value) {
        self.menu.clear();

        match &self.store_type {
            StoreType::Fixed{items} => {
                for i in items {
                    if card_factory.contains_key(i) {
                        self.menu.push(card_factory.get(i).unwrap().clone())
                    }
                }
            },
            StoreType::Drafted{size, from_deck} => {
                let deck_node = json.get(from_deck)
                    .expect(format!("deck node {} not found", from_deck).as_str())
                    .clone();
                let mut deck = Deck::from_json(deck_node, card_factory);

                for _ in 0..*size {
                    match deck.draw() {
                        Some(card) => self.menu.push(card),
                        None => ()
                    }
                }

                self.deck = Some(deck);
            }
        }

    }
}

#[derive(Serialize, Deserialize, Debug)]
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
        let trade_row = "kaiju_trade";
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

    pub fn play_card(&mut self, card: Card) {
        let idx = self.hand.cards.iter().position(|c| card.eq(c) )
            .expect("WTF? PLaying card not in hand?");
        let played = self.hand.cards.remove(idx);

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
        println!("Game state:");
        for (key, val) in self.globals.changed.iter().sorted() {
            println!(" {}: {}", key, val);
        }
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

#[derive(Debug)]
pub struct GameplayState;

impl GameplayState {
    pub fn new() -> Self {
        Self
    }
}

impl AutomatonState for GameplayState {
    fn event(&mut self, board_state: &mut Option<BoardState>, event: GameEvent) -> ProcessingResult {
        // TODO:
        // we want to start processing game logic right away, not waiting for events (except the ones we ask the UI for).
        // Modify automaton to always send a StateEntered event when stack changes?
        // Or we might to allow update() to return a new event (that would be probably good for timers etc. anyway).
        println!("GameState received event: {:?}", event);
        let ui = TakeTurnState::new();
        let board: &mut BoardState = board_state.as_mut().unwrap();

        match event {
            GameEvent::Started => {
                board.begin_turn();
                (StateAction::Push(ui), Some(GameEvent::Started))
            } 
            GameEvent::CardPicked(card) => {
                board.play_card(card);
                (StateAction::Push(ui), Some(GameEvent::Started))
            } 
            //GameEvent::CardTargeted => (StateAction::None, None),
            GameEvent::CardBought(card) => {
                board.globals.add(&card.cost_currency, -card.cost);
               
                // TODO replace when we have references
                {
                    let source = board.find_card_in_stores(&card);
                    let store = source.0;
                    let idx = source.1;

                    if let StoreType::Drafted{size: _, from_deck: _} = store.store_type {
                        store.menu.remove(idx);
                        if let Some(newcard) = store.deck.as_mut().unwrap().draw() {
                            store.menu.push(newcard);
                        }
                    }
                }

                board.deck.add(card);

                (StateAction::Push(ui), Some(GameEvent::Started))
            }
            GameEvent::EndTurn => {
                board.end_turn();
                board.begin_turn();
                (StateAction::Push(ui), Some(GameEvent::Started))
            },
            GameEvent::GameEnded => (StateAction::Pop, None),
            _ => {
                println!("Passing processing to UI");
                
                (StateAction::Push(ui), None)
            }
        }
    }
}
