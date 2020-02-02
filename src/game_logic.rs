
extern crate quicksilver;
extern crate json;

use quicksilver::prelude::*;
use std::collections::VecDeque;
use std::collections::HashMap;
use serde_derive::*;
use itertools::Itertools; 

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "effect")]
enum Effect {
    Echo{msg: String},
    Global{key: String, val: i16},
    Return,
    None
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Card {
    name : String,
    on_play : Vec<Effect>
}

#[derive(Serialize, Deserialize, Debug)]
struct Hand {
    size : usize,
    cards : Vec<Card>
}

impl Hand {
    fn is_full(&self) -> bool {
        self.cards.len() == self.size
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
        let factory : HashMap<String, Card> = serde_json::from_value(card_node)
                .expect("Malformed card list");

        let deck_node  = { json.get(node_name)
                            .expect(format!("file should have \"{}\" node", node_name).as_str())
                            .clone()
                        };
        let data : HashMap<String, u16> =  serde_json::from_value(deck_node)
                .expect("Malformed deck list");

        let mut new_deck = Deck::new();
        for (key, num) in data.iter() {
            for _ in 1..*num {
                if factory.contains_key(key) {
                    let card = factory.get(key).unwrap().clone();
                    new_deck.add(card);
                }
            }
        };

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
struct NumberMap {
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

#[derive(Serialize, Deserialize, Debug)]
pub struct GameState {
    hand : Box<Hand>,
    deck : Box<Deck>,
    globals : Box<NumberMap>,
    turn : u16
}

impl GameState {
    pub fn new() -> Box<Self> {
        Box::<Self>::new(Self::setup(None))
    }

    pub fn setup(deck_node : Option<&str>) -> GameState {
        let hand_size = 5;

        let hand = Hand { size : hand_size, cards : Vec::<Card>::with_capacity(hand_size)};
        let deck = match deck_node {
            Some(node) => Deck::load_deck("cards.json", node).expect("No deck loaded"),
            None => Deck::new()
        };

        GameState {
            turn : 1,
            hand : Box::new(hand),
            deck : deck,
            globals: NumberMap::new()
        }
    }

    pub fn play_card(&mut self, idx : usize) {
        let card = self.hand.cards.remove(idx);

        println!("Played card {}", card.name);

        let mut returning = false;

        for effect in &card.on_play {
            match effect {
                Effect::Echo{msg} => println!("  {}", msg),
                Effect::Global{key, val} => self.globals.add(&key, *val),
                Effect::None => println!("  It does nothing"),
                Effect::Return => { returning = true; }
            }
        }

        if returning {
            self.deck.add(card)
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
