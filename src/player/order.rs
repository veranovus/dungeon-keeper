use std::collections::VecDeque;

use bevy::prelude::*;

use log::info;

use crate::{
    pawn::{prelude::*, core, worker}, 
    util::cursor, world,
    player::selection::prelude::*, globals, tileset,
};

pub struct OrderPlugin;

#[allow(unused_variables)]
impl Plugin for OrderPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PreUpdate, select_pawns)
            .add_system_to_stage(CoreStage::PreUpdate, move_order)
            .add_system_to_stage(CoreStage::PreUpdate, mine_order)
            .add_system_to_stage(CoreStage::PreUpdate, prepare_selection);
    }
}

// NOTE: Tag that is used to detect mine order indicators.
#[derive(Component)]
pub struct MineOrderIndicator;

// NOTE: Color of the mine order indicator entities.
const MINE_ORDER_INDICATOR_COLOR: Color = Color::rgba(1.0, 1.0, 0.1, 0.15);

// NOTE: Glyph that will be used for mine order indicators.
const MINE_ORDER_INDICATOR_GLYPH: usize = 11 * 16;

// NOTE: Colors for the selection that will be used for mine order.
const MINE_ORDER_SELECTION_COLORS: [Color; 2] = [
    Color::rgba(1.0, 1.0, 0.1, 0.05),
    Color::rgba(1.0, 0.1, 0.1, 0.05),
];

// NOTE: Depending on the player's input prepares the selection id, 
//       and other properties of selection.
fn prepare_selection(
    mut event_writer: EventWriter<SelectionPrepareEvent>,
    keys: Res<Input<KeyCode>>,
) {
    if keys.just_released(KeyCode::M) {
        event_writer.send(SelectionPrepareEvent {
            selection_id: SelectionID::Mine,
            colors: MINE_ORDER_SELECTION_COLORS,
            snap: true,
        })
    }
    
    if keys.just_pressed(KeyCode::Escape) {
        event_writer.send(SelectionPrepareEvent::default());
    }
}

// NOTE: Selects pawns under cursor or in the selection area.
fn select_pawns(
    mut query: Query<(&Transform, &mut Selectable), With<Pawn>>,
    mut event_reader: EventReader<SelectionEvent>,
) {
    for e in event_reader.iter() {
        // NOTE: Check if the event is sent to this function.
        let valid = match e.selection_id {
            SelectionID::Entity => true,
            _ => false,
        };

        if valid {
            // NOTE: Check if the result is in the right format.
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
            // NOTE: Clear the active task.
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

                    // NOTE: Convert path into a VecDeque from Vec, and
                    //       push the task to the queue as the first task.
                    task_queue.queue.push_front(Task::Move(MoveTask {
                        path: VecDeque::from(path),
                        target,
                    }));
                },
            }
        }
    }
}

// NOTE: Marks the tiles under cursor or in the selection's are to be mined.
fn mine_order(
    mut commands: Commands,
    mut world: ResMut<world::World>,
    mut event_reader: EventReader<SelectionEvent>,
    mut global_work_pool: ResMut<worker::GlobalWorkPool>,
    query: Query<(Entity, &Position), With<MineOrderIndicator>>,
    tileset: Res<tileset::Tileset>,
) {
    for e in event_reader.iter() {
        // NOTE: Check if the event is sent to this function.
        let valid = match e.selection_id {
            SelectionID::Mine => true,
            _ => false,
        };

        if valid {
            // NOTE: Check if the result is in the right format.
            let (position, size) = match e.result {
                SelectionResult::Snap(position, size) => {
                    (position, size)
                },
                _ => {
                    continue;
                }
            };

            // NOTE: Calculate the current tile positions,
            //       and save them into a vector.
            let mut positions: Vec<(usize, usize)> = vec![];

            for y in 0..size.y {
                for x in 0..size.x {
                    let position: (usize, usize) = (
                        (position.x + x) as usize,
                        (position.y + y) as usize,
                    );

                    positions.push(position);
                }
            }

            // NOTE: Depending on the selection type, either
            //       delete or create new indicators.
            match e.selection_type {
                SelectionType::Possitive => {
                    for position in &positions {
                        if world.is_solid_tile(*position) && !world.get_tile(*position).marked{
                            // NOTE: Setup the mine-task shadow entity..
                            let e = tileset::spawn_sprite_from_tileset(
                                &mut commands,
                                &tileset,
                                MINE_ORDER_INDICATOR_GLYPH,
                                Vec3::new(
                                    position.0 as f32 * globals::SPRITE_SIZE,
                                    position.1 as f32 * globals::SPRITE_SIZE,
                                    globals::SPRITE_ORDER_USER,
                                ),
                                Vec3::new(globals::SPRITE_SCALE, globals::SPRITE_SCALE, 1.0),
                                MINE_ORDER_INDICATOR_COLOR,
                            );
    
                            commands.entity(e)
                                .insert(Position::from(*position))
                                .insert(MineOrderIndicator);
    
                            // NOTE: Change the tile's marked flag to true.
                            let mut tile = world.get_tile_mut(*position);

                            tile.marked = true;

                            // NOTE: Push the work to the global work pool
                            let id = uuid::Uuid::new_v4();

                            global_work_pool.works.push(
                                worker::GlobalWork::new(
                                    Task::Mine(((*position).into(), id)),
                                    id,
                                )
                            );
                        }
                    }
                },
                SelectionType::Negative => {
                    for (entity, tile) in &query {
                        let tile: (usize, usize) = (*tile).into();

                        // NOTE: If entity's position is selected to
                        //       be removed, remove the entity.
                        for position in &positions {
                            if tile.0 == position.0 && tile.1 == position.1 {
                                // NOTE: Despawn the entity.
                                commands.entity(entity).despawn_recursive();

                                // NOTE: Change the tile's marked flag to false.    
                                let mut tile = world.get_tile_mut(*position);
                            
                                tile.marked = false;

                                // NOTE: Remove the work from the global work queue.
                                let mut remove = vec![];

                                for (i, work) in global_work_pool.works.iter().enumerate() {
                                    if let Task::Mine((target, _)) = work.task {
                                        if (target.x as usize == position.0) && 
                                           (target.y as usize == position.1) {
                                            
                                            if !remove.contains(&i) {
                                                remove.push(i);
                                            }
                                        }
                                    }
                                }

                                for i in remove {
                                    global_work_pool.works.remove(i);
                                }

                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}