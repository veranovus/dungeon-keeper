use log::error;
use bevy::prelude::*;

use crate::{
    globals, 
    util::cursor, 
    world, 
    ui::inspector,
    pawn::prelude::*,
};

pub mod prelude {
    pub use super::{
        SelectionID,
        SelectionType,
        SelectionState,
        SelectionResult,
        SelectionEvent,
        SelectionPrepareEvent,
        DEFAULT_SELECTION_ID,
        DEFAULT_SELECTION_COLOR,
    };
}

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SelectionEvent>()
            .add_event::<SelectionPrepareEvent>()
            .add_startup_system_to_stage(StartupStage::PostStartup, setup_selection)
            .add_system_to_stage(CoreStage::PreUpdate, control_selection)
            .add_system_to_stage(CoreStage::Update, update_selection)
            .add_system_to_stage(CoreStage::PostUpdate, selection_prepare_event);
    }
}

// NOTE: Required change in cursor position for a drag-selection to reegister.
pub const DRAG_TRESHOLD: f32 = 5.0;

// NOTE: Path to the selection image in the assets folder.
pub const SELECTION_IMAGE_PATH: &str = "select.png";

// NOTE: Default state of the selection, currently active.
pub const DEFAULT_SELECTION_STATE: SelectionState = SelectionState::Capture;

// NOTE: Color for the default kind of selection
pub const DEFAULT_SELECTION_COLOR: Color = Color::rgba(0.1, 1.0, 0.1, 0.05);

// NOTE: ID for the default kind of selection.
pub const DEFAULT_SELECTION_ID: SelectionID = SelectionID::Entity;

#[allow(dead_code)]
// NOTE: ID that is used to distinguish between
//       different kind of selection events.
#[derive(Clone, Copy)]
pub enum SelectionID {
    Invalid,
    Entity,
    Mine,
    Build,
}

#[allow(dead_code)]
// NOTE: Determines whether current selection is subtractive or additive.
#[derive(Clone, Copy)]
pub enum SelectionType {
    Possitive,
    Negative,
}

#[allow(dead_code)]
// NOTE: State that is used to decide whether
//       selection is enabled or not.
#[derive(Resource, Clone, Copy)]
pub enum SelectionState {
    Ignore,
    Capture
}

#[allow(dead_code)]
// NOTE: Utility functions to make the code
//       more readable and easy to understand.
impl SelectionState {
    pub fn set(&mut self, state: SelectionState) {
        *self = state;
    }

    pub fn get(&self) -> Self {
        *self
    } 
}

// NOTE: Contain's all the parameters of the current selection,
//       only one entity should have this component.
#[derive(Component)]
struct Selection {
    started: bool,
    drag: bool,
    snap: bool,
    colors: [Color; 2],
    selection_id: SelectionID,
    selection_type: SelectionType,
    start_pos: Vec2,
    final_pos: Vec2,
}

impl Default for Selection {
    fn default() -> Self {
        Self {
            started: false,
            drag: false,
            snap: false,
            colors: [
                DEFAULT_SELECTION_COLOR,
                DEFAULT_SELECTION_COLOR
            ],
            selection_id: DEFAULT_SELECTION_ID,
            selection_type: SelectionType::Possitive,
            start_pos: Vec2::ZERO,
            final_pos: Vec2::ZERO,
        }
    }
}

impl Selection {
    // NOTE: Resets the `Selection` parameter's to default values.
    fn reset(&mut self) {
        *self = Self::default();
    }
}

#[allow(dead_code)]
// NOTE: Result of a selection event.
pub enum SelectionResult {
    Default(Vec2, Vec2),
    Snap(Position, Position),
}

// NOTE: Event that is send at the end of every selection.
pub struct SelectionEvent {
    pub result: SelectionResult, 
    pub selection_id: SelectionID,
    pub selection_type: SelectionType,
}

// NOTE: Event that is used to change the properties of the selection.
pub struct SelectionPrepareEvent {
    pub selection_id: SelectionID,
    pub colors: [Color; 2],
    pub snap: bool,
}

impl Default for SelectionPrepareEvent {
    // NOTE: Default parameters for a selection event.
    fn default() -> Self {
        Self {
            selection_id: DEFAULT_SELECTION_ID,
            colors: [
                DEFAULT_SELECTION_COLOR,
                DEFAULT_SELECTION_COLOR
            ],
            snap: false,
        }
    }
}

// NOTE: Sets up the selection entity, and `SelectionState` resource.
fn setup_selection(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // NOTE: Setup selection.
    let sprite = Sprite {
        anchor: globals::DEFAULT_SPRITE_ANCHOR,
        color: DEFAULT_SELECTION_COLOR,
        ..Default::default()
    };

    commands.spawn(SpriteBundle {
        sprite,
        texture: asset_server.load(SELECTION_IMAGE_PATH),
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, globals::SPRITE_ORDER_USER),
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(Selection::default());

    // NOTE: Setup the `SelectionState`.
    commands.insert_resource(DEFAULT_SELECTION_STATE);
}

// NOTE: Snaps selection to grid positions, returns the snapped version.
fn snap_selection(selection: &Selection) -> (Vec2, Vec2) {
    // NOTE: Convert world positions to tile positions. 
    let mut start = world::normalize_to_world_coordinates(
        selection.start_pos
    );

    let mut r#final = world::normalize_to_world_coordinates(
        selection.final_pos
    );

    // NOTE: Apply shifting to either the start or the
    //       final positions for an accurate selection.
    if r#final.0 >= start.0 {
        r#final.0 += 1;
    } else {
        start.0 += 1;
    }
    
    if r#final.1 >= start.1 {
        r#final.1 += 1;
    } else {
        start.1 += 1;
    }

    // NOTE: Convert snapped and shifted positions back
    //       to the world positions.
    let start = Vec2::new(
        start.0 as f32 * globals::SPRITE_SIZE,
        start.1 as f32 * globals::SPRITE_SIZE
    );

    let r#final = Vec2::new(
        r#final.0 as f32 * globals::SPRITE_SIZE,
        r#final.1 as f32 * globals::SPRITE_SIZE
    );

    return (start, r#final)
}

