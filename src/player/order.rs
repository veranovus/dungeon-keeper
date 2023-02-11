use std::collections::VecDeque;

use bevy::prelude::*;

use log::info;

use crate::{pawn::{prelude::*, core}, util::cursor, world};

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

            let result = core::pawn_find_path(*position, target, &world);

            match result {
                None => {
                    info!("Ignored move order, no possible path for given location.");
                },
                Some((mut path, _cost)) => {
                    // NOTE: Remove the starting position since
                    //       pawn is already on that tile.
                    path.remove(0);

                    // NOTE: Convert path into a VecDeque from
                    //       Vec, and push the task to the pawn.
                    task_queue.queue.push_back(Task::Move(MoveTask {
                        path: VecDeque::from(path),
                        target,
                    }));
                },
            }
        }
    }
}