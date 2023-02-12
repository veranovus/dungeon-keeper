use bevy::prelude::*;

use pathfinding::prelude::*;

use noise::{BasicMulti, NoiseFn, Perlin};

use super::tile::{prelude::*, self};
use crate::{tileset, globals::MAP_SIZE};

pub struct GenerationPlugin;

impl Plugin for GenerationPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::Startup, generate_world);
    }
}

// NOTE: Seed that is supplied to the noise generator,
//       remove this after map generation system is complete.
const GENERATION_SEED: u32 = 13;

// NOTE: Multiplier that affects the size of caves in generated map.
pub const GENERATION_CAVE_SIZE: f64 = 3.50;
// NOTE: Threshold that decides which values are considered empty.
pub const GENERATION_CAVE_TRESHOLD: f64 = 0.12;

// NOTE: Generates a random world at startup using a perlin noise.
//       This function also sets up the `World` resource.
fn generate_world(
    mut commands: Commands,
    tileset: Res<tileset::Tileset>,
) {
    // NOTE: Create a vector to store all the solid tile positions.
    let mut world: Vec<(usize, usize)> = Vec::with_capacity(
        MAP_SIZE.0 * MAP_SIZE.1
    );

    // NOTE: Generate the perlin noise.
    let noise = BasicMulti::<Perlin>::new(GENERATION_SEED);

    for y in 0..(MAP_SIZE.1) {
        for x in 0..(MAP_SIZE.0) {
            // NOTE: Convert tile positions to noise positions.
            let position: [f64; 2] = [
                (x as f64) / MAP_SIZE.0 as f64 * GENERATION_CAVE_SIZE,
                (y as f64) / MAP_SIZE.1 as f64 * GENERATION_CAVE_SIZE,
            ];

            let solid = noise.get(position) <= GENERATION_CAVE_TRESHOLD;

            let material = ResourceMaterial::Dirt;
            let state = if solid {
                // NOTE: Push solid tile position to grid
                world.push((x, y));

                // NOTE: Create tile as solid.
                TileState::Solid
            } else {
                // NOTE: Create tile as empty.
                TileState::Empty
            };

            // Create tile entity
            tile::spawn_tile(
                &mut commands, 
                &tileset, 
                (x, y), 
                state, 
                material,
            );
        }
    }

    // NOTE: Setup world resource.
    commands.insert_resource(super::World {
        grid: world.into_iter().collect::<Grid>(),
        entities: vec![None; MAP_SIZE.0 * MAP_SIZE.1]
    });
}