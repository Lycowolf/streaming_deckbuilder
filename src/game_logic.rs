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
    pub name : String,
    on_play : Vec<Effect>,
    cost: i16,
    cost_currency: String
}

type CardFactory = HashMap<String, Card>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Hand {
    pub size : usize,
    pub cards : Vec<Card>
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
    cards : VecDeque<Card>
}

impl Deck {
    fn new() -> Box<Self> {
        Box::new(Self {cards: VecDeque::new() } )
    }

    fn from_json(deck_node: serde_json::value::Value, card_factory: &CardFactory) -> Box<Self> {
        let data : HashMap<String, u16> =  serde_json::from_value(deck_node)
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
    fn load_deck(filename : &str, node_name : &str) -> Result<Box<Deck>> {
        
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

    fn add(&mut self, new_card : Card) {
        self.cards.push_back(new_card)
    }

    fn shuffle(&self) {
        unimplemented!
        ()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NumberMap {
    changed : HashMap<String, i16>
}

impl NumberMap {
    fn new() -> Box<Self> {
        Box::<NumberMap>::new(Self { changed : HashMap::<String, i16>::new()})
    }

    fn get(&self, key : &str) -> i16 {
        match self.changed.get(key) {
            Some(val) => *val,
            None => 0
        }
    }

    fn add(&mut self, key : &str, change : i16) {
        let val = self.changed.entry(key.to_string()).or_insert(0);
        *val += change;
    }

    fn reset(&mut self, key : &str) {
        self.changed.remove(key);
    }

    fn reset_all(&mut self) {
        self.changed.clear();
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum StoreType {
    Fixed{items: Vec<String>},
    Drafted{size: i8, from_deck: String}
}

#[derive(Debug, Deserialize)]
pub struct Store {
    pub store_type: StoreType,
    pub menu: Vec<Card>
}

impl Store {
    fn from_json(json: serde_json::value::Value) -> Store {
        serde_json::from_value(json).expect("Malformed store description")
    }

    fn populate(&mut self, card_factory: CardFactory) {
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
                let mut deck = Deck::load_deck("filename", &from_deck).unwrap();

                for _ in 0..*size {
                    let card = deck.draw();
                    match deck.draw() {
                        Some(card) => self.menu.push(card),
                        None => ()
                    }
                }
            }
        }

    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BoardState {
    pub hand : Box<Hand>,
    deck : Box<Deck>,
    pub globals : Box<NumberMap>,
    turn : u16
}

impl BoardState {
    pub fn new() -> Box<Self> {
        Box::<Self>::new(Self::setup(None))
    }

    pub fn setup(deck_node : Option<&str>) -> BoardState {
        let hand_size = 5;

        let hand = Hand { size : hand_size, cards : Vec::<Card>::with_capacity(hand_size)};
        let deck = match deck_node {
            Some(node) => Deck::load_deck("cards.json", node).expect("No deck loaded"),
            None => Deck::new()
        };

        BoardState {
            turn : 1,
            hand : Box::new(hand),
            deck : deck,
            globals: NumberMap::new()
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
        let board : &mut BoardState = board_state.as_mut().unwrap();

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
