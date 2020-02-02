
mod game_logic;
mod automaton;
mod ui;


use game_logic as gl;


fn main() {
    //let mut game = gl::GameState::setup(Some("starter_deck"));
    let mut game = gl::BoardState::setup(Some("test_deck"));

    game.begin_turn();
    game.report();
    game.play_card(game.hand.get(0));
    game.play_card(game.hand.get(0));
    game.play_card(game.hand.get(0));
    game.end_turn();

    game.begin_turn();
    game.report();
    game.play_card(game.hand.get(2));
    game.play_card(game.hand.get(2));
    game.end_turn();

    game.report();
}