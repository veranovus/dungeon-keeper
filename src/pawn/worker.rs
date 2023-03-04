#![allow(unused_variables, unused_mut)]
use std::collections::{VecDeque, HashMap};
use bevy::prelude::*;
use uuid::Uuid;

use crate::{
    world::{self, tile}, 
    tileset, 
    player::{resource, order},
};
use super::{
    turn::prelude::*,
    core::{prelude::*, self, spawn_default_pawn_with_alignment},
};

pub struct WorkerPlugin;

impl Plugin for WorkerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MineTileEvent>()
            .add_event::<RegisterGlobalWorkEvent>()
            .add_startup_system_to_stage(StartupStage::PostStartup, setup_global_work_pool)
            .add_system_to_stage(CoreStage::PreUpdate, worker_behaviour)
            .add_system_to_stage(CoreStage::PostUpdate, register_global_work_event)
            .add_system_to_stage(CoreStage::PostUpdate, mine_tile_event);
    }
}

// NOTE: Default glyph that is used for worker pawns.
pub const DEFAULT_WORKER_PAWN_GLYPH: usize = 1;

// NOTE: Any kind of work which should be executed by a worker,
//       every work has its unique id to identify it.
#[derive(Debug, Clone)]
pub struct GlobalWork {
    pub id: Uuid,
    pub task: Task,
    pub position: Position,
}

impl GlobalWork {
    pub fn new(task: Task, id: Uuid, position: Position) -> Self {
        Self {
            id,
            task,
            position,
        }
    }
}

// NOTE: Resource which holds every avaiable work for the workers.
#[derive(Resource)]
pub struct GlobalWorkValidator {
    pub works: HashMap<Uuid, bool>,
}

impl Default for GlobalWorkValidator {
    fn default() -> Self {
        Self {
            works: HashMap::new(),
        }
    }
}

#[allow(dead_code)]
impl GlobalWorkValidator {
    pub fn validate(&mut self, id: &Uuid) -> Option<&bool> {
        return self.works.get(id);
    }

    pub fn set_occupied(&mut self, id: &Uuid, occupied: bool) -> bool {
        if let Some(v) = self.works.get_mut(id) {
            *v = occupied;
        }
        return false;
    }

    pub fn remove_work(&mut self, id: &Uuid) -> Option<bool> {
        return self.works.remove(id);
    }

    pub fn push_work(&mut self, work: &GlobalWork) {
        self.works.insert(work.id, false);
    }
}

// NOTE: Tag that is used to distinguish worker pawns.
#[derive(Component)]
pub struct Worker {
    accessible: Vec<GlobalWork>,
    inaccessible: Vec<GlobalWork>,
}

// NOTE: Event that is used to send a global work to worker pawns.
pub struct RegisterGlobalWorkEvent {
    work: GlobalWork
}

impl RegisterGlobalWorkEvent {
    pub fn new(work: GlobalWork) -> Self {
        Self {
            work,
        }
    }
}

pub fn register_global_work_event(
    mut query: Query<(&Position, &mut Worker)>,
    mut gw_validator: ResMut<GlobalWorkValidator>,
    mut event_reader: EventReader<RegisterGlobalWorkEvent>,
    world: Res<world::World>,
) {
    for e in event_reader.iter() {
        for (position, mut worker) in &mut query {
            let result = find_best_path_to_target(
                position, &e.work.position, &world
            );

            match result {
                Some(_) => {
                    worker.accessible.push(e.work.clone());
                },
                None => {
                    worker.inaccessible.push(e.work.clone());
                }
            }

            gw_validator.push_work(&e.work);
        }
    }
}

// NOTE: Event that is sent when a worker mines a tile.
#[derive(Clone, Copy)]
pub struct MineTileEvent(pub Position);

// NOTE: Spawns a worker pawn, every worker
//       pawn is currently owned by the player.
pub fn spawn_worker_pawn(
    commands: &mut Commands,
    world: &mut world::World,
    tileset: &tileset::Tileset,
    position: (usize, usize),
) -> Entity {
    let e = spawn_default_pawn_with_alignment(
        commands, 
        world, 
        tileset, 
        DEFAULT_WORKER_PAWN_GLYPH, 
        position, 
        Alignment::Player
    );

    commands.entity(e).insert(Worker {
        accessible: vec![],
        inaccessible: vec![],
    });

    return e;
}

// NOTE: Setup the `GlobalWorkPool` resource.
fn setup_global_work_pool(mut commands: Commands) {
    commands.insert_resource(GlobalWorkValidator::default());
}

