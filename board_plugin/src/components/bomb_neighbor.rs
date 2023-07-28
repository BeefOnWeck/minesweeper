use bevy::prelude::Component;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component)]
pub struct BombNeighbor {
    /// Number of neighbor bombs
    pub count: u8,
}