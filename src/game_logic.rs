extern crate quicksilver;
extern crate json;

use serde_derive::*;

use crate::automaton::*;
use crate::ui::TakeTurnState;
use crate::game_objects::*;
use std::mem::take;

#[derive(Serialize, Deserialize, Debug, Clone)]
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
}

impl Default for BoardState {
    fn default() -> Self {
        Self::setup(None)
    }
}

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
        let mut taken_self = take(self);
        println!("GameState received event: {:?}", event);

        match event {
            GameEvent::CardPicked(card) => {
                taken_self.board.play_card(card);
                TakeTurnState::new(Box::new(taken_self))
            } 
            //GameEvent::CardTargeted => (StateAction::None, None),
            GameEvent::EndTurn => {
                taken_self.board.end_turn();
                taken_self.board.begin_turn();
                TakeTurnState::new(Box::new(taken_self))
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
