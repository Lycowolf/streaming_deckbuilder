use quicksilver::prelude::*;
use quicksilver::Future;
use derivative::*;
use crate::automaton::*;
use crate::game_objects::*;
use crate::loading::CARD_TITLE_FONT;
use crate::loading::Assets;
use std::collections::HashMap;
use std::rc::Rc;

// should be even: we often use half of the unit (centering etc.) and half-pixels break the text antialiasing
pub const UI_UNIT: f32 = 16.0;
pub const TEXT_SIZE: f32 = UI_UNIT * 1.5;
pub const PAD_SIZE: f32 = UI_UNIT;
const TITLE_OFFSET: (f32, f32) = (5.0, 5.0); // card background image does not cover whole rectangle

const MAX_Z_PER_WIDGET: f32 = 10.0; // when nesting widgets, child widgets will be offset by this much in Z direction

pub type CardHandler = Box<dyn Fn(usize, &Card, BoardZone) -> Option<GameEvent>>;

pub trait Widget: std::fmt::Debug {
    fn bounding_box(&self) -> Rectangle;
    fn maybe_activate(&self) -> Option<GameEvent>;
    fn draw(&self, window: &mut Window) -> Result<()>;
    fn update_hovered(&mut self, pointer_position: Vector);
}

/// Implemented by widgets that represent a card.
pub trait CardWidget: Widget {
    fn new(card: Card, top_left: Vector, z_index: f32, assets: &Assets, on_action: Option<GameEvent>) -> Self;
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
    z_index: f32,
}

impl<W> CardZone<W> where W: CardWidget {
    pub fn new(zone_id: BoardZone, top_left: Vector, direction: ZoneDirection, z_index: f32) -> Self {
        Self {
            zone_id: zone_id,
            direction,
            widgets: Box::new(Vec::new()),
            area: Rectangle::new(top_left, Vector::new(0, 0)), // TODO: leave space for title
            z_index,
        }
    }

    pub fn from_container(container: &CardContainer, top_left: Vector, direction: ZoneDirection, z_index: f32, assets: &Assets, handlers: &HashMap<BoardZone, CardHandler>) -> Self {
        let mut zone = CardZone::new(container.zone, top_left, direction, z_index);

        for (idx, card) in container.cards.iter().enumerate() {
            let action = match handlers.get(&zone.zone_id) {
                Some(handler) => handler(idx, &card, zone.zone_id),
                None => None
            };
            zone.add(card.clone(), assets, action);
        }

        zone
    }

