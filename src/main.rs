#![windows_subsystem = "windows"]
extern crate quicksilver;

mod game_logic;

use quicksilver::prelude::*;
use quicksilver::graphics::View;
use std::process::exit;
use game_logic as gl;

// TODO: more reasonable way to identify cards in play
type HandIndex = u8;
type BoardPosition = u8;

struct Card<'a> {
    name: &'a str,
    effect: fn(&mut Game) -> (),
}

enum GameEvent {
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
enum StateAction {
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

type ProcessingResult = (StateAction, Option<GameEvent>);

trait AutomatonState: std::fmt::Debug {
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

//////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct LoadingState;

impl LoadingState {
    fn new() -> Box<Self> {
        Box::new(Self)
    }
}

impl AutomatonState for LoadingState {
    fn event(&self, event: GameEvent) -> ProcessingResult {
        match event {
            GameEvent::IO(Event::Key(Key::Return, ButtonState::Pressed)) => {
                let new_game = GameplayState::new();
                (StateAction::Replace(new_game), Some(GameEvent::Started))
            },
            _ => (StateAction::None, None)
        }
    }

    fn draw(&self, window: &mut Window, z_index: u32) -> () {
        window.draw_ex(&Circle::new((300, 300), 32), Col(Color::BLUE), Transform::IDENTITY, z_index);
    }
}

#[derive(Debug)]
struct GameplayState {
    timer: i32,
    game : Box<game_logic::GameState>
}

impl GameplayState {
    fn new() -> Box<Self> {

        let mut game = gl::GameState::new();
        game.begin_turn();
        game.report_hand();

        Box::new(Self {
            timer: 0,
            game: game
        })
    }
}

impl AutomatonState for GameplayState {


    fn event(&self, event: GameEvent) -> ProcessingResult {
        match event {
            GameEvent::IO(Event::Key(Key::Escape, ButtonState::Pressed)) => {
                (StateAction::Pop, None)
            },
            _ => (StateAction::None, None)
        }
    }

    fn update(&mut self) -> () {
        self.timer += 1;
    }

    fn draw(&self, window: &mut Window, z_index: u32) -> () {
        let rectangle = Rectangle::new((300, 300), (32, 32));
        window.draw_ex(&rectangle, Col(Color::RED), Transform::rotate(self.timer), z_index);
    }
}

//////////////////////////////////////////////////////////////////////////////////////////

struct Game {
    state_stack: Box<Vec<Box<dyn AutomatonState>>>,
}
 
impl State for Game {
    fn new() -> Result<Game> {
        let mut loading = vec![LoadingState::new() as Box<dyn AutomatonState>];
        let game = Game {
            state_stack: Box::new(loading)
        };
        Ok(game)
    }

    fn event(&mut self, event: &Event, _window: &mut Window) -> Result<()> {
        let mut game_event = GameEvent::wrap_io(*event);

        loop {
            let stack_top = self.state_stack.last();
            println!("Stack top: {:?}", stack_top);
            if stack_top.is_none() { exit(0) }

            let current_state = stack_top.unwrap();
            let (state_op, new_event) = current_state.event(game_event);

            match state_op {
                StateAction::None => break,
                StateAction::Replace(new_state) => {
                    self.state_stack.pop();
                    self.state_stack.push(new_state);
                },
                StateAction::Pop => {
                    self.state_stack.pop();
                },
                StateAction::Push(new_state) => {
                    self.state_stack.push(new_state)
                },
            }

            if new_event.is_none() {
                break
            } else {
                game_event = new_event.unwrap();
            }
        }
        Ok(())
    }

    fn update(&mut self, window: &mut Window) -> Result<()> {
        let stack_top = self.state_stack.last_mut();
        if stack_top.is_none() { exit(0) }

        let current_state = stack_top.unwrap();
        current_state.update();

        Ok(())
    }
 
    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::BLACK)?;
        for (z, state) in self.state_stack.iter().enumerate() {
            state.draw(window, z as u32);
        }
        Ok(())
    }
}

fn test_game_logic() {
    let mut game = gl::GameState::setup();
    game.begin_turn();
    game.report_hand();
    game.play_card(0);
    game.play_card(0);
    game.play_card(0);
    game.end_turn();

    game.begin_turn();
    game.report_hand();
    game.play_card(2);
    game.play_card(2);
    game.end_turn();
}

fn main() {
    run::<Game>("Draw Geometry", Vector::new(800, 600), Settings::default());
    //test_game_logic();
}