use quicksilver::prelude::*;
use quicksilver::Future;
use derivative::*;
use crate::automaton::*;
use crate::game_objects::*;

// should be even: we often use half of the unit (centering etc.) and half-pixels break the text antialiasing
pub const UI_UNIT: f32 = 16.0;
pub const TEXT_SIZE: f32 = UI_UNIT * 1.5;
pub const PAD_SIZE: f32 = UI_UNIT;

pub trait Widget: std::fmt::Debug {
    fn bounding_box(&self) -> Rectangle;
    fn maybe_activate(&self) -> Option<GameEvent>;
    fn draw(&self, window: &mut Window) -> Result<()>;
    fn update_hovered(&mut self, pointer_position: Vector);
}

/// Implemented by widgets that represent a card.
pub trait CardWidget: Widget {
    fn new(card: Card, top_left: Vector, font: &Font, on_action: Option<GameEvent>) -> Self;
}

#[derive(Debug)]
pub enum ZoneDirection {
    Horizontal,
    Vertical,
}

/// A rectangle containing a list of widgets with the same type. New widgets can be added either
/// vertically or horizontally (no grid).
///
/// Zone owns its widgets.
///
/// Area is recalculated as widgets are added.
#[derive(Debug)]
pub struct CardZone<W> where W: CardWidget {
    zone_id: BoardZone,
    area: Rectangle,
    direction: ZoneDirection,
    widgets: Box<Vec<W>>,
}

impl<W> CardZone<W> where W: CardWidget {
    pub fn new(zone_id: BoardZone, top_left: Vector, direction: ZoneDirection) -> Self {
        Self {
            zone_id: zone_id,
            direction,
            widgets: Box::new(Vec::new()),
            area: Rectangle::new(top_left, Vector::new(0, 0)), // TODO: leave space for title
        }
    }

    pub fn add(&mut self, card: Card, font: &Font, on_action: Option<GameEvent>) {
        // NOTE: each cardWidget is wrapped in half-padding on each side. Newly placed widget must also be shifted by
        // a half-padding when placed.
        match self.direction {
            ZoneDirection::Horizontal => {
                let widgets_width = match self.widgets.first() {
                    Some(w) => ((PAD_SIZE + w.bounding_box().size.x) * self.widgets.len() as f32),
                    None => 0.0
                };
                let widget_pos = self.area.pos + Vector::new(widgets_width + PAD_SIZE / 2.0, PAD_SIZE / 2.0);
                let new_widget = W::new(card, widget_pos, &font, on_action);

                let widget_size = new_widget.bounding_box().size;
                self.area = Rectangle::new(
                    self.area.pos,
                    Vector::new(
                        PAD_SIZE + (PAD_SIZE * self.widgets.len() as f32) + (widget_size.x * (self.widgets.len() + 1) as f32),
                        widget_size.y + PAD_SIZE,
                    ),
                );
                self.widgets.push(new_widget);
            }
            ZoneDirection::Vertical => {
                let widgets_height = match self.widgets.first() {
                    Some(w) => ((PAD_SIZE + w.bounding_box().size.y) * self.widgets.len() as f32),
                    None => 0.0
                };
                let widget_pos = self.area.pos + Vector::new(PAD_SIZE / 2.0, widgets_height + PAD_SIZE / 2.0);
                let new_widget = W::new(card, widget_pos, &font, on_action);

                let widget_size = new_widget.bounding_box().size;
                self.area = Rectangle::new(
                    self.area.pos,
                    Vector::new(
                        widget_size.x + PAD_SIZE,
                        PAD_SIZE + (PAD_SIZE * self.widgets.len() as f32) + (widget_size.y * (self.widgets.len() + 1) as f32),
                    ),
                );
                self.widgets.push(new_widget);
            }
        };
    }
}

impl<W: CardWidget> Widget for CardZone<W> {
    fn bounding_box(&self) -> Rectangle {
        self.area
    }

    fn maybe_activate(&self) -> Option<GameEvent> {
        self.widgets.iter()
            .map(|widg| { widg.maybe_activate() }) // translate to events (maybe all None)
            .find(|event| { event.is_some() }) // maybe find first Some
            .map(|some_event| { some_event.unwrap() }) // if some, unwrap
    }

    fn draw(&self, window: &mut Window) -> Result<()> {
        window.draw(&self.area, Col(Color::from_rgba(50, 50, 100, 1.0)));
        for widget in self.widgets.iter() {
            widget.draw(window);
        }
        Ok(())
    }

