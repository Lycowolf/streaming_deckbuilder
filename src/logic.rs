use crate::automaton::*;
use crate::gameobjects::*;

type CardPile = Vec<Card>;

#[derive(Debug)]
pub struct TurnState {
    draw_pile: CardPile,
    discard_pile: CardPile,
    hand: CardPile,
}

impl TurnState {
    fn new_game_state() -> TurnState {
        let mut workers = CardPile::new();
        for i in 1..=10 {
            workers.push(Card::new(format!("Worker #{}", i)));
        }
        let kaijus = CardPile::new();
        for i in 1..=5 {
            workers.push(Card::new(format!("'Zilla' #{}", i)));
        }
        Self {
            draw_pile: workers,
            discard_pile: CardPile::new(),
            hand: kaijus,
        }
    }
}

impl AutomatonState for TurnState {
    fn event(&self, event: GameEvent) -> ProcessingResult {
        // TODO:
        // we want to start processing game logic right away, not waiting for events (except the ones we ask the UI for).
        // Modify automaton to always send a StateEntered event when stack changes?
        // Or we might to allow update() to return a new event (that would be probably good for timers etc. anyway).
        (StateAction::None, None)
    }
}
