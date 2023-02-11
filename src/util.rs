pub mod dice;
pub mod cursor;

use bevy::prelude::*;

pub mod prelude {
    pub use super::cursor::CursorPos;
    pub use super::dice::{
        Die,
        Advantage,
        DiceRollResult,
        roll,
    };
}

pub struct UtilPlugin;

impl Plugin for UtilPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(cursor::CursorPlugin);
    }
}