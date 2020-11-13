use crate::automaton::*;
use crate::game_objects::*;
use crate::game_logic::BoardState;
use serde_derive::*;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AI {

}

impl AI {
    pub fn new() -> Box<Self> {
        Box::new(Self{})
    }

    pub fn select_card(&self, board: &BoardState) -> GameEvent {
        GameEvent::CardPicked(0)
        //GameEvent::CardBought(0)
    } 

    pub fn target_card(&self, board: &BoardState, card_idx: usize, card_target: BoardZone) -> GameEvent {
        // TODO not to be used until card_targetting is merged. Then, fix this

        let target = board.kaiju_zone.cards.iter()
            .enumerate()
            .filter_map(|(i, c)| match c.available {
                true => Some(i),
                false => None
            })
            .last();
        
        match target {
            Some(idx) => GameEvent::CardTargeted(BoardZone::Hand, card_idx, card_target, idx),
            None => GameEvent::CardTargeted(BoardZone::None, 0, BoardZone::None, 0) // nothing to target
        }
    }
}