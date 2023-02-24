#![allow(unused_variables, unused_mut)]
use bevy::prelude::*;

use super::{
    turn::prelude::*
};

pub struct WorkerPlugin;

impl Plugin for WorkerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PostStartup, setup_global_work_pool)
            .add_system_to_stage(CoreStage::PreUpdate, worker_behaviour);
    }
}

#[derive(Resource)]
pub struct GlobalWorkPool {
    pub works: Vec<(Task, bool)>,
}

fn setup_global_work_pool(mut commands: Commands) {
    commands.insert_resource(GlobalWorkPool {
        works: vec![],
    });
}

/* 
    TODO:
    [X] - Move tile data, to the global world resources.
    [X] - Implement the global work queue.
    [ ] - Make a system to select the best work for a worker.
    [ ] - Refactor the system for work ownership.
    [ ] - Make it so worker's respond to the changes in the global 
          work pool, such as works being removed, etc.
    [ ] - Make a system which deletes a work from the queue when the tile is mined.
    [ ] - Worker system should assign a move task to the worker.
    [ ] - After the move task a worker should try to mine the tile, 
          make a system to check if the worker is next to tile. 
    [ ] - If a worker has no task in the queue and no active task push the same tasks.
*/

// NOTE: Tag that is used to distinguish worker pawns.
#[derive(Component)]
pub struct Worker;

fn worker_behaviour(
    mut query: Query<(Entity, &mut TaskQueue), With<Worker>>,
    mut global_work_pool: ResMut<GlobalWorkPool>,
) {
    for (e, mut tq) in &mut query {
        let active = if let Task::None = tq.active { 
            false 
        } else { 
            true 
        };

        if !active && tq.queue.is_empty() {
        }
    }
}