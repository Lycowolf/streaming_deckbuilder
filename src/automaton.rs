// TODO:
// allow update to return a new state (for start-of-turn animations, time-limited actions etc.)
// (See also notes in logic::TurnState.)
// To do this, we want to be able to return a new event, just like the event() does,
// but we should probably defer this to event processing?
// If true, then we will have to implement our own event queue.

/// Push-down automaton powering our game

use quicksilver::lifecycle::{Event, Window};
use quicksilver::graphics::Color;
use quicksilver::Result;

// TODO: more reasonable way to identify cards in play
pub type HandIndex = u8;
pub type BoardPosition = u8;

pub enum GameEvent {
    Started,
    CardPicked(HandIndex),
    CardTargeted(BoardPosition),
    IO(Event), // keyboard, mouse etc.
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
    fn event(&self, event: GameEvent) -> ProcessingResult;

    /// This is called periodically, probably every frame. Used for timers, UI animations etc.
    /// By default does nothing.
    // TODO: pass elapsed time?
    fn update(&mut self) -> () {}

    /// Called every frame (if possible). It should draw only on the provided z-index.
    /// It is a good idea to draw into texture and cache the result for performance.
    /// Screen is blanked before stack is drawn.
    /// By default does nothing.
    fn draw(&self, window: &mut Window, z_index: u32) -> () {}
}

pub struct Automaton {
    stack: Box<Vec<Box<dyn AutomatonState>>>
}

/// returns true if automaton's stack has been emptied
impl Automaton {
    pub fn new(starting_state: Box<dyn AutomatonState>) -> Self {
        Self {stack: Box::new(vec![starting_state])}
    }

    pub fn event(&mut self, event: &quicksilver::lifecycle::Event) -> bool {
        let mut game_event = GameEvent::wrap_io(*event);

        loop {
            let stack_top = self.stack.last();
            println!("Stack top: {:?}", stack_top);
            if stack_top.is_none() { return true }

            let current_state = stack_top.unwrap();
            let (state_op, new_event) = current_state.event(game_event);

            match state_op {
                StateAction::None => break,
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

            if new_event.is_none() {
                break
            } else {
                game_event = new_event.unwrap();
            }
        }
        false
    }

    /// call periodically so current state can emasure elapsed time
    /// does nothing when there's no current state
    // TODO: maybe should panic when no state?
    pub fn update(&mut self) {
        let stack_top = self.stack.last_mut();
        if stack_top.is_none() { return }

        let current_state = stack_top.unwrap();
        current_state.update();
    }

    pub fn draw(&self, window: &mut Window) -> Result<()> {
        window.clear(Color::BLACK)?; // TODO: maybe make clearing screen the caller's responsibility?
        for (z, state) in self.stack.iter().enumerate() {
            state.draw(window, z as u32);
        }
        Ok(())
    }
}