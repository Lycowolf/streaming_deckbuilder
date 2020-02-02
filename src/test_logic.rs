
mod game_logic;

use game_logic as gl;


fn main() {
    //let mut game = gl::GameState::setup(Some("starter_deck"));
    let mut game = gl::GameState::setup(Some("test_deck"));

    game.begin_turn();
    game.report();
    game.play_card(0);
    game.play_card(0);
    game.play_card(0);
    game.end_turn();

    game.begin_turn();
    game.report();
    game.play_card(2);
    game.play_card(2);
    game.end_turn();

    game.report();
}