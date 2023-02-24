mod name;
pub mod turn;
pub mod core;
pub mod worker;
pub mod combat;
pub mod stats;

#[allow(unused_imports)]
pub mod prelude {
    pub use super::turn::prelude::*;
    pub use super::core::prelude::*;
    pub use super::stats::prelude::*;
    pub use super::combat::prelude::*;
}

use bevy::prelude::*;

pub struct PawnPlugin;

impl Plugin for PawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(core::CorePlugin)
            .add_plugin(worker::WorkerPlugin)
            .add_plugin(combat::CombatPlugin);
    }
}