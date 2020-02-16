//#![windows_subsystem = "windows"]
extern crate quicksilver;
mod automaton;
mod ui;
mod game_objects;
mod game_logic;

use quicksilver::prelude::*;

use automaton::Automaton;
use ui::LoadingState;
use ui::{WINDOW_SIZE_H, WINDOW_SIZE_W};

struct Game {
    automaton: Automaton,
}
 
impl State for Game {
    fn new() -> Result<Game> {
        let loading = LoadingState::new();
        let game = Game {
            automaton: Automaton::new(loading)
        };
        Ok(game)
    }

    fn event(&mut self, event: &Event, _window: &mut Window) -> Result<()> {
        self.automaton.event(event);
        Ok(())
    }

    fn update(&mut self, window: &mut Window) -> Result<()> {
        self.automaton.update();
        Ok(())
    }
 
    fn draw(&mut self, window: &mut Window) -> Result<()> {
        self.automaton.draw(window)
    }
}

fn main() {
    run::<Game>("Draw Geometry", Vector::new(WINDOW_SIZE_W, WINDOW_SIZE_H), Settings::default());
}