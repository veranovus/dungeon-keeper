use bevy::prelude::*;

use pathfinding::prelude::*;

use log::info;

use crate::{pawn::prelude::*, util::cursor, world};

use super::component::*;

pub struct OrderPlugin;

#[allow(unused_variables)]
impl Plugin for OrderPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PreUpdate, move_order);
    }
}

// NOTE: Sets active task of the selected entities (with `Player` tag)
//       to move action. This function clears the task queue.
fn move_order(
    mut query: Query<(&Selectable, &Position, &mut TaskQueue), (With<Player>, With<Pawn>)>,
    cursor_pos: Res<cursor::CursorPos>,
    buttons: Res<Input<MouseButton>>,
    world: Res<world::World>,
) {
    if !buttons.just_released(MouseButton::Right) {
        return;
    }

    for (selectable, position, mut task_queue) in &mut query {
        if selectable.selected {
            task_queue.queue.clear();

            let target: Position = world::normalize_to_world_coordinates(cursor_pos.world).into();

            // NOTE: Find the path from the pawn's position to target.
            let result = astar(
                position, 
                |p| p.successors(&world.grid),
                |p| p.distance(&target), 
                |p| *p == target
            );

            match result {
                None => {
                    info!("Ignored move order, no possible path for given location.");
                },
                Some((path, _cost)) => {
                    for point in path {
                        task_queue.queue.push_back(Task::Move(point.into()));
                    }

                    // NOTE: Remove the starting position since
                    //       pawn is already on that tile.
                    task_queue.queue.pop_front();
                },
            }
        }
    }
}