// NOTE: Responds to `SelectionPrepareEvent`, used to
//       setup selection for different kinds of use cases.
fn selection_prepare_event(
    mut query: Query<(&mut Selection, &mut Sprite)>,
    mut event_reader: EventReader<SelectionPrepareEvent>,
) {
    let (mut selection, mut sprite) = if let Ok(s) = query.get_single_mut() { s } else {
        error!("More than one entity has the trait `Selection`.");
        panic!();
    };

    for e in event_reader.iter() {
        // NOTE: Reset the selection before applying the changes.
        selection.reset();

        selection.selection_id = e.selection_id;
        selection.colors = e.colors;
        selection.snap = e.snap;

        // NOTE: Apply color change to sprite.
        sprite.color = selection.colors[0];
    }
}

fn control_selection(
    mut selection: Query<(&mut Selection, &mut Sprite)>,
    mut event_writer: EventWriter<SelectionEvent>,
    selection_state: Res<SelectionState>,
    cursor_pos: Res<cursor::CursorPos>,
    buttons: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
) {
    // NOTE: Flag that is used to indicate wheter cursor is over ui panel or not.
    let over_ui = cursor_pos.screen.x < inspector::INSPECTOR_PANEL_SIZE;

    if let SelectionState::Ignore = selection_state.get() {
        return;
    }

    let (mut selection, mut sprite) = if let Ok(s) = selection.get_single_mut() { s } else {
        error!("More than one entity has the trait `Selection`.");
        panic!();
    };

    // NOTE: Set the selection type.
    if keys.pressed(KeyCode::LShift) {
        selection.selection_type = SelectionType::Negative;
        sprite.color = selection.colors[1];
    } else {
        selection.selection_type = SelectionType::Possitive;
        sprite.color = selection.colors[0];
    }


    // NOTE: If the cursor is not over the UI handle the selection changes.
    if !over_ui {
        if buttons.just_pressed(MouseButton::Left) {
            // NOTE: Set the selection flag to true to start the selection.
            selection.started = true;

            // NOTE: Set the both start and final positions
            //       to cursor's world position.
            selection.start_pos = cursor_pos.world;
            selection.final_pos = cursor_pos.world;
        }
    
        // NOTE: If difference in cursor's position is big enough start the drag-selection.
        if buttons.pressed(MouseButton::Left) && selection.started {
            let diff = (cursor_pos.world - selection.start_pos).length();
    
            if (diff > DRAG_TRESHOLD) || selection.snap {
                selection.drag = true;
            }
        }
    }

    // NOTE: If the left button is released send the selection event.
    if buttons.just_released(MouseButton::Left) && selection.started {
        // NOTE: Set `started` and `drag` to false.
        selection.drag = false;
        selection.started = false;

        // NOTE: Apply snapping to selection if snap is enabled.
        if selection.snap {
            let (s, f) = snap_selection(&selection);

            selection.start_pos = s;
            selection.final_pos = f;
        }

        let mut position = selection.start_pos;

        let size = Vec2::new(
            selection.final_pos.x - selection.start_pos.x,
            selection.final_pos.y - selection.start_pos.y,
        );

        // NOTE: Shift start position to make further calculations easier,
        //       actual selection area and position is not changed.
        if selection.final_pos.x < selection.start_pos.x {
            position.x += size.x;
        }
        if selection.final_pos.y < selection.start_pos.y {
            position.y += size.y;
        }

        // NOTE: Calcualte the result depending on the snap flag.
        let result = if selection.snap {
            SelectionResult::Snap(
                world::normalize_to_world_coordinates(position).into(), 
                world::normalize_to_world_coordinates(size.abs()).into()
            )
        } else {
            SelectionResult::Default(position, size.abs())
        };

        event_writer.send(SelectionEvent {
            result,
            selection_id: selection.selection_id,
            selection_type: selection.selection_type,
        });
    }
}

// NOTE: Update the selection's position and size acording
//       to the cursor's position if it has started.
fn update_selection(
    mut query: Query<(&mut Selection, &mut Transform, &mut Visibility)>,
    selection_state: Res<SelectionState>,
    cursor_pos: Res<cursor::CursorPos>,
) {
    if let SelectionState::Ignore = selection_state.get() {
        return;
    }

    let (mut selection, mut transform, mut visibility) = if let Ok(t) = query.get_single_mut() {t} else {
        error!("More than one entity has the trait `Selection`.");
        panic!();
    };

    // NOTE: If a drag-selection is not present or if
    //       the selection hasn't started yet just return.
    if !selection.drag || !selection.started {
        visibility.is_visible = false;
        return;
    }

    // NOTE: If a drag-selection is present make the entity visible.
    visibility.is_visible = true;

    // NOTE: Calculate the change in size and
    //       position depending on the snap flag.
    if selection.snap {
        let (start, r#final) = snap_selection(&selection);

        selection.final_pos = cursor_pos.world;

        transform.translation.x = start.x;
        transform.translation.y = start.y;

        transform.scale.x = r#final.x - start.x;
        transform.scale.y = r#final.y - start.y;
    } else {
        selection.final_pos = cursor_pos.world;

        transform.translation.x = selection.start_pos.x;
        transform.translation.y = selection.start_pos.y;

        transform.scale.x = selection.final_pos.x - selection.start_pos.x;
        transform.scale.y = selection.final_pos.y - selection.start_pos.y;
    }
}
