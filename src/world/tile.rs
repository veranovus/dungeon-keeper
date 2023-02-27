use bevy::prelude::*;
use bevy_egui::egui::Color32;

use crate::pawn::prelude::*;
use crate::{tileset, globals};

// TODO: Add necessary comments.

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PreUpdate, update_tile_visuals);
    }
}

pub mod prelude {
    pub use super::{
        TileState,
        Tile,
        ResourceMaterial,
        Resource,
    };
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
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
#[derive(Debug, Clone, Copy)]
pub enum ResourceMaterial {
    Dirt = 0,
    Stone,
    Coal,
    Iron,
    Gold,
    Crystal,
}

impl ResourceMaterial {
    pub fn identifier(&self) -> &str {
        match self {
            ResourceMaterial::Dirt => "D",
            ResourceMaterial::Stone => "S",
            ResourceMaterial::Coal => "C",
            ResourceMaterial::Iron => "I",
            ResourceMaterial::Gold => "G",
            ResourceMaterial::Crystal => "Cr",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            ResourceMaterial::Dirt => Color::hex("4D312B").unwrap(),
            ResourceMaterial::Stone => Color::hex("4D4D4D").unwrap(),
            ResourceMaterial::Coal => Color::hex("242424").unwrap(),
            ResourceMaterial::Iron => Color::hex("AAA8BA").unwrap(),
            ResourceMaterial::Gold => Color::hex("FFD700").unwrap(),
            ResourceMaterial::Crystal => Color::hex("DE10DA").unwrap(),
        }
    }

    pub fn color32(&self) -> Color32 {
        match self {
            ResourceMaterial::Dirt => Color32::from_rgb(77, 49, 43),
            ResourceMaterial::Stone => Color32::from_rgb(77, 77, 77),
            ResourceMaterial::Coal => Color32::from_rgb(56, 56, 56),
            ResourceMaterial::Iron => Color32::from_rgb(170, 168, 186),
            ResourceMaterial::Gold => Color32::from_rgb(255, 215, 0),
            ResourceMaterial::Crystal => Color32::from_rgb(222, 16, 218), 
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

#[derive(Component, Debug, Clone, Copy)]
pub struct Resource {
    pub material: ResourceMaterial,
    pub quantity: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct TileData {
    pub state: TileState,
    pub resource: Resource,
    pub marked: bool,
}

impl Default for TileData {
    fn default() -> Self {
        Self {
            state: TileState::Empty,
            resource: Resource {
                material: ResourceMaterial::Dirt,
                quantity: 0,
            },
            marked: false,
        }
    }
}

pub fn spawn_tile(
    commands: &mut Commands,
    tile_grid: &mut Vec<TileData>,
    tileset: &tileset::Tileset,
    position: (usize, usize),
    state: TileState,
    material: ResourceMaterial,
) -> Entity {
    // NOTE: Spwan and initialize the tile entity
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

    // NOTE: Set the tile in the tile grid.
    let tile = tile_grid.get_mut(
        (position.1 * globals::MAP_SIZE.0) + position.0
    ).unwrap();

    *tile = TileData {
        state,
        resource: Resource {
            material,
            quantity: 1,
        },
        marked: false,
    };
    
    return e;
}

fn update_tile_visuals(
    mut query: Query<
        (&Tile, &Resource, &mut TextureAtlasSprite), 
        Or<(Changed<Tile>, Changed<Resource>)>,
    >,
) {
    for (tile, res, mut sprite) in &mut query {
        // NOTE: Change the glyph according to the `TileState`.
        *sprite = TextureAtlasSprite::new(tile.state.glyph());

        // NOTE: Change the color according to the `ResourceMaterial`.
        sprite.color = res.material.color();

        // NOTE: Set the anchor to defualt
        sprite.anchor = globals::DEFAULT_SPRITE_ANCHOR;
    }
}