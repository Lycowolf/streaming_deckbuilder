/// Prototype of a new game stack automaton
///
/// The problem with the past design was that we passed references (in Events) out of our states, to be passed to
/// another states by the Automaton. This was problematic, because Automaton was 'static (it was required
/// by Quicksilver), and therefore any reference it stored in its stack must have been 'static too
/// (at least I don't know any way to explain to Rust the fact that referenced value did not leave an Automaton's stack
/// before it was consumed).
///
/// We fix this by having the states to construct and *own* another states, making the stack implicit.
/// Events can then be passed directly by calling self.previous_state.event(). Event won't return the stack operation:
/// it will return the new state, by calling event() on the stored state.
/// This will somewhat increase the coupling between states: UI will e. g. have to know
/// that Game can return cards to it, but might be unavoidable anyway.
///
/// We still want to model states as trait objects, to make possible to separate them to their source files.
/// We would like to have their event() method to have this signature:
///
/// fn event(self, event: Event) -> Box<State>
///
/// i. e. it *consumes* the old state and returns a new one. Sadly, this is impossible: to move a value to the method
/// (to be consumed), it must be Sized, but the point of a trait object is that we don't know its type, and therefore
/// we don't know its size either.
/// We do the next best thing: we pass a mutable reference, and then take() from it, leaving a dummy object behind,
/// thus satisfying the borrow checker. We then transform and pass the taken value as needed, because we own it.
/// To do this, the states must be Default. It is also slightly dangerous: a bug might have us use the dummy value.
/// Rust will not catch this: take() is specifically designed to work around the borrow checker.
///
/// TODO: it might be possible to implement the State trait on Box<SpecificState>,
///     enabling the consumption without take()s

use std::collections::hash_set::HashSet;
use std::mem::take;
use std::fmt::Debug;

#[derive(Eq, PartialEq, Debug, Hash, Clone)]
struct Card {
    name: Box<String>
}

enum Event<'a> {
    Action,
    CardPicked(&'a Card),
}

trait State: Debug {
    fn periodic_update(&mut self) -> Box<dyn State>;
    fn process_event(&mut self, event: Event) -> Box<dyn State>;
    fn draw(&self);
}

#[derive(Default, Debug)]
struct Loading {
    loaded_data: Option<Vec<i32>>,
}

impl Loading {
    fn new() -> Box<Self> {
        println!("Loading started");
        Box::new(Self {
            loaded_data: None
        })
    }
}

impl State for Loading {
    fn periodic_update(&mut self) -> Box<dyn State> {
        let mut taken_self = take(self);
        match taken_self.loaded_data {
            None => {
                println!("Waiting");
                taken_self.loaded_data = Some(vec![1, 2, 3]);
                Box::new(taken_self)
            }
            Some(data) => {
                println!("Loading done");
                Game::wrapped_in_ui(data)
            }
        }
    }

    fn process_event(&mut self, event: Event) -> Box<dyn State> {
        Box::new(take(self))
    }

    fn draw(&self) {
        println!("Loading is: {:?}", self)
    }
}

#[derive(Default, Debug)]
struct Game {
    data: Vec<i32>,
    board: Vec<Card>,
}

impl Game {
    fn new(data: Vec<i32>) -> Box<Self> {
        println!("game started");
        let mut board = Vec::new();
        board.push(Card { name: Box::new(String::from("x")) });
        board.push(Card { name: Box::new(String::from("y")) });
        board.push(Card { name: Box::new(String::from("z")) });
        Box::new(Self { data, board })
    }

    fn wrapped_in_ui(data: Vec<i32>) -> Box<UI> {
        UI::new(Self::new(data))
    }

    fn get_cards(&self) -> &Vec<Card> {
        &self.board
    }
}

impl State for Game {
    fn periodic_update(&mut self) -> Box<dyn State> {
        Box::new(take(self))
    }

    fn process_event(&mut self, event: Event) -> Box<dyn State> {
        let mut taken_self = take(self);
        match event {
            Event::CardPicked(card) => {
                println!("Card picked: {:?}", card);
                match taken_self.board.iter().enumerate().find(|(i, c)| { *c == card }) {
                    Some((index, card)) => {
                        println!("Played a card: {:?}", card);
                        taken_self.board.remove(index);
                        println!("My board is: {:?}", taken_self.board)
                    }
                    None => {
                        println!("This card is not in my hand");
                    }
                }
            }
            _ => {}
        }
        UI::new(Box::new(taken_self))
    }

    fn draw(&self) {
        println!("Game is: {:?}", self)
    }
}

#[derive(Debug, Default)]
struct UI {
    gameplay_state: Box<Game>,
}

impl UI {
    fn new(gameplay_state: Box<Game>) -> Box<Self> {
        println!("UI created");
        Box::new(Self { gameplay_state })
    }
}

impl State for UI {
    fn periodic_update(&mut self) -> Box<dyn State> {
        Box::new(take(self))
    }

    fn process_event(&mut self, event: Event) -> Box<dyn State> {
        let mut taken_self = take(self);
        match event {
            Event::Action => {
                let card = taken_self.gameplay_state.get_cards().iter().next();
                match card {
                    None => panic!("No card"),
                    Some(card) => {
                        let picked_card = (*card).clone();
                        let mut generic_state = taken_self.gameplay_state as Box<dyn State>;
                        // returns only generic Box<dyn State> => we can't do anything game-related with it
                        // luckily, we just want to return it
                        generic_state.process_event(Event::CardPicked(&picked_card))
                    }
                }
            }
            _ => Box::new(take(self))
        }
    }

    fn draw(&self) {
        println!("UI is: {:?}", self)
    }
}

fn main() {
    let mut state: Box<dyn State> = Loading::new();
    state.draw();
    state = state.periodic_update();
    state.draw();
    state = state.periodic_update();
    state.draw();
    state = state.process_event(Event::Action);
    state.draw();
    state = state.process_event(Event::Action);
    state.draw();
    state = state.process_event(Event::Action);
    state.draw();
}
