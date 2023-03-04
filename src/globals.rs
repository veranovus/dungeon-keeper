// NOTE: Collection of commonly used and shared constants for the game

#![allow(dead_code)]

use bevy::sprite::Anchor;

// ##### WINDOW #####

// NOTE: Initial window size for the application
pub const WINDOW_SIZE: (usize, usize) = (800, 600);
pub const WINDOW_TITLE: &str = "Dungeon Keeper | v0.2.0.7_@wr0.1";

// ##### SPRITE #####

pub const DEFAULT_SPRITE_ANCHOR: Anchor = Anchor::BottomLeft;
// NOTE: Size of each image in atlas in pixels
pub const SPRITE_IMAGE_SIZE: (usize, usize) = (12, 12);
// NOTE: Size of atlas in sprites
pub const ATLAS_SIZE: (usize, usize) = (16, 16);
// NOTE: How many sprites should fit in a screen from left to right
const SPRITE_RATIO: usize = 64;
// NOTE: Scale for any default sprite
pub const SPRITE_SCALE: f32 = (WINDOW_SIZE.0 as f32 / SPRITE_RATIO as f32) / SPRITE_IMAGE_SIZE.0 as f32;
// NOTE: Size of a sprite with a default scale
pub const SPRITE_SIZE: f32 = SPRITE_IMAGE_SIZE.0 as f32 * SPRITE_SCALE;

// ##### SPRITE ORDER #####

pub const SPRITE_ORDER_WORLD: f32 = 100.0;
pub const SPRITE_ORDER_ENTITY: f32 = 200.0;
pub const SPRITE_ORDER_USER: f32 = 900.0;

// ##### GAME #####

// NOTE: Default size for the world in tiles. 
pub const MAP_SIZE: (usize, usize) = (100, 100);
