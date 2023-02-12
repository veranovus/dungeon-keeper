pub mod component;
mod selection;
mod order;

use bevy::prelude::*;

use crate::{pawn::{prelude::*, self}, tileset, world};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(selection::SelectionPlugin)
            .add_plugin(order::OrderPlugin)
            .add_startup_system_to_stage(StartupStage::PostStartup, spawn_test_pawns);
    }
}

// NOTE: Spawn player pawns for testing purposes.
fn spawn_test_pawns(
    mut commands: Commands,
    mut world: ResMut<world::World>,
    t: Res<tileset::Tileset>,
) {
    pawn::core::spawn_default_pawn_with_alignment(
        &mut commands,
        &mut world,
        &t,
        (50, 35),
        Alignment::Player,
    );

    pawn::core::spawn_default_pawn_with_alignment(
        &mut commands,
        &mut world,
        &t,
        (50, 39),
        Alignment::Player,
    );

    pawn::core::spawn_default_pawn_with_alignment(
        &mut commands,
        &mut world,
        &t,
        (48, 37),
        Alignment::Neutral,
    );

    pawn::core::spawn_default_pawn_with_alignment(
        &mut commands,
        &mut world,
        &t,
        (52, 37),
        Alignment::Enemy,
    );
}