    fn update_hovered(&mut self, pointer_position: Vector) {
        // TODO: do not call maybe_hovered on subwidgets if zone isn't hovered
        for widget in self.widgets.iter_mut() {
            widget.update_hovered(pointer_position);
        }
    }
}

#[derive(Debug)]
pub struct CardFull {
    card: Box<Card>,
    area: Rectangle,
    on_action: Option<GameEvent>,
    hovered: bool,
    image: Image,
}

impl CardWidget for CardFull {
    fn new(card: Card, top_left: Vector, font: &Font, on_action: Option<GameEvent>) -> Self {
        let area = Rectangle::new(top_left, Vector::new(7.0 * UI_UNIT, 12.0 * UI_UNIT));
        let image = font.render(
            format!("{}", card.name).as_str(),
            &FontStyle::new(TEXT_SIZE, Color::WHITE),
        ).expect("Can't render text");
        Self {
            card: Box::new(card),
            area,
            on_action,
            hovered: false,
            image,
        }
    }
}

impl Widget for CardFull {
    fn bounding_box(&self) -> Rectangle {
        self.area
    }

    fn maybe_activate(&self) -> Option<GameEvent> {
        if self.hovered {
            self.on_action.clone()
        } else {
            None
        }
    }

    fn draw(&self, window: &mut Window) -> Result<()> {
        let position = self.area.pos;

        if self.hovered {
            let border_size = self.area.size + Vector::new(PAD_SIZE, PAD_SIZE);
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

#[derive(Debug)]
pub struct CardIcon {
    card: Box<Card>,
    area: Rectangle,
    on_action: Option<GameEvent>,
    hovered: bool,
    image: Image,
}

impl CardWidget for CardIcon {
    fn new(card: Card, top_left: Vector, font: &Font, on_action: Option<GameEvent>) -> Self {
        let area = Rectangle::new(top_left, Vector::new(7.0 * UI_UNIT, 2.0 * UI_UNIT));
        let image = font.render(
            format!("{}", card.name).as_str(),
            &FontStyle::new(TEXT_SIZE, Color::WHITE),
        ).expect("Can't render text");
        Self {
            card: Box::new(card),
            area,
            on_action,
            hovered: false,
            image,
        }
    }
}

impl Widget for CardIcon {
    fn bounding_box(&self) -> Rectangle {
        self.area
    }

    fn maybe_activate(&self) -> Option<GameEvent> {
        if self.hovered {
            self.on_action.clone()
        } else {
            None
        }
    }

    fn draw(&self, window: &mut Window) -> Result<()> {
        let position = self.area.pos;

        if self.hovered {
            let border_size = self.area.size + Vector::new(PAD_SIZE, PAD_SIZE);
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


#[derive(Debug)]
pub struct Button {
    text: Box<String>,
    area: Circle,
    on_action: Option<GameEvent>,
    hovered: bool,
    image: Image,
}

impl Button {
    pub fn new(text: String, center: Vector, font: &Font, on_action: Option<GameEvent>) -> Self {
        let area = Circle::new(center, 2.0 * UI_UNIT);
        let image = font.render(
            text.as_str(),
            &FontStyle::new(TEXT_SIZE, Color::WHITE),
        ).expect("Can't render text");
        Self {
            text: Box::new(text),
            area,
            on_action,
            hovered: false,
            image,
        }
    }
}

impl Widget for Button {
    fn bounding_box(&self) -> Rectangle {
        self.area.bounding_box()
    }

    fn maybe_activate(&self) -> Option<GameEvent> {
        if self.hovered {
            self.on_action.clone()
        } else {
            None
        }
    }

    fn draw(&self, window: &mut Window) -> Result<()> {
        let position = self.area.pos - (self.image.area().size / 2.0); // put image to the circle's center

        let color = if self.hovered && self.on_action.is_some() {
            Col(Color::from_rgba(100, 100, 100, 1.0))
        } else {
            Col(Color::from_rgba(50, 50, 50, 1.0))
        };

        let text_rect = self.image.area().translate(position);
        window.draw(&self.area, color);
        window.draw(&text_rect, Img(&self.image));
        Ok(())
    }

    fn update_hovered(&mut self, pointer_position: Vector) {
        self.hovered = self.area.contains(pointer_position);
    }
}