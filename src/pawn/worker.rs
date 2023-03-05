use std::collections::{VecDeque, HashMap};
use bevy::prelude::*;

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
            .add_event::<RemoveGlobalWorkEvent>()
            .add_event::<RegisterGlobalWorkEvent>()
            .add_startup_system_to_stage(StartupStage::PostStartup, setup_global_work_pool)
            .add_system_to_stage(CoreStage::PreUpdate, worker_behaviour)
            .add_system_to_stage(CoreStage::PreUpdate, check_inaccessible_works)
            .add_system_to_stage(CoreStage::PostUpdate, register_global_work_event)
            .add_system_to_stage(CoreStage::PostUpdate, remove_global_work_event)
            .add_system_to_stage(CoreStage::PostUpdate, mine_tile_event);
    }
}

// NOTE: Default glyph that is used for worker pawns.
pub const DEFAULT_WORKER_PAWN_GLYPH: usize = 1;

// NOTE: Amount of works to be recheched for every worker at once.
pub const MAX_WORK_RECHECK_COUNT: usize = 20;

// NOTE: Treshold for maximum number of accessible tasks for a pawn.
pub const MAX_ACCESSIBLE_WORK_TRESHOLD: usize = 300;

// NOTE: Work identifiers
pub const MINE_WORK_IDENTIFIER: &str = "m";

// NOTE: Identifier used to distinguish between works, first letter
//       is a code for the type and the numbers are the position.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlobalWorkID {
    id: String,
}

impl GlobalWorkID {
    pub fn new(work_identifier: &str, position: &Position) -> Self {
        Self {
            id: work_identifier.to_string() + 
                &position.x.to_string() + 
                &position.y.to_string(),
        }
    }
}

// NOTE: Any kind of work which should be executed by a worker,
//       every work has its unique id to identify it.
#[derive(Debug, Clone)]
pub struct GlobalWork {
    pub id: GlobalWorkID,
    pub task: Task,
    pub position: Position,
}

impl GlobalWork {
    pub fn new(task: Task, id: GlobalWorkID, position: Position) -> Self {
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
    pub works: HashMap<GlobalWorkID, bool>,
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
    pub fn validate(&mut self, id: &GlobalWorkID) -> Option<&bool> {
        return self.works.get(id);
    }

    pub fn set_occupied(&mut self, id: &GlobalWorkID, occupied: bool) -> bool {
        if let Some(v) = self.works.get_mut(id) {
            *v = occupied;
        }
        return false;
    }

    pub fn remove_work(&mut self, id: &GlobalWorkID) -> Option<bool> {
        return self.works.remove(id);
    }

    pub fn push_work(&mut self, work: &GlobalWork) {
        self.works.insert(work.id.clone(), false);
    }
}

// NOTE: Tag that is used to distinguish worker pawns.
#[derive(Component)]
pub struct Worker {
    pub accessible: Vec<GlobalWork>,
    pub inaccessible: Vec<GlobalWork>,
    iterator: usize,
}

// NOTE: Event that is used to send a global work to worker pawns.
pub struct RegisterGlobalWorkEvent {
    work: GlobalWork,
}

impl RegisterGlobalWorkEvent {
    pub fn new(work: GlobalWork) -> Self {
        Self {
            work,
        }
    }
}

// NOTE: Event that is used to remove a `GlobalWork` from the `GlobalWorkValidator`.
pub struct RemoveGlobalWorkEvent {
    id: GlobalWorkID,
}

impl RemoveGlobalWorkEvent {
    pub fn new(id: GlobalWorkID) -> Self {
        Self {
            id
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
        accessible: Vec::with_capacity(500),
        inaccessible: Vec::with_capacity(500),
        iterator: 0,
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
) -> f32 {
    return Vec2::new(
        (work.position.x - position.x) as f32, 
        (work.position.y - position.y) as f32
    ).length();
}

// NOTE: Behaviour code which determines what workers do under certain circumstances.
fn worker_behaviour(
    mut query: Query<(&Position, &mut TaskQueue, &mut Worker)>,
    mut gw_validator: ResMut<GlobalWorkValidator>,
    world: Res<world::World>,
) {
    for (position, mut tq, mut worker) in &mut query {
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
            let mut occupied = vec![];

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
                        occupied.push(work.id.clone());
                        continue;
                    }
                }

                // NOTE: Find the distance to the work.
                let dist = distance_to_work(position, &work);

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
            let mut iter = 0;
            while iter < worker.accessible.len() {
                if occupied.contains(&worker.accessible[iter].id) {
                    worker.accessible.remove(iter);
                } else {
                    iter += 1;
                }
            }
        }
    }
}

