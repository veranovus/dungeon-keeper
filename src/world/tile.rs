use bevy::prelude::*;

use crate::pawn::prelude::*;
use crate::{tileset, globals};

pub mod prelude {
    pub use super::{
        TileState,
        Tile,
        ResourceMaterial,
        Resource,
    };
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum TileState {
    Empty,
    Solid,
}

impl TileState {
    pub fn glyph(&self) -> usize {
        return match self {
            TileState::Empty => '.',
            TileState::Solid => '#',
        } as usize;
    }
}

#[derive(Component)]
pub struct Tile {
    pub state: TileState,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum ResourceMaterial {
    Dirt,
    Stone,
    Coal,
    Iron,
    Gold,
    Crystal,
}

impl ResourceMaterial {
    pub fn color(&self) -> Color {
        match self {
            ResourceMaterial::Dirt => Color::hex("4D312B").unwrap(),
            ResourceMaterial::Stone => Color::hex("4D4D4D").unwrap(),
            ResourceMaterial::Coal => Color::hex("242424").unwrap(),
            ResourceMaterial::Iron => Color::hex("AAA8bA").unwrap(),
            ResourceMaterial::Gold => Color::hex("FFD700").unwrap(),
            ResourceMaterial::Crystal => Color::hex("DE10DA").unwrap(),
        }
    }

    pub fn ratio(&self) -> f64 {
        match self {
            ResourceMaterial::Dirt => 0.00,
            ResourceMaterial::Stone => 0.00,
            ResourceMaterial::Coal => 0.90,
            ResourceMaterial::Iron => 0.80,
            ResourceMaterial::Gold => 0.60,
            ResourceMaterial::Crystal => 0.40,
        }
    }

    pub fn ratio_reduction_rate(&self) -> f64 {
        match self {
            ResourceMaterial::Dirt => 0.00,
            ResourceMaterial::Stone => 0.00,
            ResourceMaterial::Coal => 0.10,
            ResourceMaterial::Iron => 0.15,
            ResourceMaterial::Gold => 0.20,
            ResourceMaterial::Crystal => 0.20,
        }
    }

    pub fn range(&self) -> std::ops::Range<usize> {
        match self {
            ResourceMaterial::Dirt => 0..0,
            ResourceMaterial::Stone => 0..0,
            ResourceMaterial::Coal => 5..(8 + 1),
            ResourceMaterial::Iron => 6..(8 + 1),
            ResourceMaterial::Gold => 4..(6 + 1),
            ResourceMaterial::Crystal => 3..(5 + 1),
        }
    }
}

#[derive(Component)]
pub struct Resource {
    pub material: ResourceMaterial,
    pub quantity: usize,
}

pub fn spawn_tile(
    commands: &mut Commands,
    tileset: &tileset::Tileset,
    position: (usize, usize),
    state: TileState,
    material: ResourceMaterial,
) -> Entity {
    let e = tileset::spawn_sprite_from_tileset(
        commands, 
        tileset, 
        state.glyph(), 
        Vec3::new(
            position.0 as f32 * globals::SPRITE_SIZE,
            position.1 as f32 * globals::SPRITE_SIZE,
            globals::SPRITE_ORDER_WORLD,
        ), 
        Vec3::new(globals::SPRITE_SCALE, globals::SPRITE_SCALE, 1.0), 
        material.color(),
    );

    commands.entity(e)
        .insert(Position::from(position))
        .insert(Tile {
            state
        })
        .insert(Resource {
            material,
            quantity: 0,
        });
    
    return e;
}