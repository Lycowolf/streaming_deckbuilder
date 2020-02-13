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
    ToBuildings,
    Break,
    None
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrawTo {
    Hand,
    Kaiju
}

impl Default for DrawTo {
    fn default() -> DrawTo {
        DrawTo::Hand
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Card {
    pub name: String,
    pub on_play: Vec<Effect>,
    pub on_turn_begin: Vec<Effect>,
    pub on_turn_end: Vec<Effect>,
    pub on_strike: Vec<Effect>,
    pub on_defend: Vec<Effect>,
    pub cost: i16,
    pub cost_currency: String,
    pub draw_to: DrawTo
}

pub type CardFactory = HashMap<String, Card>;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Deck {
    cards: VecDeque<Card>
}

impl Deck {
    pub fn new() -> Box<Self> {
        Box::new(Self {cards: VecDeque::new() } )
    }

    pub fn from_json(deck_node: serde_json::value::Value, card_factory: &CardFactory) -> Box<Self> {
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
    pub fn load_deck(filename: &str, node_name: &str) -> Result<Box<Deck>> {
        
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

    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop_front()
    }

    pub fn add(&mut self, new_card: Card) {
        self.cards.push_back(new_card)
    }

    pub fn shuffle(&self) {
        unimplemented!
        ()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct NumberMap {
    changed: HashMap<String, i16>
}

impl NumberMap {
    pub fn new() -> Box<Self> {
        Box::<NumberMap>::new(Self { changed: HashMap::<String, i16>::new()})
    }

    pub fn get(&self, key: &str) -> i16 {
        match self.changed.get(key) {
            Some(val) => *val,
            None => 0
        }
    }

    pub fn add(&mut self, key: &str, change: i16) {
        let val = self.changed.entry(key.to_string()).or_insert(0);
        *val += change;
    }

    pub fn reset(&mut self, key: &str) {
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum StoreType {
    Fixed{items: Vec<String>},
    Drafted{size: i8, from_deck: String}
}

impl Default for StoreType {
    fn default() -> StoreType {
        StoreType::Fixed{items: Vec::<String>::new()}
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Buildings {
    pub list: Vec<Card>
}

impl Buildings {
    pub fn add(&mut self, card: Card) {
        self.list.push(card)
    }

    pub fn empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn break_one(&mut self) {
        self.list.remove(0);
    }

    pub fn from_jsom(json: &serde_json::value::Value, node: &str, factory: &CardFactory) -> Self {
        let source_node = json.get(node).expect(format!("store node {} not found", node).as_str()).clone();
        let data: HashMap<String, u16> =  serde_json::from_value(source_node)
                                                .expect("Malformed starting building list");

        let mut buildings = Buildings{ list: Vec::<Card>::new() };
        for (key, num) in data.iter() {
            for _ in 0..*num {
                if factory.contains_key(key) {
                    let card = factory.get(key).unwrap().clone();
                    buildings.add(card);
                }
            }
        };

        buildings
    }
}


#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Store {
    pub name: String,
    pub store_type: StoreType,
    pub menu: Vec<Card>,
    deck: Option<Box<Deck>>
}

impl Store {
    pub fn from_json(json: &serde_json::value::Value, node: &str, factory: &CardFactory) -> Store {
        let source_node = json.get(node).expect(format!("store node {} not found", node).as_str()).clone();

        let store_type : StoreType = serde_json::from_value(source_node).expect("Malformed store description");

        let mut store = Store { name: node.to_string(),
                                store_type: store_type,
                                menu: Vec::<Card>::new(),
                                deck: None };

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

    pub fn buy_card(&mut self, card_idx: usize) -> Card {
        let card = self.menu.get(card_idx).expect("Bought card not in store") .clone();

        if let StoreType::Drafted{size: _, from_deck: _} = self.store_type {
            self.menu.remove(card_idx);
            self.refill();
        }

        card

    }

    pub fn refill(&mut self) {
        if let Some(newcard) = self.deck.as_mut().unwrap().draw() {
            self.menu.push(newcard);
        }
    }
}