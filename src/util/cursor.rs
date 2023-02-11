use bevy::prelude::*;

use log::error;

use crate::camera;

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, setup_cursor_pos_resource)
        .add_system_to_stage(CoreStage::PreUpdate, update_cursor_pos);
    }
}


// NOTE: Global resource for the cursor position
//       in world and screen coordinates.
#[derive(Resource)]
pub struct CursorPos {
    pub screen: Vec2,
    pub world: Vec2,
}

// NOTE: Sets up the cursor position resource.
fn setup_cursor_pos_resource(mut commands: Commands) {
    commands.insert_resource(CursorPos {
        screen: Vec2::ZERO,
        world: Vec2::ZERO,
    });
}

// NOTE: Calculates the cursor position in world coordinates
//       and updates the resource.
fn update_cursor_pos(
    mut cursor_pos: ResMut<CursorPos>,
    query: Query<(&Camera, &GlobalTransform), With<camera::MainCamera>>,
    windows: Res<Windows>,
) {
    let (camera, transform) = if let Ok(i) = query.get_single() {
        i
    } else {
        error!("More than one camera has the trait `MainCamera`.");
        panic!();
    };

    let window = if let Some(w) = windows.get_primary() {
        w
    } else {
        error!("Failed to get the primary window.");
        panic!();
    };

    // NOTE: Check if the cursor is inside the window and get its position
    if let Some(screen_pos) = window.cursor_position() {
        
        let window_size = Vec2::new(window.width() as f32, window.height() as f32);

        // NOTE: Convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

        // NOTE: Matrix for undoing the projection and camera transform
        let ndc_to_world = transform.compute_matrix() * camera.projection_matrix().inverse();

        // NOTE: Use it to convert ndc to world-space coordinates
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        // NOTE: Reduce it to a 2D value
        let world_pos: Vec2 = world_pos.truncate();

        // NOTE: Set the resource's values
        cursor_pos.world = world_pos;
        cursor_pos.screen = screen_pos;
    }
}