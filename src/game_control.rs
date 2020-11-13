use crate::automaton::*;
use crate::game_logic::{BoardState, GameplayState};
use crate::ui::game_end_state::GameEndState;
use crate::loading::Assets;
use std::collections::HashMap;
use std::mem::take;
use std::hash::{Hash, Hasher};
use serde_derive::*;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum PlayerControl {
    Human,
    AI
}

impl Default for PlayerControl {
    fn default() -> Self {
        PlayerControl::Human
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Player {
   pub name: String,
   pub starting_deck: String,
   pub starting_buildings: String,
   pub control: PlayerControl,
   //opponent: Option<&Player>

   #[serde(skip)]
   pub opponent_idx: usize
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Hash for Player {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Eq for Player {}

#[derive(Debug, Default)]
pub struct GameControlState {
    player_boards: Vec<BoardState>,
    current_player_idx: usize,
    round: i16,
    assets: Assets
}

impl GameControlState {
    pub fn new(player_boards: Vec<BoardState>, assets: Assets) -> Self {
        Self { player_boards: player_boards,
            current_player_idx: 0,
            round: 0,
            assets: assets }
    }

    pub fn overtake(&mut self) -> Box<dyn AutomatonState> {
        self.start_player_turn(0)
    }

    pub fn get_board(&self, idx: usize) -> &BoardState {
        &self.player_boards[idx]
    }

    pub fn get_board_mut(&mut self, idx: usize) -> &mut BoardState {
        &mut self.player_boards[idx]
    }

    pub fn get_assets(&self) -> &Assets {
        &self.assets
    }

    pub fn start_player_turn(&mut self, idx: usize) -> Box<dyn AutomatonState> {
        let opponent_idx = self.player_boards[idx].player.opponent_idx;
        GameplayState::new_with_ui(Box::new(take(self)), idx, opponent_idx)
        //self.player_boards[idx].player.opponent_idx)
    }
}

impl AutomatonState for GameControlState {
    fn event(&mut self, event: GameEvent) -> Box<dyn AutomatonState> {
        println!("GameControlState received event: {:?}", event);

        match event {
            GameEvent::EndTurn => {
                let game_ended = self.player_boards.iter()
                    .filter(|b| !b.is_defeated())
                    .count() == 1;
                
                if game_ended {
                    let me = take(self);
                    GameEndState::new(me.player_boards, me.assets)
                } else {
                    self.current_player_idx += 1;

                    if self.current_player_idx >= self.player_boards.len() {
                        self.round += 1;
                        self.current_player_idx = 0;
                        // handle end of round here, if it ever mattered
                    }
                    
                    //let board = &mut self.player_boards[self.current_player_idx];
                    self.start_player_turn(self.current_player_idx)
                }
            },
            _ => {
                panic!("This state can't handle event {:?}", event)
            }
        }
    }

    fn update(&mut self) -> Box<dyn AutomatonState> {
        Box::new(take(self))
    }
}