// NOTE: Find the best position around the target
// TODO: Optimisse this algorithm, to find better paths with higher efficiency.
fn find_best_path_to_target(
    position: &Position,
    target: &Position,
    world: &world::World,
) -> Option<MoveTask> {
    let mut best_cost = u32::MAX;

    let mut best_path = vec![];
    let mut best_pos = Position::new(-1, -1);

    for y in -1..=1_i32 {
        for x in -1..=1_i32 {
            if x == 0 && y == 0 {
                continue;
            }

            let target = Position::new(
                target.x + x,
                target.y + y,
            );

            // NOTE: If that tile is a solid one or if it is already
            //       occupied don't bother trying to find a path.
            if world.is_solid_tile(target.into()) || 
                world.get_entity(target.into()).is_some() {
                continue;
            }

            let result = core::pawn_find_path(
                *position, 
                target, 
                world
            );

            match result {
                Some((path, cost)) => {
                    if cost < best_cost {
                        best_cost = cost;

                        best_path = path;
                        best_pos = target;
                    }
                },
                None => {}
            }
        }
    }

    if best_path.is_empty() {
        return None;
    } else {
        // NOTE: Remove the initial position from the path.
        best_path.remove(0);

        return Some(MoveTask {
            path: VecDeque::from(best_path),
            target: best_pos
        })
    }
}

// NOTE: Returns the distance between a pawn an a global task.
fn distance_to_work(
    position: &Position,
    work: &GlobalWork,
    world: &world::World
) -> f32 {
    return Vec2::new(
        (work.position.x - position.x) as f32, 
        (work.position.y - position.y) as f32
    ).length();
}

// NOTE: Behaviour code which determines what workers do under certain
//       circumstances, worker capabilities (Not all implemented):
//       [X] - Mine tiles.
//       [_] - Construct buildings.
//       [_] - Repair buildings.
fn worker_behaviour(
    mut query: Query<(Entity, &Position, &mut TaskQueue, &mut Worker)>,
    mut gw_validator: ResMut<GlobalWorkValidator>,
    world: Res<world::World>,
) {
    for (e, position, mut tq, mut worker) in &mut query {
        let active = if let Task::None = tq.active { 
            false 
        } else { 
            true 
        };

        if !active && tq.queue.is_empty() {
            // NOTE: Find the nearest unoccupied task.
            let mut index = -1;
            let mut close = f32::MAX;

            // NOTE: Exhausted works
            let mut exhausted = vec![];

            for (i, work) in worker.accessible.iter().enumerate() {
                // NOTE: Validate the work.
                match gw_validator.validate(&work.id) {
                    Some(occupied) => {
                        // NOTE: Skip the work if it's alrady occupied.
                        if *occupied {
                            continue;
                        }
                    },
                    // NOTE: Handle the case of work no longer existing.
                    None => {
                        exhausted.push(i);
                    }
                }

                // NOTE: Find the distance to the work.
                let dist = distance_to_work(position, &work, &world);

                // NOTE: If that work is inaccessible skip it.
                if dist < 0.0 {
                    continue;
                }

                if dist < close {
                    close = dist;
                    index = i as i32;
                }
            }

            // NOTE: If a work is available task it.
            if index != -1 {
                let work = worker.accessible.get_mut(index as usize).unwrap();

                // NOTE: Find the best nearest position around work.
                let result = find_best_path_to_target(position, &work.position, &world);

                // NOTE: Send the required taks to worker.
                if let Some(mt) = result {
                    tq.queue.push_back(Task::Move(mt));
                    tq.queue.push_back(work.task.clone());
                } else {
                    continue;
                }

                // NOTE: Mark the work as occupied.
                gw_validator.set_occupied(&work.id, true);
            }

            // NOTE: Remove exhausted works from the work list
            for i in exhausted {
                worker.accessible.remove(i);
            }
        }
    }
}

// NOTE: Event that is used to mine a tile, apply
//       required changes to world and tile entity.
fn mine_tile_event(
    mut commands: Commands,
    mut world: ResMut<world::World>,
    mut event_reader: EventReader<MineTileEvent>,
    mut tiles: Query<(&Position, &tile::Resource, &mut tile::Tile)>,
    mut player_resources: ResMut<resource::PlayerResources>,
    indicators: Query<(Entity, &Position), With<order::MineOrderIndicator>>,
) {
    // NOTE: If there is no event present return.
    if event_reader.is_empty() {
        return;
    }

    // NOTE: Process events and store their positions.
    let mut targets = vec![];

    for e in event_reader.iter() {
        let target = e.0;

        // NOTE: Push position to vector
        targets.push(target);

        // NOTE: Change the world data for the target tile
        let tile = world.get_tile_mut(target.into());
        
        tile.state = tile::TileState::Empty;

        // NOTE: Increase the player's resource count acording to the tile's material
        let mut res =  &mut player_resources.resources[tile.resource.material as usize];

        res.quantity += 1;

        // NOTE: Change the grid data for the target tile
        world.grid.remove_vertex(target.into());
    }

    // NOTE: Despawn the indicator entities with target positions.
    let mut marked = vec![];

    for (entity, position) in &indicators {
        if !targets.contains(position) {
            continue;
        }

        marked.push(entity);
    }

    for e in marked {
        commands.entity(e).despawn();
    }

    // NOTE: Change tiles with the same positions in the array.
    for (position, res, mut tile) in &mut tiles {
        let mut index = -1;
        for (i, target) in targets.iter().enumerate() {
            if position == target {
                index = i as i32;
                break;
            }
        }

        if index == -1 {
            continue;
        }

        // NOTE: Remove target position from the vector
        targets.remove(index as usize);

        // NOTE: Change `TileState` to empty.
        tile.state = tile::TileState::Empty;
    }
}