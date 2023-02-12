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
pub enum ResourceMaterial {
    Dirt,
    Rock,
    Coal,
    Iron,
    Gold,
    Crystal,
}

impl ResourceMaterial {
    pub fn color(&self) -> Color {
        match self {
            ResourceMaterial::Dirt => Color::hex("4D312B").unwrap(),
            ResourceMaterial::Rock => Color::hex("4D4D4D").unwrap(),
            ResourceMaterial::Coal => Color::hex("141414").unwrap(),
            ResourceMaterial::Iron => Color::hex("AAA8bA").unwrap(),
            ResourceMaterial::Gold => Color::hex("FFD700").unwrap(),
            ResourceMaterial::Crystal => Color::hex("DE10DA").unwrap(),
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