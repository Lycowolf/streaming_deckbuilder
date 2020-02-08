/// UI states for our game's push-down automaton

use crate::automaton::*;
use crate::game_logic::*;
use quicksilver::prelude::*;
use quicksilver::Future;
use serde::export::fmt::Debug;
use derivative::*;
use std::mem::take;

#[derive(Debug, Default)]
pub struct LoadingState {
    board_state: BoardState,
}

impl LoadingState {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            board_state: BoardState::load_board("cards.json"),
        })
    }
}

impl AutomatonState for LoadingState {
    fn event(&mut self, event: GameEvent) -> Box<dyn AutomatonState> {
        Box::new(take(self))
    }

    fn draw(&self, window: &mut Window) -> () {
        window.draw(&Circle::new((300, 300), 32), Col(Color::BLUE));
    }

    fn update(&mut self) -> Box<dyn AutomatonState> {
        // TODO async load
        GameplayState::new_with_ui(take(self).board_state)
    }
}

// TODO: cache widgets?
#[derive(Derivative)]
#[derivative(Debug)]
pub struct TakeTurnState {
    gameplay_state: Box<GameplayState>,
    widgets: Vec<Box<dyn Widget>>,
    #[derivative(Debug = "ignore")]
    font: Font,
}

// TODO: load fonts in LoadingState
impl TakeTurnState {
    pub fn new(gameplay_state: Box<GameplayState>) -> Box<Self> {
        let font = Font::load("Roboto-Italic.ttf").wait().expect("Can't load font file");

        let card_size = Vector::new(75f32, 120f32);

        let mut widgets = Vec::new();
        widgets.push(Box::new(CardWidget::new(
            &"Draw pile\n\nClick to\nend turn".to_string(),
            Vector::new(45, 390),
            card_size,
            &font,
            GameEvent::EndTurn,
        ),
        ) as Box<dyn Widget>);

        let hand = &gameplay_state.get_board().hand.cards;
        let offset_top_left = Vector::new(240.0, 465.0);
        let gap = Vector::new(30.0, 0);
        for (num, card) in hand.clone().drain(..).enumerate() {
            let name = card.name.clone();
            let action_text = format!("Card {} clicked", name);
            widgets.push(Box::new(CardWidget::new(
                &name,
                offset_top_left + ((card_size.x_comp() + gap) * num as f32),
                card_size,
                &font,
                GameEvent::CardPicked(num),
            ),
            ) as Box<dyn Widget>);
        }

        Box::new(Self {
            gameplay_state,
            widgets,
            font,
        })
    }
}

// This is only a placeholder, to allow us to take() ourself from &mut Self
impl Default for TakeTurnState {
    fn default() -> Self {
        *Self::new(Box::new(GameplayState::default()))
    }
}

impl AutomatonState for TakeTurnState {
    fn event(&mut self, event: GameEvent) -> Box<dyn AutomatonState> {
        match event {
            // TODO: generalize to arbitrary window sizes
            GameEvent::IO(Event::MouseMoved(position)) => {
                for mut w in &mut self.widgets {
                    w.update_hovered(position);
                }
                Box::new(take(self))
            }
            GameEvent::IO(Event::MouseButton(MouseButton::Left, ButtonState::Released)) => {
                let found = self.widgets.iter()
                    .map(|widg| { widg.maybe_activate() }) // translate to events (maybe all None)
                    .find(|event| { event.is_some() }) // maybe find first Some
                    .map(|some_event| { some_event.unwrap() }); // if some, unwrap
                match found {
                    Some(event) => self.gameplay_state.event(event),
                    None => Box::new(take(self))
                }
            }
            GameEvent::IO(Event::Key(Key::Escape, ButtonState::Released)) => {
                Box::new(GameEndedState {})
            }
            _ => Box::new(take(self))
        }
    }

    fn update(&mut self) -> Box<dyn AutomatonState> {
        Box::new(take(self))
    }

    // TODO: make widgets draw to Surface, and arrange the Surfaces
    fn draw(&self, window: &mut Window) -> () {
        for widget in &self.widgets {
            widget.draw(window).unwrap();
        }
    }
}

trait Widget: Debug {
    fn bounding_box(&self) -> Rectangle;
    fn maybe_activate(&self) -> Option<GameEvent>;
    fn draw(&self, window: &mut Window) -> Result<()>;
    fn update_hovered(&mut self, pointer_position: Vector);
}

#[derive(Derivative)]
#[derivative(Debug)]
struct CardWidget {
    name: Box<String>,
    area: Rectangle,
    #[derivative(Debug = "ignore")]
    on_action: GameEvent,
    hovered: bool,
    image: Image,
}

impl CardWidget {
    fn new(name: &String, top_left: Vector, card_size: Vector, font: &Font, on_action: GameEvent) -> Self {
        let area = Rectangle::new(top_left, card_size);
        Self {
            name: Box::new(name.to_string().clone()),
            area,
            on_action,
            hovered: false,
            image: font.render(format!("{}", name).as_str(), &FontStyle::new(12.0, Color::WHITE)).expect("Can't render text"),
        }
    }
}

impl Widget for CardWidget {
    fn bounding_box(&self) -> Rectangle {
        self.area
    }

    fn maybe_activate(&self) -> Option<GameEvent> {
        if self.hovered {
            Some(self.on_action.clone())
        } else {
            None
        }
    }

    fn draw(&self, window: &mut Window) -> Result<()> {
        let position = self.area.pos;
        let text_rect = self.image.area().translate(position);
        window.draw(&self.area, Col(Color::from_rgba(50, 50, 50, 1.0)));
        window.draw(&text_rect, Img(&self.image));
        Ok(())
    }

    fn update_hovered(&mut self, pointer_position: Vector) {
        self.hovered = self.area.contains(pointer_position);
    }
}