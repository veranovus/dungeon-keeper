pub mod generation;
pub mod tile;

use bevy::prelude::*;

use pathfinding::prelude::*;

use crate::globals;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(generation::GenerationPlugin);
    }
}

// NOTE: World resource, which holds the position
//       data of everything in the world.
#[derive(Resource)]
pub struct World {
    pub grid: Grid,
    pub tiles: Vec<tile::TileData>,
    pub entities: Vec<Option<Entity>>,
}

impl World {
    // NOTE: Return an entity from the given world coordinates,
    //       returns None if that grid doesn't contains any entity.
    pub fn get_entity(&self, pos: (usize, usize)) -> Option<Entity> {
        return self.entities[(pos.1 * globals::MAP_SIZE.0 as usize) + pos.0];
    }

    // NOTE: Sets data of the grid at given position.
    pub fn set_entity(&mut self, pos: (usize, usize), value: Option<Entity>) {
        self.entities[(pos.1 * globals::MAP_SIZE.0 as usize) + pos.0] = value;
    }

    // NOTE: Returns the state of a tile at given position.
    pub fn is_solid_tile(&self, pos: (usize, usize)) -> bool {
        return self.grid.has_vertex(pos);
    }

    // NOTE: Returns the tile in the given position.
    pub fn get_tile(&self, pos: (usize, usize)) -> tile::TileData {
        return self.tiles[(pos.1 * globals::MAP_SIZE.0 as usize) + pos.0];
    }

    // NOTE: Sets the tile in the given position.
    pub fn get_tile_mut(&mut self, pos: (usize, usize)) -> &mut tile::TileData {
        return &mut self.tiles[(pos.1 * globals::MAP_SIZE.0 as usize) + pos.0];
    }
}

// NOTE: Normalizes engine coordinates to grid coordinates.
pub fn normalize_to_world_coordinates(point: Vec2) -> (usize, usize) {
    return (
        (point.x / globals::SPRITE_SIZE) as usize,
        (point.y / globals::SPRITE_SIZE) as usize,
    );
}
