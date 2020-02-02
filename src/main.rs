#![windows_subsystem = "windows"]
extern crate quicksilver;
mod automaton;
mod gameobjects;
mod logic;
mod ui;

mod game_logic;

use quicksilver::prelude::*;
use quicksilver::graphics::View;
use std::process::exit;
use game_logic as gl;

use automaton::Automaton;
use ui::LoadingState;

struct Game {
    automaton: Automaton,
}
 
impl State for Game {
    fn new() -> Result<Game> {
        let mut loading = LoadingState::new();
        let game = Game {
            automaton: Automaton::new(loading)
        };
        Ok(game)
    }

    fn event(&mut self, event: &Event, _window: &mut Window) -> Result<()> {
        let run_out = self.automaton.event(event);
        if run_out {
            exit(0)
        }
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