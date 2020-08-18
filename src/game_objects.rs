extern crate quicksilver;
extern crate json;

use quicksilver::prelude::*;
use std::collections::VecDeque;
use std::collections::HashMap;
use serde_derive::*;
use itertools::Itertools;
use itertools::izip;
use std::iter;
use std::fmt;
use crate::game_logic::BoardState;

pub struct GameData {
    board_state: BoardState,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "effect")]
pub enum Effect {
    Echo{msg: String},
    Global{key: Globals, val: i16},
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Globals {
    Attack,
    Build,
    Evil
}

impl Globals {
    pub fn in_game() -> Vec<Self> {
        vec!(Globals::Build, Globals::Attack, Globals::Evil)
    }
}

impl fmt::Display for Globals {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // We know that debug formatting is OK
        <Self as fmt::Debug>::fmt(self, f)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cost {
    count: i16,
    currency: Globals
}

impl Default for Cost {
    fn default() -> Self {
        Cost{ count: 0, currency: Globals::Build }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoardZone {
    None,
    Hand,
    Buildings,
    Kaiju,
    BuildStore,
    KaijuStore
}

impl Default for BoardZone {
    fn default() -> BoardZone {
        BoardZone::None
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
    pub cost: Cost,
    pub draw_to: DrawTo,
    pub available: bool,
    pub image: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CardContainer {
    pub zone: BoardZone,
    pub cards: Vec<Card>,
    pub size: Option<usize>
}

impl CardContainer {
    pub fn new(zone: BoardZone) -> Self {
        Self { zone: zone, cards: Vec::<Card>::new(), size: None }
    }

    pub fn new_sized(zone: BoardZone, size: usize) -> Self {
        Self { zone: zone, cards: Vec::<Card>::with_capacity(size), size: Some(size) }

    }

    pub fn add(&mut self, card: Card) {
        self.cards.push(card)
    }

    pub fn empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn is_full(&self) -> bool {
        match self.size {
            Some(size) =>  self.cards.len() == size,
            None => false
        }
        
    }

    pub fn get(&self, idx: usize) -> Card {
        self.cards[idx].clone()
    }

    fn remove(&mut self, card_idx: usize) -> Card {
        self.cards.remove(card_idx)
    }

    /// Extract effects linked to speciffied event for each card in the container (effects can repeat). 
    /// Join these lists together, tagging every effect with the card that causes it and zone the card belongs to.
    ///
    /// Expected use: to evaluate events (on_turn_start effects etc.)
    pub fn all_effects(&self, event_selector: fn(&Card)-> &Vec<Effect>) -> Vec<(BoardZone, Card, Effect)> {

        self.cards.iter()
                .flat_map(|c| izip!(iter::repeat(self.zone), 
                                    iter::repeat(c).cloned(),
                                    event_selector(c).iter().cloned()) )
                .collect()
    }
}

impl PartialEq for CardContainer {
    fn eq(&self, other: &Self) -> bool {
        self.zone == other.zone
    }
}
impl Eq for CardContainer {}


#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Deck {
    cards: VecDeque<Card>
}

impl Deck {
    pub fn new() -> Self {
        Self{ cards: VecDeque::new() }
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

impl From<Vec<Card>> for Deck {
    fn from(source: Vec<Card>) -> Self {
        Deck{ cards: VecDeque::<Card>::from(source) }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct NumberMap {
    changed: HashMap<Globals, i16>
}

impl NumberMap {
    pub fn new() -> Box<Self> {
        Box::<NumberMap>::new(Self { changed: HashMap::<Globals, i16>::new()})
    }

    pub fn get(&self, key: Globals) -> i16 {
        match self.changed.get(&key) {
            Some(val) => *val,
            None => 0
        }
    }

    pub fn add(&mut self, key: Globals, change: i16) {
        let val = self.changed.entry(key).or_insert(0);
        *val += change;
    }

    pub fn pay(&mut self, cost: &Cost) {
        let val = self.changed.entry(cost.currency.clone()).or_insert(0);
        *val -= cost.count;
    }

    pub fn can_afford(&self, cost: &Cost) -> bool {
        match self.changed.get(&cost.currency) {
            Some(val) => val >= &cost.count,
            None => cost.count <= 0
        }
    }

    pub fn reset(&mut self, key: Globals) {
        self.changed.remove(&key);
    }

    pub fn reset_all(&mut self) {
        self.changed.clear();
    }

    // FIXME: either implement other iter methods, or convert this into some less dynamic type and drop this method
    pub fn iter(&self) -> std::collections::hash_map::Iter<Globals, i16> {
        self.changed.iter()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum StoreType {
    Fixed{items: Vec<String>},
    Drafted{size: usize, from_deck: String}
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

// FIXME: stringly typed values are bad in Rust: make this a trait & implement it with multiple structs, or make this an enum
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Store {
    pub store_type: StoreType,
    pub menu: CardContainer,
    pub deck: Option<Box<Deck>>
}

impl Store {

    pub fn buy_card(&mut self, card_idx: usize) -> Card {
        let card = self.menu.get(card_idx);

        if let StoreType::Drafted{size: _, from_deck: _} = self.store_type {
            self.menu.remove(card_idx);
            self.refill();
        }

        card

    }

    pub fn refill(&mut self) {
        if let Some(newcard) = self.deck.as_mut().unwrap().draw() {
            self.menu.add(newcard);
        }
    }
}
