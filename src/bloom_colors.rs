use bevy::ecs::component::Component;
use ratatui::style::Color;

#[derive(Component, Clone, Copy)]
#[repr(transparent)]
pub struct RatatuiBloomColors(pub &'static [Color]);