// NOTE: Checks every workers inaccessible works and
//       promotes them to accessible ones.
fn check_inaccessible_works(
    mut query: Query<(&Position, &mut Worker)>,
    mut gw_validator: ResMut<GlobalWorkValidator>,
    world: Res<world::World>,
) {
    for (position, mut worker) in &mut query {
        if worker.accessible.len() > MAX_ACCESSIBLE_WORK_TRESHOLD {
            continue;
        }

        let mut counter = 0;

        let mut promote = vec![];
        let mut invalid = vec![];

        // NOTE: Check if a work become accesssible, if so add it to promotion vector.
        //       Also checks if that work still exists, if so add it to removal vector.
        for i in worker.iterator..worker.inaccessible.len() {
            let work = &worker.inaccessible[i];

            if let None = gw_validator.validate(&work.id) {
                invalid.push(work.id.clone());
            } else {
                let result = find_best_path_to_target(
                    position, &work.position, &world
                );
        
                match result {
                    Some(_) => {
                        promote.push(work.id.clone());
                    },
                    None => {}
                }
            }

            counter +=1;
            if counter == MAX_WORK_RECHECK_COUNT {
                break;
            }
        }

        // NOTE: Increase the iterator.
        worker.iterator += counter;

        // NOTE: Promote the selected works.
        let mut iter = 0;
        while (iter < worker.iterator) && (iter < worker.inaccessible.len()) {
            if promote.contains(&worker.inaccessible[iter].id) {
                // NOTE: Push the work to the accessible list.
                let work = worker.inaccessible[iter].clone();
                worker.accessible.push(work);

                // NOTE: Remove the work from inaccessible list.
                worker.inaccessible.remove(iter);
            } else if invalid.contains(&worker.inaccessible[iter].id) {
                // NOTE: Remove the work if it doesn't exists anymore.
                worker.inaccessible.remove(iter);
            } else {
                iter += 1;
            }
        }

        // NOTE: Wrap the iterator to start if a cycle is complete.
        if worker.iterator >= worker.inaccessible.len() {
            worker.iterator = 0;
        }
    }
}

// NOTE: Adds works to every pawn's respective list depeding on
//       the accessibility also adds work to the `GlobalWorkValidator`.
// TODO: Send works to `GlobalWorkValidator` in batches
//       instead sending all at once.
fn register_global_work_event(
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
        }

        gw_validator.push_work(&e.work);
    }
}

// NOTE: Removes works from the `GlobalWorkValidator`.
// TODO: Add error checking here.
fn remove_global_work_event(
    mut gw_validator: ResMut<GlobalWorkValidator>,
    mut event_reader: EventReader<RemoveGlobalWorkEvent>,
) {
    for e in event_reader.iter() {
        gw_validator.remove_work(&e.id);
    }
}

// NOTE: Event that is used to mine a tile, apply
//       required changes to world and tile entity.
fn mine_tile_event(
    mut commands: Commands,
    mut world: ResMut<world::World>,
    mut event_reader: EventReader<MineTileEvent>,
    mut tiles: Query<(&Position, &mut tile::Tile)>,
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
    for (position, mut tile) in &mut tiles {
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