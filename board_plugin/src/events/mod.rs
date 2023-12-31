use bevy::ecs::event::Event;
use crate::components::Coordinates;

#[derive(Debug, Copy, Clone, Event)]
pub struct TileTriggerEvent{
    pub coordinates: Coordinates
}