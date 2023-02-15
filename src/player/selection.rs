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

pub const DRAG_TRESHOLD: f32 = 5.0;

pub const SELECTION_IMAGE_PATH: &str = "select.png";

pub const DEFAULT_SELECTION_COLOR: Color = Color::rgba(0.1, 1.0, 0.1, 0.05);

pub const DEFAULT_SELECTION_STATE: SelectionState = SelectionState::Capture;

pub const DEFAULT_SELECTION_ID: SelectionID = SelectionID::Entity;

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum SelectionID {
    Invalid,
    Entity,
    Mine,
    Build,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum SelectionType {
    Possitive,
    Negative,
}

#[allow(dead_code)]
#[derive(Resource, Clone, Copy)]
pub enum SelectionState {
    Ignore,
    Capture
}

#[allow(dead_code)]
impl SelectionState {
    pub fn set(&mut self, state: SelectionState) {
        *self = state;
    }

    pub fn get(&self) -> Self {
        *self
    } 
}

#[derive(Component)]
struct Selection {
    drag: bool,
    snap: bool,
    color: Color,
    selection_id: SelectionID,
    selection_type: SelectionType,
    start_pos: Vec2,
    final_pos: Vec2,
}

impl Default for Selection {
    fn default() -> Self {
        Self {
            drag: false,
            snap: false,
            color: DEFAULT_SELECTION_COLOR,
            selection_id: DEFAULT_SELECTION_ID,
            selection_type: SelectionType::Possitive,
            start_pos: Vec2::ZERO,
            final_pos: Vec2::ZERO,
        }
    }
}

impl Selection {
    fn reset(&mut self) {
        *self = Self::default();
    }
}

#[allow(dead_code)]
pub enum SelectionResult {
    Default(Vec2, Vec2),
    Snap(Position, Position),
}

pub struct SelectionEvent {
    pub result: SelectionResult, 
    pub selection_id: SelectionID,
    pub selection_type: SelectionType,
}

pub struct SelectionPrepareEvent {
    pub selection_id: SelectionID,
    pub result: SelectionResult,
    pub color: Color,
    pub snap: bool,
}

fn setup_selection(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let selection = Selection::default();

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
    }).insert(selection);

    commands.insert_resource(DEFAULT_SELECTION_STATE);
}

fn selection_prepare_event(
    mut query: Query<(&mut Selection, &mut Sprite)>,
    mut event_reader: EventReader<SelectionPrepareEvent>,
) {
    let (mut selection, mut sprite) = if let Ok(s) = query.get_single_mut() { s } else {
        error!("More than one entity has the trait `Selection`.");
        panic!();
    };

    for e in event_reader.iter() {
        selection.reset();

        selection.selection_id = e.selection_id;
        selection.color = e.color;
        selection.snap = e.snap;

        // TODO: Apply color change to sprite.
        sprite.color = selection.color;
    }
}

fn control_selection(
    mut selection: Query<&mut Selection>,
    mut event_writer: EventWriter<SelectionEvent>,
    selection_state: Res<SelectionState>,
    cursor_pos: Res<cursor::CursorPos>,
    buttons: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
) {
    if cursor_pos.screen.x < inspector::INSPECTOR_PANEL_SIZE {
        return;
    } else if let SelectionState::Ignore = selection_state.get() {
        return;
    }

    let mut selection = if let Ok(s) = selection.get_single_mut() { s } else {
        error!("More than one entity has the trait `Selection`.");
        panic!();
    };

    if keys.pressed(KeyCode::LShift) {
        selection.selection_type = SelectionType::Negative;
    } else {
        selection.selection_type = SelectionType::Possitive;
    }

    if buttons.just_pressed(MouseButton::Left) {
        selection.start_pos = cursor_pos.world;
    }

    if buttons.pressed(MouseButton::Left) {
        let diff = (cursor_pos.world - selection.start_pos).length();

        if (diff > DRAG_TRESHOLD) || selection.snap {
            selection.drag = true;
        }
    }

    if !buttons.pressed(MouseButton::Left) && selection.drag {
        selection.drag = false;

        if selection.snap {

            let start = world::normalize_to_world_coordinates(
                selection.start_pos
            );

            let mut r#final = world::normalize_to_world_coordinates(
                selection.final_pos
            );

            if r#final.0 >= start.0 {
                r#final.0 += 1;
            }
            if r#final.1 >= start.1 {
                r#final.1 += 1;
            }

            // TODO: Add event writing...
        } else {
            let mut position = selection.start_pos;

            let size = Vec2::new(
                selection.final_pos.x - selection.start_pos.x,
                selection.final_pos.y - selection.start_pos.y,
            );

            if selection.final_pos.x < selection.start_pos.x {
                position.x += size.x;
            }
            if selection.final_pos.y < selection.start_pos.y {
                position.y += size.y;
            }

            event_writer.send(SelectionEvent {
                result: SelectionResult::Default(position, size.abs()),
                selection_id: selection.selection_id,
                selection_type: selection.selection_type,
            });
        }
    }
}

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

    if !selection.drag {
        visibility.is_visible = false;
        return;
    }

    visibility.is_visible = true;

    if selection.snap {
        let start = world::normalize_to_world_coordinates(
            selection.start_pos
        );

        let mut r#final = world::normalize_to_world_coordinates(
            cursor_pos.world
        );

        if r#final.0 >= start.0 {
            r#final.0 += 1;
        }
        if r#final.1 >= start.1 {
            r#final.1 += 1;
        }

        let start = Vec2::new(
            start.0 as f32 * globals::SPRITE_SIZE,
            start.1 as f32 * globals::SPRITE_SIZE,
        );

        selection.final_pos = Vec2::new(
            r#final.0 as f32 * globals::SPRITE_SIZE,
            r#final.1 as f32 * globals::SPRITE_SIZE
        );

        transform.translation.x = start.x;
        transform.translation.y = start.y;

        transform.scale.x = selection.final_pos.x - start.x;
        transform.scale.y = selection.final_pos.y - start.y;
    } else {
        selection.final_pos = cursor_pos.world;

        transform.translation.x = selection.start_pos.x;
        transform.translation.y = selection.start_pos.y;

        transform.scale.x = selection.final_pos.x - selection.start_pos.x;
        transform.scale.y = selection.final_pos.y - selection.start_pos.y;
    }
}
