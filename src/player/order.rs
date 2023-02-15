use std::collections::VecDeque;

use bevy::prelude::*;

use log::info;

use crate::{
    pawn::{prelude::*, core}, 
    util::cursor, world,
    player::selection::prelude::*, globals,
};

pub struct OrderPlugin;

#[allow(unused_variables)]
impl Plugin for OrderPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PreUpdate, select_pawns)
            .add_system_to_stage(CoreStage::PreUpdate, move_order);
    }
}

fn select_pawns(
    mut query: Query<(&Transform, &mut Selectable), With<Pawn>>,
    mut event_reader: EventReader<SelectionEvent>,
) {
    for e in event_reader.iter() {
        let select = match e.selection_id {
            SelectionID::Entity => true,
            _ => false,
        };

        if select {
            let (position, size) = match e.result {
                SelectionResult::Default(position, size) => {
                    (position, size)
                },
                _ => {
                    continue;
                }
            };

            for (transform, mut selectable) in &mut query {

                // NOTE: Check if entity is in the square in both x and y coordinates.
                let x = (transform.translation.x + globals::SPRITE_SIZE) > position.x
                    && transform.translation.x < (position.x + size.x);
                let y = (transform.translation.y + globals::SPRITE_SIZE) > position.y
                    && transform.translation.y < (position.y + size.y);
    
                // NOTE: If it is simpy mark the entity as
                //       selected and as not selected otherwise.
                if x && y {
                    selectable.selected = true;
                } else {
                    selectable.selected = false;
                }
            }
        }
    }
}

// NOTE: Sets active task of the selected entities (with `Player` tag)
//       to move action. This function clears the task queue.
fn move_order(
    mut query: Query<(&Selectable, &Position, &Alignment, &mut TaskQueue), With<Pawn>>,
    cursor_pos: Res<cursor::CursorPos>,
    buttons: Res<Input<MouseButton>>,
    world: Res<world::World>,
) {
    if !buttons.just_released(MouseButton::Right) {
        return;
    }

    for (selectable, position, alignment, mut task_queue) in &mut query {
        let mut player_pawn = false;
        if let Alignment::Player = alignment {
            player_pawn = true;
        }

        if selectable.selected && player_pawn {
            // NOTE: Clear the task queue and the active task.
            task_queue.queue.clear();
            task_queue.active = Task::None;

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