extern crate quicksilver;
extern crate json;

use quicksilver::prelude::*;
use std::collections::VecDeque;
use std::collections::HashMap;
use serde_derive::*;
use itertools::Itertools; 

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "effect")]
pub enum Effect {
    Echo{msg: String},
    Global{key: String, val: i16},
    Return,
    None
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card {
    pub name : String,
    pub on_play : Vec<Effect>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Deck {
    cards : VecDeque<Card>
}

impl Deck {
    pub fn new() -> Box<Self> {
        Box::new(Self {cards: VecDeque::new() } )
    }

    // FIXME: load as asset
    pub fn load_deck(filename : &str, node_name : &str) -> Result<Box<Deck>> {
        
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
            for _ in 0..*num {
                if factory.contains_key(key) {
                    let card = factory.get(key).unwrap().clone();
                    new_deck.add(card);
                }
            }
        };

        Ok(new_deck)
    }

    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop_front()
    }

    pub fn add(&mut self, new_card : Card) {
        self.cards.push_back(new_card)
    }

    fn shuffle(&self) {
        unimplemented!
        ()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NumberMap {
    changed : HashMap<String, i16>
}

impl NumberMap {
    pub fn new() -> Box<Self> {
        Box::<NumberMap>::new(Self { changed : HashMap::<String, i16>::new()})
    }

    pub fn get(&self, key : &str) -> i16 {
        match self.changed.get(key) {
            Some(val) => *val,
            None => 0
        }
    }

    pub fn add(&mut self, key : &str, change : i16) {
        let val = self.changed.entry(key.to_string()).or_insert(0);
        *val += change;
    }

    pub fn reset(&mut self, key : &str) {
        self.changed.remove(key);
    }

    pub fn reset_all(&mut self) {
        self.changed.clear();
    }

    pub fn report(&self) {
        println!("Game state:");
        for (key, val) in self.changed.iter().sorted() {
            println!(" {}: {}", key, val);
        }
    }
}
