use bevy_egui::egui::Color32;
use rand::Rng;
use std::collections::VecDeque;
use bevy::prelude::*;
use pathfinding::prelude::*;

use crate::{world, tileset, globals, turn_system};

use super::{prelude::*, turn, name, worker};

pub mod prelude {
    pub use super::{
        Pawn,
        Selectable,
        Position,
        Alignment,
    };
}

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::Update, process_pawn_turns)
            .add_system_to_stage(CoreStage::PostUpdate, highlight_selected_pawns);
    }
}

// NOTE: Highlight color used for selected pawns without `Alignment`.
const DEFAULT_PAWN_HIGHLIGHT_COLOR: Color = Color::WHITE;

// NOTE: Default glyph for any pawn.
pub const DEFAULT_PAWN_GLYPH: usize = 2;

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
    pub fn successors(&self, world: &world::World) -> Vec<(Position, u32)> {
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
                 
                // NOTE: If the successor is not a solid tile and if
                //       it is not occupied, add it to the vector.
                if !world.is_solid_tile(pos) && world.get_entity(pos).is_none() {
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

// NOTE: Alignment component, determines who owns a pawn.
#[allow(dead_code)]
#[derive(Component, PartialEq, Eq, PartialOrd, Ord)]
pub enum Alignment {
    Neutral = -1,
    Player = 1,
    Enemy = 0,
}

impl Alignment {
    pub fn identifier(&self) -> &'static str {
        match self {
            Alignment::Neutral => "N",
            Alignment::Player => "P",
            Alignment::Enemy => "E",
        }
    }

    pub fn highlight(&self) -> Color {
        match self {
            Alignment::Neutral => Color::rgb(1.0, 0.5, 1.0),
            Alignment::Player => Color::rgb(0.4, 0.4, 1.0),
            Alignment::Enemy => Color::rgb(1.0, 0.4, 0.4),
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Alignment::Neutral => Color::rgb(0.8, 0.2, 0.8),
            Alignment::Player => Color::rgb(0.1, 0.1, 0.7),
            Alignment::Enemy => Color::rgb(0.7, 0.1, 0.1),
        }
    }

    pub fn color32(&self) -> Color32 {
        match self {
            Alignment::Neutral => Color32::from_rgb(255, 120, 255),
            Alignment::Player => Color32::from_rgb(25, 25, 255),
            Alignment::Enemy => Color32::from_rgb(255, 25, 25),
        }
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
    glyph: usize,
    position: (usize, usize),
    color: Color,
) -> Entity {
    // NOTE: Create a basic tileset sprite entity.
    let e = tileset::spawn_sprite_from_tileset(
        commands,
        tileset,
        glyph,
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
            hit_die: HitDie::D6,
            current: 100,
            maximum: 100,
        });

    // NOTE: Insert entity into world.
    world.set_entity(position, Some(e));

    return e;
}

// NOTE: Spawns a default entity with an alignment, this
//       is the preffered way of spawning an entity.
pub fn spawn_default_pawn_with_alignment(
    commands: &mut Commands,
    world: &mut world::World,
    tileset: &tileset::Tileset,
    glyph: usize,
    position: (usize, usize),
    alignment: Alignment,
) -> Entity {
    let e = spawn_default_pawn(
        commands, world, tileset, glyph, position, alignment.color()
    );

    commands.entity(e).insert(alignment);

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

// NOTE: Finds the shortest path from the pawn's position to
//       target. Caller has to remove the initial position from
//       the vector if a path is found.
pub fn pawn_find_path(
    position: Position,
    target: Position,
    world: &world::World,
) -> Option<(Vec<Position>, u32)> {
    return astar(
        &position, 
        |p| p.successors(&world),
        |p| p.distance(&target), 
        |p| *p == target
    );
}

// NOTE: Process every pawns turn.
fn process_pawn_turns(
    mut query: Query<(Entity, &mut TaskQueue, &mut Transform, &mut Position), With<Pawn>>,
    mut world: ResMut<world::World>,
    mut global_work_pool: ResMut<worker::GlobalWorkPool>,
    mut event_reader: EventReader<turn_system::TurnOverEvent>,
    mut mine_tile_ew: EventWriter<worker::MineTileEvent>,
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
            &mut global_work_pool,
            &mut mine_tile_ew,
        );
    }
}

// NOTE: Changes a pawn's color if it is selected, depending on
//       their `Alignment`. If they don't have one, their color is
//       set to `DEFAULT_PAWN_HIGHLIGHT_COLOR`. 
fn highlight_selected_pawns(
    mut query: Query<(
        &Selectable, 
        Option<&Alignment>, 
        &mut TextureAtlasSprite
    ), (
        With<Pawn>, 
        Changed<Selectable>
    )>,
) {
    for (selectable, alignment, mut sprite) in &mut query {
        let new_color = match alignment {
            Some(a) => a.highlight(),
            None => DEFAULT_PAWN_HIGHLIGHT_COLOR,
        };

        if selectable.selected {
            sprite.color = new_color;
        } else {
            sprite.color = selectable.original_color;
        }
    }
}