use bevy::prelude::*;
use std::collections::VecDeque;
use log::{info, error};

use crate::world;
use super::{core::{prelude::*, self}, worker};

pub mod prelude {
    pub use super::{
        Task,
        TaskQueue,
        MoveTask,
    };
}

// NOTE: Holds the path and location of the target tile.
#[derive(Debug, Clone)]
pub struct MoveTask {
    pub path: VecDeque<Position>,
    pub target: Position,
}

// NOTE: A task is basically what a pawn is going to do that turn.
//       This enum holds the every possible task for a pawn.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Task {
    None,
    Move(MoveTask),
    Attack(Entity),
    Mine((Position, worker::GlobalWorkID)),
}

#[allow(dead_code)]
impl Task {
    pub fn name(&self) -> String {
        match self {
            Task::None => "None",
            Task::Move(_) => "Move",
            Task::Attack(_) => "Attack",
            Task::Mine(_) => "Mine",
        }.to_string()
    } 
}

// NOTE: A component that holds the tasks
//       queue and active task of a pawn.
#[derive(Component)]
pub struct TaskQueue {
    pub queue: VecDeque<Task>,
    pub active: Task,
}

impl TaskQueue {
    // NOTE: Sets active task to the next task in the Task Queue,
    //       sets it to `Task::None` if queue is empty.
    pub fn next_tast(&mut self) {
        if let Some(task) = self.queue.pop_front() {
            self.active = task;
        } else {
            self.active = Task::None;
        }
    }
}

// NOTE: Determines the turn logic for every pawn.
pub fn pawn_act_turn(
    entity: Entity,
    task_queue: &mut TaskQueue,
    transform: &mut Transform,
    position: &mut Position,
    world: &mut world::World,
    gw_validator: &mut worker::GlobalWorkValidator,
    mine_tile_er: &mut EventWriter<worker::MineTileEvent>,
) {
    match &mut task_queue.active {
        Task::None => {}
        Task::Move(move_task) => {
            if let Some (target) = move_task.path.pop_front() {
                let result = core::move_pawn( 
                    target.into(),
                    entity,
                    transform, 
                    position, 
                    world
                );
    
                // NOTE: If next tile on the path is invalid, try to find a new path.
                if result == false {
                    info!("Finding a new path for the target location.");

                    let result = core::pawn_find_path(
                        *position, 
                        move_task.target, 
                        world
                    );

                    match result {
                        // NOTE: If a new path is found push a new movement task to
                        //       the front of the queue, and skip the current one.
                        Some((mut path, _)) => {
                            path.remove(0);

                            task_queue.queue.push_front(Task::Move(MoveTask {
                                path: VecDeque::from(path),
                                target: move_task.target,
                            }));
                        },
                        // NOTE: Otherwise just skip to the next
                        //       task, without pushing a new one.
                        None => {
                            info!("No possible path found for the target, move task is skipped.");
                        }
                    }
                } else {
                    // NOTE: If pawn was able to move, return
                    //       to keep pawn in the move task.
                    return;
                }
            }
        }
        Task::Attack(_) => {},
        Task::Mine((target, id)) => {
            // NOTE: Get the current work from the pool
            let result = gw_validator.validate(id);

            if let Some(_) = result {
                // NOTE: Calculate the distance to the target tile.
                let dist = Vec2::new(
                    (target.x - position.x).abs() as f32,
                    (target.y - position.y).abs() as f32,
                ).length();

                // NOTE: If the pawn failed to reach to the target tile
                //       for some reason, set work to unoccupied again.
                if dist > f32::sqrt(2.0) {
                    gw_validator.set_occupied(id, false);

                    info!("Failed to reach to the current work, mine task is skipped.");
                } else {
                    // NOTE: Otherwise remove the work from the `GlobalWorkValidator`.
                    let result = gw_validator.remove_work(id);

                    if result.is_none() {
                        error!("Failed to remove work from the `GlobalWorkValidator`, this should have never happened.");
                        panic!();
                    }

                    // NOTE: Send a `MineTileEvent` with given target position.
                    mine_tile_er.send(worker::MineTileEvent(*target));
                }
            } else {
                info!("Failed to validate work from the `GlobalWorkValidator`, mine task is skipped.");
            }
        },
    }

    task_queue.next_tast();
}