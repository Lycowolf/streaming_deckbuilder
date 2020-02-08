extern crate quicksilver;
extern crate json;

use serde_derive::*;

use crate::automaton::*;
use crate::ui::TakeTurnState;
use crate::game_objects::*;

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

     fn play_card(&mut self, card: Card) {
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
        self.globals.report();
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
