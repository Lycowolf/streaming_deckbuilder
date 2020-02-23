/// Push-down automaton powering our game

use quicksilver::lifecycle::{Event, Window};
use quicksilver::graphics::Color;
use quicksilver::Result;
use std::process::exit;
use crate::game_objects::BoardZone;

// NOTE: we identify cards by their index in the relevant vector (hand, board, ...). We can't pass reference because
//  the structure it resides in gets stored in / returned to different places than the event, causing problems
//  with lifetimes.
#[derive(Debug, Clone)]
pub enum GameEvent {
    Started, // usually passed to new states to run their logic immediately
    CardPicked(usize),
    CardTargeted(usize),
    CardBought(BoardZone, usize),
    EndTurn, 
    IO(Event), // keyboard, mouse etc.
    Timeout,
    GameEnded,
}

impl GameEvent {
    fn wrap_io(event: Event) -> Self {
        GameEvent::IO(event)
    }
}

#[derive(Debug, Default)]
pub struct GameEndedState;

impl AutomatonState for GameEndedState {
    fn event(&mut self, event: GameEvent) -> Box<dyn AutomatonState> {
        exit(0)
    }

    fn update(&mut self) -> Box<dyn AutomatonState> {
        exit(0)
    }
}

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
    fn event(&mut self, event: GameEvent) -> Box<dyn AutomatonState>;

    /// This is called periodically, probably every frame. Used for timers, UI animations etc.
    /// By default does nothing.
    // TODO: pass elapsed time?
    /// The default noop implementation is:
    ///
    /// Box::new(take(self))
    ///
    /// We can't do it here, because we are in a trait object (that requires !Sized) and take() requires Sized
    /// but specific states can be Sized and may implement this without any problem
    fn update(&mut self) -> Box<dyn AutomatonState>;

    /// Called every frame (if possible). It should draw only on the provided z-index.
    /// It is a good idea to draw into texture and cache the result for performance.
    /// Screen is blanked before stack is drawn.
    /// By default does nothing.
    fn draw(&self, window: &mut Window) -> () {}
}

pub struct Automaton {
    state: Box<dyn AutomatonState>,
}

impl Automaton {
    pub fn new(starting_state: Box<dyn AutomatonState>) -> Self {
        Self {
            state: starting_state,
        }
    }

    pub fn event(&mut self, event: &quicksilver::lifecycle::Event) {
        self.state = self.state.event(GameEvent::wrap_io(*event));
    }

    /// call periodically so current state can measure elapsed time
    pub fn update(&mut self) {
        self.state = self.state.update();
    }

    pub fn draw(&self, window: &mut Window) -> Result<()> {
        window.clear(Color::BLACK)?; // TODO: maybe make clearing screen the caller's responsibility?
        self.state.draw(window);
        Ok(())
    }
}