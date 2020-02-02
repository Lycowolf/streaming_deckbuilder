
extern crate quicksilver;
extern crate json;

use quicksilver::prelude::*;
use std::collections::VecDeque;
use std::collections::HashMap;
use serde_derive::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Effect {
    command : String
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Card {
    name : String,
    on_play : Option<Effect>
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
pub struct GameState {
    hand : Box<Hand>,
    deck : Box<Deck>,
    turn : u16
}

impl GameState {
    pub fn new() -> Box<GameState> {
        Box::<GameState>::new(GameState::setup())
    }

    pub fn setup() -> GameState {
        let hand_size = 5;

        let hand = Hand { size : hand_size, cards : Vec::<Card>::with_capacity(hand_size)};

        GameState {
            turn : 1,
            hand : Box::new(hand),
            deck : Deck::load_deck("cards.json", "starter_deck").expect("No deck loaded")}
    }

    pub fn play_card(&mut self, idx : usize) {
        let card = self.hand.cards.remove(idx);

        println!("Played card {}", card.name);

        match card.on_play {
            Some(effect) => println!("  {}", effect.command),
            None => println!("It does nothing")
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
            println!("{}", card.name)
        }
        println!();
    }
}
