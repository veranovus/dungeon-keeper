pub mod component;
mod selection;
mod order;

use bevy::prelude::*;

use crate::{pawn, tileset, world};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(selection::SelectionPlugin)
            .add_plugin(order::OrderPlugin)
            .add_startup_system_to_stage(StartupStage::PostStartup, spawn_player_pawn);
    }
}

// NOTE: Spawn a player pawn for testing purposes.
fn spawn_player_pawn(
    mut commands: Commands,
    mut world: ResMut<world::World>,
    t: Res<tileset::Tileset>,
) {
    let e = pawn::core::spawn_default_pawn(
        &mut commands,
        &mut world,
        &t,
        (50, 35),
        Color::rgb(0.1, 0.1, 1.0),
    );

    commands.entity(e).insert(component::Player);

    let e = pawn::core::spawn_default_pawn(
        &mut commands,
        &mut world,
        &t,
        (50, 40),
        Color::rgb(1.0, 0.1, 0.1),
    );

    commands.entity(e).insert(component::Player);
}
