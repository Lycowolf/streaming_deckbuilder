use quicksilver::prelude::*;
use quicksilver::Future;
use derivative::*;
use crate::automaton::*;
use crate::game_objects::*;

pub const UI_UNIT: f32 = 15.0;
pub const CARD_WIDTH: f32 = 7.0 * UI_UNIT;
pub const CARD_HEIGHT: f32 = 12.0 * UI_UNIT;
pub const CARD_PAD_HORIZONTAL: f32 = 1.0 * UI_UNIT;
pub const CARD_PAD_VERTICAL: f32 = 1.0 * UI_UNIT;

pub trait Widget: std::fmt::Debug {
    fn bounding_box(&self) -> Rectangle;
    fn maybe_activate(&self) -> Option<GameEvent>;
    fn draw(&self, window: &mut Window) -> Result<()>;
    fn update_hovered(&mut self, pointer_position: Vector);
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct CardWidget {
    name: Box<String>,
    area: Rectangle,
    #[derivative(Debug = "ignore")]
    on_action: GameEvent,
    hovered: bool,
    image: Image,
}

impl CardWidget {
    pub fn new(name: &String, top_left: Vector, card_size: Vector, font: &Font, on_action: GameEvent) -> Self {
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

        if self.hovered {
            let border_size = self.area.size + Vector::new(CARD_PAD_HORIZONTAL, CARD_PAD_VERTICAL);
            let border_position = position - (border_size - self.area.size) * 0.5;
            let border_area = Rectangle::new(border_position, border_size);
            window.draw(&border_area, Col(Color::from_rgba(100, 100, 100, 1.0)));
        }

        let text_rect = self.image.area().translate(position);
        window.draw(&self.area, Col(Color::from_rgba(50, 50, 50, 1.0)));
        window.draw(&text_rect, Img(&self.image));
        Ok(())
    }

    fn update_hovered(&mut self, pointer_position: Vector) {
        self.hovered = self.area.contains(pointer_position);
    }
}