use bevy::ecs::component::Component;
use ratatui::style::Color;
use ratatui_kurbo::{BezPathShape, ShapeBuilder, ShapeParams, kurbo};

#[derive(Component, Clone)]
pub enum RatatuiShape {
    Rect(BezPathShape<kurbo::Rect>),
    Circle(BezPathShape<kurbo::Circle>),
    Triangle(BezPathShape<kurbo::Triangle>),
}

impl RatatuiShape {
    pub fn draw(&self, builder: &mut ShapeBuilder, color: Color, affine: kurbo::Affine) {
        match self {
            Self::Rect(rect) => builder.draw(rect, ShapeParams { color, affine }),
            Self::Circle(circle) => builder.draw(circle, ShapeParams { color, affine }),
            Self::Triangle(triangle) => builder.draw(triangle, ShapeParams { color, affine }),
        }
    }
}
