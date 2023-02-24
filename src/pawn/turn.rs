use bevy::prelude::*;
use std::collections::VecDeque;
use log::info;

use crate::world;
use super::core::{prelude::*, self};

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
    Mine(Position),
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
) {
    match &mut task_queue.active {
        Task::None => {}
        Task::Move(move_task) => {
            match move_task.path.pop_front() {
                Some(v) => {
                    let result = core::move_pawn( 
                        v.into(),
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
                            Some((mut path, _cost)) => {
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
                },
                // NOTE: If there is no tile left in the
                //       path, get to the next task.
                None => {}
            };
        }
        Task::Attack(_) => {},
        Task::Mine(_target) => {},
    }

    task_queue.next_tast();
}