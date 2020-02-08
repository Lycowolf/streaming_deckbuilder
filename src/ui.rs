/// UI states for our game's push-down automaton

use crate::automaton::*;
use crate::game_logic::*;
use quicksilver::prelude::*;
use quicksilver::Future;
use serde::export::fmt::Debug;
use derivative::*;
use std::borrow::Borrow;
use itertools::Itertools;

#[derive(Debug)]
pub struct LoadingState {
    timer: u32
}

impl LoadingState {
    pub fn new() -> Box<Self> {
        Box::new(Self { timer: 1 })
    }
}

impl AutomatonState for LoadingState {
    fn event(&mut self, board_state: &mut Option<BoardState>, event: GameEvent) -> ProcessingResult {
        match event {
            GameEvent::IO(Event::Key(Key::Return, ButtonState::Pressed)) | GameEvent::Timeout => {
                let new_game = Box::new(GameplayState::new());
                (StateAction::Replace(new_game), Some(GameEvent::Started))
            }
            _ => (StateAction::None, None)
        }
    }

    fn draw(&self, board_state: &Option<BoardState>, window: &mut Window, z_index: u32) -> () {
        // TODO draw boardstate.global
        // TODO draw hand
        window.draw_ex(&Circle::new((300, 300), 32), Col(Color::BLUE), Transform::IDENTITY, z_index);
    }

    fn update(&mut self, board_state: &mut Option<BoardState>) -> Option<GameEvent> {
        // TODO async load
        *board_state = Some(BoardState::load_board("cards.json"));


        if self.timer > 0 {
            self.timer -= 1;
        }

        if self.timer % 60 == 0 {
            println!("Seconds remaining: {}", self.timer / 60)
        }

        if self.timer == 0 {
            Some(GameEvent::Timeout)
        } else {
            None
        }
    }
}

// TODO: cache widgets?
// TODO: we have API problems. Get LoadState to generate empty state (or its shell), pass it in Event to GameplayState
//  (or let it generate a new one) and  then pass it into constructors. This will eliminate all the Started states and
//  Option<StateInternalData>s in our states, and will make the code cleaner.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct TakeTurnState {
    widgets: Option<Vec<Box<dyn Widget>>>,
    #[derivative(Debug = "ignore")]
    font: Font,
}

// TODO: load fonts in LoadingState
impl TakeTurnState {
    pub fn new() -> Box<Self> {
        let font = Font::load("Roboto-Italic.ttf").wait().expect("Can't load font file");
        Box::new(Self {
            widgets: None,
            font,
        })
    }
}

impl AutomatonState for TakeTurnState {
    fn event(&mut self, board_state: &mut Option<BoardState>, event: GameEvent) -> ProcessingResult {
        match event {
            // TODO: generalize to arbitrary window sizes
            GameEvent::Started => {
                let card_size = Vector::new(75f32, 120f32);

                let mut widgets = Vec::new();
                widgets.push(Box::new(CardWidget::new(
                    &"Draw pile".to_string(),
                    Vector::new(45, 390),
                    card_size,
                    &self.font,
                    Box::new(|| {
                        println!("Draw pile clicked");
                        GameEvent::Started // TODO: this is placeholder
                    }),
                )) as Box<dyn Widget>);

                let hand = &board_state.as_ref().unwrap().hand.cards;
                let offset_top_left = Vector::new(240.0, 465.0);
                let gap = Vector::new(30.0, 0);
                for (num, card) in hand.clone().drain(..).enumerate() {
                    let name = card.name.clone();
                    let action_text = format!("Card {} clicked", name);
                    widgets.push(Box::new(CardWidget::new(
                        &name,
                        offset_top_left + ((card_size.x_comp() + gap) * num as f32),
                        card_size,
                        &self.font,
                        Box::new(move || {
                            println!("{}", action_text);
                            GameEvent::CardPicked(card.clone())
                        }),
                    )) as Box<dyn Widget>);
                }
                self.widgets = Some(widgets);
                (StateAction::None, None)
            },
            GameEvent::IO(Event::MouseMoved(position)) => {
                if let Some(widgets) = &mut self.widgets {
                    for w in widgets {
                        w.update_hovered(position);
                    }
                }
                (StateAction::None, None)
            },
            GameEvent::IO(Event::MouseButton(MouseButton::Left, ButtonState::Released)) => {
                match &self.widgets {
                    None => (StateAction::None, None),
                    Some(widgets) => {
                        let found = widgets.iter()
                            .map(|widg| {widg.maybe_activate()}) // translate to events (maybe all None)
                            .find(|event| {event.is_some()}) // maybe find first Some
                            .map(|some_event| {some_event.unwrap()}); // if some, unwrap
                        match found {
                            Some(event) => (StateAction::Pop, Some(event)),
                            None => (StateAction::None, None)
                        }
                    }
                }
            },

            GameEvent::IO(Event::Key(Key::Escape, ButtonState::Released)) => {
                (StateAction::Pop, Some(GameEvent::GameEnded))
            },
            _ => (StateAction::None, None)
        }
    }

    fn update(&mut self, board_state: &mut Option<BoardState>) -> Option<GameEvent> {
        None
    }

    // TODO: make widgets draw to Surface, and arrange the Surfaces
    fn draw(&self, board_state: &Option<BoardState>, window: &mut Window, z_index: u32) -> () {
        // TODO: z-index
        for widget in self.widgets.as_ref().unwrap_or(&Vec::new()) {
            widget.draw(window);
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
    action: Box<dyn Fn() -> GameEvent>,
    hovered: bool,
    image: Image,
}

impl CardWidget {
    fn new(name: &String, top_left: Vector, card_size: Vector, font: &Font, action: Box<dyn Fn() -> GameEvent>) -> Self {
        let area = Rectangle::new(top_left, card_size);
        Self {
            name: Box::new(name.to_string().clone()),
            area,
            action,
            hovered: false,
            image: font.render( format!("{}", name).as_str(), &FontStyle::new(12.0, Color::WHITE)).expect("Can't render text")
        }
    }
}

impl Widget for CardWidget {
    fn bounding_box(&self) -> Rectangle {
        self.area
    }

    fn maybe_activate(&self) -> Option<GameEvent> {
        if self.hovered {
            Some((self.action)())
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