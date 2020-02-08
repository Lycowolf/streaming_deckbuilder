/// Push-down automaton powering our game

use quicksilver::lifecycle::{Event, Window};
use quicksilver::graphics::Color;
use quicksilver::Result;
use std::collections::VecDeque;
use crate::game_logic;

// TODO: more reasonable way to identify cards in play
pub type HandIndex = u8;
pub type BoardPosition = u8;

#[derive(Debug)]
pub enum GameEvent {
    Started, // usually passed to new states to run their logic immediately
    CardPicked(game_logic::Card),
    CardTargeted(BoardPosition),
    EndTurn, 
    IO(Event), // keyboard, mouse etc.
    Timeout,
    GameEnded
}

impl GameEvent {
    fn wrap_io(event: Event) -> Self {
        GameEvent::IO(event)
    }
}

#[derive(Debug)]
pub enum StateAction {
    Push(Box<dyn AutomatonState>),
    Pop,
    Replace(Box<dyn AutomatonState>),
    None, // do nothing; typically used in states that wait for user input to signalize we are still waiting
}

impl StateAction {
    fn is_none(&self) -> bool {
        match self {
            StateAction::None => true,
            _ => false
        }
    }

    fn is_some(&self) -> bool {
        !self.is_none()
    }
}

pub type ProcessingResult = (StateAction, Option<GameEvent>);

pub trait AutomatonState: std::fmt::Debug {
    /// Handle event, return a state transition and possibly a new event to process with it.
    /// Returns:
    ///
    /// StateAction:
    /// None: this signals that we don't want to handle the incoming event and it should be discarded.
    ///     Returned event is ignored.
    /// Push, Pop and Replace: do the specified action with the state stack
    ///
    /// GameEvent:
    /// None signals that there's nothing more to do (at least for now) and we should wait for another event / update
    /// in the (new) top state on the stack.
    /// When state change is specified, this event will be passed to the new state immediately.
    /// If state wishes the incoming event should be reprocessed in the new state, it should pass it back here.
    fn event(&mut self, board_state: &mut Option<game_logic::BoardState>, event: GameEvent) -> ProcessingResult;

    /// This is called periodically, probably every frame. Used for timers, UI animations etc.
    /// By default does nothing.
    // TODO: pass elapsed time?
    fn update(&mut self, board_state: &mut Option<game_logic::BoardState>) -> Option<GameEvent> { None }

    /// Called every frame (if possible). It should draw only on the provided z-index.
    /// It is a good idea to draw into texture and cache the result for performance.
    /// Screen is blanked before stack is drawn.
    /// By default does nothing.
    fn draw(&self, board_state: &Option<game_logic::BoardState>, window: &mut Window, z_index: u32) -> () {}
}

pub struct Automaton {
    stack: Box<Vec<Box<dyn AutomatonState>>>,
    event_queue: Box<VecDeque<GameEvent>>,
    board_state: Option<game_logic::BoardState>
}

impl Automaton {
    pub fn new(starting_state: Box<dyn AutomatonState>) -> Self {
        Self {
            stack: Box::new(vec![starting_state]),
            event_queue: Box::new(VecDeque::new()),
            board_state: None
        }
    }

    /// returns true if automaton's stack has been emptied
    pub fn event(&mut self, event: &quicksilver::lifecycle::Event) -> bool {
        self.event_queue.push_back(GameEvent::wrap_io(*event));

        self.process_events()
    }

    /// call periodically so current state can measure elapsed time
    /// returns true if automaton's stack has been emptied
    // TODO: maybe should panic when no state?
    pub fn update(&mut self) -> bool {
        let stack_top = self.stack.last_mut();
        if stack_top.is_none() { return true }

        let current_state = stack_top.unwrap();
        if let Some(event) = current_state.update(&mut self.board_state) {
            self.event_queue.push_back(event);
        }

        self.process_events()
    }

    pub fn draw(&self, window: &mut Window) -> Result<()> {
        window.clear(Color::BLACK)?; // TODO: maybe make clearing screen the caller's responsibility?
        for (z, state) in self.stack.iter().enumerate() {
            state.draw(&self.board_state, window, z as u32);
        }
        Ok(())
    }

    fn process_events(&mut self) -> bool {
        loop {
            let next_event = self.event_queue.pop_front();
            let game_event = match next_event {
                None => break,
                Some(event) => event
            };

            let stack_top= self.stack.last_mut();
            println!("Stack top: {:?}", stack_top);
            if stack_top.is_none() { return true }

            let current_state: &mut Box<dyn AutomatonState> = stack_top.unwrap();
            let (state_op, new_event) = current_state.event(&mut self.board_state, game_event);

            if new_event.is_some() {
                self.event_queue.push_back(new_event.unwrap())
            }

            match state_op {
                StateAction::None => {
                    // we might not be done: state may send an event to itself
                    continue
                },
                StateAction::Replace(new_state) => {
                    self.stack.pop();
                    self.stack.push(new_state);
                },
                StateAction::Pop => {
                    self.stack.pop();
                },
                StateAction::Push(new_state) => {
                    self.stack.push(new_state)
                },
            }
        }
        false
    }
}