#![allow(unused_variables)]
use bevy::prelude::*;

pub struct WorkerPlugin;

impl Plugin for WorkerPlugin {
    fn build(&self, app: &mut App) {
        
    }
}

// NOTE: Tag that is used to distinguish worker pawns.
#[derive(Component)]
pub struct Worker;

