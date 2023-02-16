use bevy::prelude::*;
use pathfinding::prelude::*;
use log::info;
use noise::{BasicMulti, NoiseFn, Perlin};
use rand::{Rng, rngs::StdRng, SeedableRng};

use super::tile::prelude::*;
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

// NOTE: Multiplier for the size of chunks that are going to turn into stone.
pub const STONE_CHUNK_SIZE: f64 = 5.0;

// NOTE: Offset that will be applied to the noise, when calculating tile hardness.
pub const STONE_CHUNK_NOISE_OFFSET: (f64, f64) = (STONE_CHUNK_SIZE * 2.0, STONE_CHUNK_SIZE * 2.0);

// NOTE: Required hardness treshold a tile has to
//       be over in order to register as a stone.
pub const MINIMUM_STONE_HARDNES: f64 = 0.12; 

// NOTE: Minimum distance that two seperate resources could be.
pub const MINIMUM_RESOURCE_DISTANCE: f32 = 8.0;

// NOTE: How much distance reduced on each maximum attempt.
pub const RESOURCE_DISTANCE_REDUCTION: f32 = 0.5;

// NOTE: Maximum tries for a resource location
pub const MAXIMUM_RESOURCE_ITERATION: usize = 15;

// NOTE: Decides whether or not resources can spread to empty tile.
pub const CAN_SPREAD_TO_FREE_TILE: bool = true;

#[allow(dead_code)]
// NOTE: Calculates the score for a given tile. This will be
//       used for future resoureces, currently unused.
fn calculate_position_score(
    pos: (usize, usize), 
    grid: &Grid,
) -> usize {
    let mut score = 0;

    // NOTE: Calculate the score in x axis.
    for i in 0..2 {
        let mut pos = pos;
        let modifier: i32 = if i == 0 { -1 } else { 1 };

        while grid.has_vertex(pos) {
            pos.0 += modifier as usize;
            score += 1;
        }
    }

    // NOTE: Calculate the score in y axis.
    for i in 0..2 {
        let mut pos = pos;
        let modifier: i32 = if i == 0 { -1 } else { 1 };

        while grid.has_vertex(pos) {
            pos.1 += modifier as usize;
            score += 1;
        }
    }

    return score;
}

// NOTE: Check distance between two resources, and returns a score.
fn calculate_resource_score(
    pos: (usize, usize),
    res: &Vec<(usize, usize)>,
) -> f32 {
    let mut small: f32 = f32::MAX;

    for other in res {
        let diff = (
            (pos.0 as i32 - other.0 as i32).abs(),
            (pos.1 as i32 - other.1 as i32).abs(),
        );

        let distance: f32 = f32::sqrt(
            (diff.0.pow(2) + diff.1.pow(2)) as f32
        );

        if distance < small {
            small = distance;
        }
    }

    return small;
}

// NOTE: Picks a random position from the grid
fn pick_random_tile(
    rng: &mut StdRng,
    res: &Vec<(usize, usize)>,
    grid: &Grid,
) -> (usize, usize) {
    let mut iter = 0;
    let mut distance = MINIMUM_RESOURCE_DISTANCE;

    loop {
        let pos = (
            rng.gen_range(0..MAP_SIZE.0),
            rng.gen_range(0..MAP_SIZE.0),
        );
    
        let score = calculate_resource_score(pos, res);
    
        if grid.has_vertex(pos) && !res.contains(&pos) && score > distance {
            return pos;
        } else {
            iter += 1;

            if iter == MAXIMUM_RESOURCE_ITERATION {
                info!(
                    "Failed to find a suitable resource position
                     during world generation with distance `{}`.", 
                    distance
                );

                distance -= RESOURCE_DISTANCE_REDUCTION;
                iter = 0;
            }
        }
    }
}

// NOTE: Speards resource to nearby tiles, creating more resources.
//       Chance for a new resource to be created is reduced in every generation.
fn spread_resource(
    commands: &mut Commands,
    rng: &mut StdRng,
    res: &mut Vec<(usize, usize)>,
    tileset: &tileset::Tileset,
    grid: &Grid,
    pos: (usize, usize),
    material: ResourceMaterial,
    ratio: f64,
) {
    if ratio < 0.0 {
        return;
    }

    for y in -1..=1_i32 {
        for x in -1..=1_i32 {
            if x.abs() == y.abs() {
                continue;
            }

            let spread = rng.gen_bool(ratio);

            let pos = (
                if (pos.0 as i32 + x) < 0 {
                    continue;
                } else {
                    (pos.0 as i32 + x) as usize
                },
                if (pos.1 as i32 + y) < 0 {
                    continue;
                } else {
                    (pos.1 as i32 + y) as usize
                },
            );

            let solid = grid.has_vertex(pos);

            if !spread || (!CAN_SPREAD_TO_FREE_TILE && !solid) || res.contains(&pos) {
                continue;
            }

            res.push(pos);

            super::tile::spawn_tile(
                commands, 
                tileset, 
                pos, 
                if solid { TileState::Solid } else { TileState::Empty }, 
                material,
            );

            spread_resource(
                commands, 
                rng, 
                res, 
                tileset, 
                grid, 
                pos, 
                material,
                ratio - material.ratio_reduction_rate(),
            );
        }
    }
}

