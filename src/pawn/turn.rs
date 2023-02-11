use bevy::prelude::*;
use std::collections::VecDeque;
use log::info;

use crate::world;
use super::core;

pub mod prelude {
    pub use super::{
        Task,
        TaskQueue,
    };
}

// NOTE: A task is basically what a pawn is going to do that turn.
//       This enum holds the every possible task for a pawn.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum Task {
    None,
    Move((usize, usize)),
    Attack(Entity),
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
    position: &mut core::Position,
    world: &mut world::World,
) {
    match &mut task_queue.active {
        Task::None => {}
        Task::Move(target) => {
            let result = core::move_pawn( 
                *target,
                entity,
                transform, 
                position, 
                world
            );

            if result == false {
                info!("Move action skipped due to target being an invalid tile.");
            }
        }
        Task::Attack(_) => {}
    }

    task_queue.next_tast();
}