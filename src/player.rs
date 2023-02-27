pub mod component;
pub mod resource;
pub mod order;
mod selection;

use bevy::prelude::*;

use crate::{pawn::{prelude::*, self}, tileset, world};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(selection::SelectionPlugin)
            .add_plugin(order::OrderPlugin)
            .add_plugin(resource::ResourcePlugin)
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
        pawn::core::DEFAULT_PAWN_GLYPH,
        (50, 35),
        Alignment::Player,
    );

    pawn::core::spawn_default_pawn_with_alignment(
        &mut commands,
        &mut world,
        &t,
        pawn::core::DEFAULT_PAWN_GLYPH,
        (50, 39),
        Alignment::Player,
    );

    pawn::core::spawn_default_pawn_with_alignment(
        &mut commands,
        &mut world,
        &t,
        pawn::core::DEFAULT_PAWN_GLYPH,
        (48, 37),
        Alignment::Neutral,
    );

    pawn::core::spawn_default_pawn_with_alignment(
        &mut commands,
        &mut world,
        &t,
        pawn::core::DEFAULT_PAWN_GLYPH,
        (52, 37),
        Alignment::Enemy,
    );

    pawn::worker::spawn_worker_pawn(
        &mut commands, 
        &mut world, 
        &t, 
        (46, 37),
    );

    pawn::worker::spawn_worker_pawn(
        &mut commands, 
        &mut world, 
        &t, 
        (45, 35),
    );

    pawn::worker::spawn_worker_pawn(
        &mut commands, 
        &mut world, 
        &t, 
        (44, 37),
    );
}