// NOTE: Generates a random world at startup using a perlin noise.
//       This function also sets up the `World` resource.
fn generate_world(
    mut commands: Commands,
    tileset: Res<tileset::Tileset>,
) {
    // NOTE: Setup rng.
    let mut rng = StdRng::seed_from_u64(GENERATION_SEED as u64);

    // NOTE: Generate the perlin noise.
    let noise = BasicMulti::<Perlin>::new(GENERATION_SEED);

    // NOTE: Create a vector to store all the solid tile positions.
    let mut world: Vec<(usize, usize)> = Vec::with_capacity(
        MAP_SIZE.0 * MAP_SIZE.1
    );

    for y in 0..(MAP_SIZE.1) {
        for x in 0..(MAP_SIZE.0) {
            // NOTE: Convert tile positions to noise positions.
            let position: [f64; 2] = [
                (x as f64) / MAP_SIZE.0 as f64 * GENERATION_CAVE_SIZE,
                (y as f64) / MAP_SIZE.1 as f64 * GENERATION_CAVE_SIZE,
            ];

            // NOTE: Push solid tile position to grid
            if noise.get(position) <= GENERATION_CAVE_TRESHOLD {
                world.push((x, y));
            }
        }
    }

    // NOTE: Create the world grid.
    let grid = world.into_iter().collect::<Grid>();

    // NOTE: Setup resource count.
    const RESOURCE_COUNT: usize = 4;

    let resources: [(usize, ResourceMaterial); RESOURCE_COUNT] = [
        (rng.gen_range(ResourceMaterial::Coal.range()), ResourceMaterial::Coal),
        (rng.gen_range(ResourceMaterial::Iron.range()), ResourceMaterial::Iron),
        (rng.gen_range(ResourceMaterial::Gold.range()), ResourceMaterial::Gold),
        (rng.gen_range(ResourceMaterial::Crystal.range()), ResourceMaterial::Crystal),
    ];

    // NOTE: Vector to store exhausted resource positions.
    let mut exhausted: Vec<(usize, usize)> = vec![];

    // Generate resources.
    for i in 0..RESOURCE_COUNT {
        let res = resources[i];
        for _ in 0..res.0 {
            let pos = pick_random_tile(&mut rng, &exhausted, &grid);

            exhausted.push(pos);

            super::tile::spawn_tile(
                &mut commands, 
                &tileset, 
                pos, 
                TileState::Solid, 
                res.1,
            );

            spread_resource(
                &mut commands, 
                &mut rng, 
                &mut exhausted, 
                &tileset, 
                &grid, 
                pos, 
                res.1,
                res.1.ratio(),
            );
        }
    }

    // NOTE: Create non-resource tiles, they are either dirt
    //       or stone depending on the hardness value of the tile.
    for y in 0..(MAP_SIZE.1) {
        for x in 0..(MAP_SIZE.0) {
            let pos = (x, y);

            if exhausted.contains(&pos) {
                continue;
            }

            let state = if grid.has_vertex(pos) {
                TileState::Solid
            } else {
                TileState::Empty
            };

            let noise_pos: [f64; 2] = [
                (x as f64) / MAP_SIZE.0 as f64 * STONE_CHUNK_SIZE + STONE_CHUNK_NOISE_OFFSET.0,
                (y as f64) / MAP_SIZE.1 as f64 * STONE_CHUNK_SIZE + STONE_CHUNK_NOISE_OFFSET.1,
            ];
            let hardness: f64 = noise.get(noise_pos);

            let material = if hardness > MINIMUM_STONE_HARDNES {
                ResourceMaterial::Stone
            } else {
                ResourceMaterial::Dirt
            };

            super::tile::spawn_tile(
                &mut commands, 
                &tileset, 
                pos, 
                state, 
                material,
            );
        }
    }

    // NOTE: Setup world resource.
    commands.insert_resource(super::World {
        grid,
        tiles: vec![false; MAP_SIZE.0 * MAP_SIZE.1],
        entities: vec![None; MAP_SIZE.0 * MAP_SIZE.1],
    });
}