    pub fn add(&mut self, card: Card, assets: &Assets, on_action: Option<GameEvent>) {
        // NOTE: each cardWidget is wrapped in half-padding on each side. Newly placed widget must also be shifted by
        // a half-padding when placed.
        match self.direction {
            ZoneDirection::Horizontal => {
                let widgets_width = match self.widgets.first() {
                    Some(w) => ((PAD_SIZE + w.bounding_box().size.x) * self.widgets.len() as f32),
                    None => 0.0
                };
                let widget_pos = self.area.pos + Vector::new(widgets_width + PAD_SIZE / 2.0, PAD_SIZE / 2.0);
                let new_widget = W::new(card, widget_pos, self.z_index + MAX_Z_PER_WIDGET, &assets, on_action);

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
                let new_widget = W::new(card, widget_pos, self.z_index + MAX_Z_PER_WIDGET, &assets, on_action);

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
        window.draw_ex(&self.area, Col(Color::from_rgba(50, 50, 100, 1.0)), Transform::IDENTITY, self.z_index);
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

fn border_color(hovered: bool, available: bool, handled: bool) -> Color {

    if !handled {
        return Color::from_rgba(100, 100, 100, 0.0) // transparent (inactive);
    }
    
    if !available {
        return Color::from_rgba(200, 100, 100, 1.0)
    }

    if hovered {
        Color::from_rgba(100, 100, 100, 1.0)
    } else {
        Color::from_rgba(40, 100, 40, 1.0)
    }
}

#[derive(Debug)]
pub struct CardFull {
    card: Box<Card>,
    area: Rectangle,
    z_index: f32,
    on_action: Option<GameEvent>,
    hovered: bool,
    image: Rc<Image>,
    background: Rc<Image>,
    title: Image,
}

impl CardWidget for CardFull {
    fn new(card: Card, top_left: Vector, z_index: f32, assets: &Assets, on_action: Option<GameEvent>) -> Self {
        let area = Rectangle::new(top_left, Vector::new(7.0 * UI_UNIT, 12.0 * UI_UNIT));
        let card = Box::new(card);
        let title = assets.fonts[CARD_TITLE_FONT].render(
            format!("{}", card.name).as_str(),
            &FontStyle::new(TEXT_SIZE, Color::WHITE),
        ).expect("Can't render text");
        let image = assets.images[&card.image].clone();
        let background = assets.images[crate::loading::CARD_BACKGROUND_IMG].clone();
        Self {
            card,
            area,
            z_index,
            on_action,
            hovered: false,
            image,
            background,
            title,
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

        let border_size = self.area.size + Vector::new(PAD_SIZE, PAD_SIZE);
        let border_position = position - (border_size - self.area.size) * 0.5;
        let border_area = Rectangle::new(border_position, border_size);

        window.draw_ex(&border_area,
            Col(border_color(self.hovered, self.card.available, self.on_action.is_some())),
            Transform::IDENTITY,
            self.z_index);

        let text_rect = self.title.area().translate(position).translate(TITLE_OFFSET);
        window.draw_ex(&self.area, Img(&self.background), Transform::IDENTITY, self.z_index + 1.0);
        window.draw_ex(&self.area, Img(&self.image), Transform::IDENTITY, self.z_index + 2.0);
        window.draw_ex(&text_rect, Img(&self.title), Transform::IDENTITY, self.z_index + 3.0);
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
    z_index: f32,
    on_action: Option<GameEvent>,
    hovered: bool,
    image: Image,
}

impl CardWidget for CardIcon {
    fn new(card: Card, top_left: Vector, z_index: f32, assets: &Assets, on_action: Option<GameEvent>) -> Self {
        let area = Rectangle::new(top_left, Vector::new(7.0 * UI_UNIT, 2.0 * UI_UNIT));
        let image = assets.fonts[CARD_TITLE_FONT].render(
            format!("{}", card.name).as_str(),
            &FontStyle::new(TEXT_SIZE, Color::WHITE),
        ).expect("Can't render text");
        Self {
            card: Box::new(card),
            area,
            z_index,
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

        let border_size = self.area.size + Vector::new(PAD_SIZE, PAD_SIZE);
        let border_position = position - (border_size - self.area.size) * 0.5;
        let border_area = Rectangle::new(border_position, border_size);

        window.draw_ex(&border_area,
            Col(border_color(self.hovered, self.card.available, self.on_action.is_some())),
            Transform::IDENTITY,
            self.z_index);

        let text_rect = self.image.area().translate(position);
        window.draw_ex(&self.area, Col(Color::from_rgba(50, 50, 50, 1.0)), Transform::IDENTITY, self.z_index + 1.0);
        window.draw_ex(&text_rect, Img(&self.image), Transform::IDENTITY, self.z_index + 1.0);
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
    z_index: f32,
    on_action: Option<GameEvent>,
    hovered: bool,
    image: Image,
}

impl Button {
    pub fn new(text: String, center: Vector, z_index: f32, assets: &Assets, on_action: Option<GameEvent>) -> Self {
        let area = Circle::new(center, 2.0 * UI_UNIT);
        let image = assets.fonts[CARD_TITLE_FONT].render(
            text.as_str(),
            &FontStyle::new(TEXT_SIZE, Color::WHITE),
        ).expect("Can't render text");
        Self {
            text: Box::new(text),
            area,
            z_index,
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
        // avoid putting text on half-pixel coordinates
        let half_size_x = (self.image.area().size / 2.0).x.ceil();
        let half_size_y = (self.image.area().size / 2.0).y.ceil();
        let position = self.area.pos - Vector::new(half_size_x, half_size_y); // put image to the circle's center

        let color = if self.hovered && self.on_action.is_some() {
            Col(Color::from_rgba(100, 100, 100, 1.0))
        } else {
            Col(Color::from_rgba(50, 50, 50, 1.0))
        };

        let text_rect = self.image.area().translate(position);
        window.draw_ex(&self.area, color, Transform::IDENTITY, self.z_index);
        window.draw_ex(&text_rect, Img(&self.image), Transform::IDENTITY, self.z_index + 1.0);
        Ok(())
    }

    fn update_hovered(&mut self, pointer_position: Vector) {
        self.hovered = self.area.contains(pointer_position);
    }
}