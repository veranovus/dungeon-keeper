use rand::Rng;
use std::collections::VecDeque;
use bevy::prelude::*;
use pathfinding::prelude::*;

use crate::{world, tileset, globals, turn_system};

use super::{prelude::*, turn, name};

pub mod prelude {
    pub use super::{
        Pawn,
        Selectable,
        Position,
    };
}

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::Update, process_pawn_turns)
            .add_system_to_stage(CoreStage::PostUpdate, highlight_selected_pawns);
    }
}

// NOTE: Highlight color for selected pawns.
const PAWN_HIGHLIGHT_COLOR: Color = Color::WHITE;

// NOTE: Glyph that is used for default pawns.
const DEFAULT_PAWN_GLYPH: usize = 2;

// NOTE: A tag that is required for every pawn to have.
#[derive(Component)]
pub struct Pawn;

// NOTE: Makes an entity selectable, previous color is
//       necessary because selected entities are highlihted.
#[derive(Component)]
pub struct Selectable {
    pub selected: bool,
    pub original_color: Color,
}

// NOTE: Required for every pawn that posseses a position
//       in the world, also used for A* pathfinding algorithm.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[allow(dead_code)]
impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        return Self {x, y};
    }

    // NOTE: Returns the approximate distance from self to
    //       the other by using the formula x^2 + b^2 = c^2.
    pub fn distance(&self, other: &Position) -> u32 {
        return ((self.x - other.x).pow(2) + (self.y - other.y).pow(2)) as u32;
    }

    // NOTE: Returns the every possible adjected successor position.
    pub fn successors(&self, grid: &Grid) -> Vec<(Position, u32)> {
        // NOTE: Create a vector to store possible successors.
        let mut vec: Vec<Position> = Vec::with_capacity(8);
        
        for y in -1..=1i32 {
            for x in -1..=1i32 {
                // NOTE: Skip the self.
                if x == 0 && y == 0 {
                    continue;
                }

                // NOTE: Calculate the possition for the successor,
                //       skip it if either x or y is below zero.
                let pos: (usize, usize) = (
                    if (self.x + x) < 0 { continue; } else { 
                        (self.x + x) as usize 
                    }, 
                    if (self.y + y) < 0 { continue; } else { 
                        (self.y + y) as usize 
                    },
                );
                 
                // NOTE: If the successor is not a solid tile add it to the vector.
                if !grid.has_vertex(pos) {
                    vec.push(pos.into());
                }
            }
        }

        return vec.into_iter().map(|p| (p, 1)).collect();
    }
}

impl From<(usize, usize)> for Position {
    fn from(p: (usize, usize)) -> Self {
        return Self { x: p.0 as i32, y: p.1 as i32 };
    }
}

impl From<Position> for (usize, usize) {
    fn from(p: Position) -> Self {
        return (p.x as usize , p.y as usize);
    }
}

// NOTE: Picks a random pawn name from pawn name table.
pub fn pick_random_pawn_name() -> &'static str {
    let mut rng = rand::thread_rng();

    let name = rng.gen_range(0..name::PAWN_NAMES.len());
    
    return name::PAWN_NAMES[name];
}

// NOTE: Spawns a default pawn with given location and color.
//       Default pawn components:
//       - Pawn
//       - Position
//       - TaskQueue
//       - Selectable
//       - Name
//       - PawnStats
//       - Health
pub fn spawn_default_pawn(
    commands: &mut Commands,
    world: &mut world::World,
    tileset: &tileset::Tileset,
    position: (usize, usize),
    color: Color,
) -> Entity {
    // NOTE: Create a basic tileset sprite entity.
    let e = tileset::spawn_sprite_from_tileset(
        commands,
        tileset,
        DEFAULT_PAWN_GLYPH,
        Vec3::new(
            position.0 as f32 * globals::SPRITE_SIZE,
            position.1 as f32 * globals::SPRITE_SIZE,
            globals::SPRITE_ORDER_ENTITY,
        ),
        Vec3::new(globals::SPRITE_SCALE, globals::SPRITE_SCALE, 1.0),
        color
    );

    // NOTE: Turn entity into a default pawn.
    commands.entity(e)
        .insert(Pawn)
        .insert(Position::from(position))
        .insert(TaskQueue {
            queue: VecDeque::new(),
            active: Task::None,
        })
        .insert(Selectable {
            selected: false,
            original_color: color,
        })
        .insert(Name::new(pick_random_pawn_name()))
        .insert(PawnStats::default())
        .insert(Health {
            current: 100,
            maximum: 100,
        });

    // NOTE: Insert entity into world.
    world.set_entity(position, Some(e));

    return e;
}

// NOTE: Moves a given pawn to a tile in world.
pub fn move_pawn(
    target: (usize, usize),
    entity: Entity,
    transform: &mut Transform,
    position: &mut Position,
    world: &mut world::World,
) -> bool {
    if !world.is_solid_tile(target) && world.get_entity(target).is_none() {
        // NOTE: Erase past position from the world
        world.set_entity((*position).into(), None);

        // NOTE: Set the new position in the world
        world.set_entity(target, Some(entity));

        transform.translation.x = target.0 as f32 * globals::SPRITE_SIZE;
        transform.translation.y = target.1 as f32 * globals::SPRITE_SIZE;

        // NOTE: Set pawn's position component.
        position.x = target.0 as i32;
        position.y = target.1 as i32;

        return true;
    }

    return false;
}

// NOTE: Process every pawns turn.
fn process_pawn_turns(
    mut query: Query<(Entity, &mut TaskQueue, &mut Transform, &mut Position), With<Pawn>>,
    mut world: ResMut<world::World>,
    mut event_reader: EventReader<turn_system::TurnOverEvent>,
) {
    let mut over = false;
    for _ in event_reader.iter() {
        over = true;
    }

    // NOTE: Return early until the turn is over.
    if !over {
        return;
    }

    // NOTE: Act the pawns turns
    for (entity, mut queue, mut transform, mut position) in &mut query {
        turn::pawn_act_turn(
            entity,
            &mut queue,
            &mut transform,
            &mut position,
            &mut world,
        );
    }
}

// NOTE: Changes a pawns color to `PAWN_HIGHLIGHT_COLOR` if it is selected.
fn highlight_selected_pawns(
    mut query: Query<(&Selectable, &mut TextureAtlasSprite), (With<Pawn>, Changed<Selectable>)>,
) {
    for (selectable, mut sprite) in &mut query {
        if selectable.selected {
            sprite.color = PAWN_HIGHLIGHT_COLOR;
        } else {
            sprite.color = selectable.original_color;
        }
    }
}