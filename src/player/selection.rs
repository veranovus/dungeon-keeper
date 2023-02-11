use bevy::prelude::*;
use log::error;

use crate::{globals, util::cursor, pawn::prelude::*};

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SelectEvent>()
        .add_startup_system_to_stage(StartupStage::PostStartup, setup_selection)
        .add_system_to_stage(CoreStage::PreUpdate, control_selection)
        .add_system_to_stage(CoreStage::Update, update_selection)
        .add_system_to_stage(CoreStage::PostUpdate, select_pawns);
    }
}

// NOTE: Treshold for required cursor movement
//       in order for a selection to register.
const SELECT_TRESHOLD: f32 = 5.0;

// NOTE: Image path for select texture.
const SELECT_IMAGE_PATH: &str = "select.png";

// NOTE: Color of the selection rectangle.
const SELECT_COLOR: Color = Color::rgba(0.1, 1.0, 0.1, 0.05);

// NOTE: Event that is used to select entities
//       with trait `Selectable` in given area.
pub struct SelectEvent {
    position: Vec2,
    size: Vec2,
}

// NOTE: Only one Selection Rectangle should be present in game.
#[derive(Component)]
pub struct SelectionRect {
    start: Vec2,
    finish: Vec2,
    select: bool,
}

// NOTE: Setup function for the selection entity
fn setup_selection(
    mut commands: Commands,
    assets_server: Res<AssetServer>,
) {
    let sprite = Sprite {
        anchor: globals::DEFAULT_SPRITE_ANCHOR,
        color: SELECT_COLOR,
        ..Default::default()
    };

    commands.spawn(SpriteBundle {
        sprite,
        texture: assets_server.load(SELECT_IMAGE_PATH),
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, globals::SPRITE_ORDER_USER),
            ..Default::default()
        },
        ..Default::default()
    }).insert(SelectionRect {
        start: Vec2::ZERO,
        finish: Vec2::ZERO,
        select: false,
    });
}

// NOTE: This function controls when the selection starts and finishes
fn control_selection(
    mut query: Query<&mut SelectionRect>,
    mut event_writer: EventWriter<SelectEvent>,
    buttons: Res<Input<MouseButton>>,
    cursor_pos: Res<cursor::CursorPos>
) {
    let mut selection = if let Ok(s) = query.get_single_mut() { s } else {
        error!("More than one entity has the trait `SelectionRect`.");
        panic!();
    };

    // NOTE: Get the cursor pos and cache it to resource,
    //       this doesn't trigger selection immediately.
    if buttons.just_pressed(MouseButton::Left) {
        selection.start = cursor_pos.world;
        return;
    }

    if buttons.pressed(MouseButton::Left) {
        // NOTE: Calculate the change in the cursor movement.
        let diff = (cursor_pos.world - selection.start).length();

        // NOTE: Compare the change in cursor movement and start
        //       the selection if it hasn't started yet.
        if diff >= SELECT_TRESHOLD && !selection.select {
            selection.select = true;
        }
    } else if selection.select {
        // NOTE: End the selection
        selection.select = false;

        // NOTE: Turn selection into a square
        //       anchored at its bottom left corner.
        let mut position = selection.start;
        let size = Vec2::new(
            selection.finish.x - selection.start.x,
            selection.finish.y - selection.start.y,
        );

        if selection.finish.x < selection.start.x {
            position.x += size.x;
        }
        if selection.finish.y < selection.start.y {
            position.y += size.y;
        }

        event_writer.send(SelectEvent {
            position,
            size: size.abs(),
        });
    }
}

// NOTE: Depending on the select flag, changes the
//       visibility, size and position of the entity.
fn update_selection(
    mut query: Query<(&mut SelectionRect, &mut Transform, &mut Visibility)>,
    cursor_pos: Res<cursor::CursorPos>,
) {
    let (mut selection, mut transform, mut visibility) = if let Ok(i) = query.get_single_mut() {i} else {
        error!("More than one entity has the trait `SelectionRect`.");
        panic!();
    };

    // NOTE: If not selecting just make the entity invisible.
    if !selection.select {
        visibility.is_visible = false;
        return;
    }

    // NOTE: Otherwise make it visible and calculate the change in size
    //       and position relative to the first and final cursor position.
    visibility.is_visible = true;
    selection.finish = cursor_pos.world;

    transform.translation.x = selection.start.x;
    transform.translation.y = selection.start.y;

    transform.scale.x = cursor_pos.world.x - selection.start.x;
    transform.scale.y = cursor_pos.world.y - selection.start.y;
}

// NOTE: Select entities with trait `Selectable` if they
//       are in given selection area of the event.
fn select_pawns(
    mut query: Query<(&Transform, &mut Selectable), With<Pawn>>,
    mut event_reader: EventReader<SelectEvent>,
    buttons: Res<Input<MouseButton>>,
) {
    let mut event = false;

    // NOTE: If there is an `SelectEvent`, select the pawns in event's area.
    for e in event_reader.iter() {
        for (transform, mut selectable) in &mut query {

            // NOTE: Check if entity is in the square in both x and y coordinates.
            let x = (transform.translation.x + globals::SPRITE_SIZE) > e.position.x
                && transform.translation.x < (e.position.x + e.size.x);
            let y = (transform.translation.y + globals::SPRITE_SIZE) > e.position.y
                && transform.translation.y < (e.position.y + e.size.y);

            // NOTE: If it is simpy mark the entity as
            //       selected and as not selected otherwise.
            if x && y {
                selectable.selected = true;
            } else {
                selectable.selected = false;
            }
        }

        event = true;
        break;
    }

    // NOTE: If there is no `SelectEvent` present and `MouseButton::Left`
    //       pressed deselect every pawn.
    if buttons.just_released(MouseButton::Left) && !event {
        for (_, mut selectable) in &mut query {
            selectable.selected = false;
        }
    }
}