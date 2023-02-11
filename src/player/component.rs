use bevy::prelude::*;

#[allow(unused_imports)]
pub mod prelude {
    pub use super::Player;
}

// NOTE: A tag component to distinguish player's pawns.
#[derive(Component)]
pub struct Player;