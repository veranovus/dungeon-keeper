use bevy::prelude::*;
use log::error;

use crate::globals::{WINDOW_SIZE, SPRITE_SIZE, MAP_SIZE};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, setup_camera)
            .add_system_to_stage(CoreStage::PreUpdate, move_camera);
    }
}

// NOTE: Camera speed in tiles per second.
const CAMERA_SPEED: i32 = 15;

// NOTE: Only one camers should possess
//       this component at any given time.
#[derive(Component)]
pub struct MainCamera;

// NOTE: Setup the Main Camera
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default()).insert(MainCamera);
}

// NOTE: Contols and restricts the camera position.
fn move_camera(
    mut query: Query<&mut Transform, With<MainCamera>>,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let mut transform = if let Ok(t) = query.get_single_mut() { t } else {
        error!("More than one camera has the trait `MainCamera`.");
        panic!();
    };

    let delta = time.delta_seconds();
    let speed = SPRITE_SIZE * CAMERA_SPEED as f32;

    let mut vector = Vec2::ZERO;
    let mut moved = false;

    // NOTE: Move camera acording to the user input.
    if keys.pressed(KeyCode::A) {
        vector.x = -1.0;
        moved = true;
    } else if keys.pressed(KeyCode::D) {
        vector.x = 1.0;
        moved = true;
    }
    if keys.pressed(KeyCode::S) {
        vector.y = -1.0;
        moved = true;
    } else if keys.pressed(KeyCode::W) {
        vector.y = 1.0;
        moved = true;
    }

    // NOTE: Normalize the direction to keep the speed consistent.
    if moved {
        vector = vector.normalize() * speed * delta;

        transform.translation.x += vector.x;
        transform.translation.y += vector.y;
    }

    // NOTE: Restrict game to the map boundaries.
    let width = MAP_SIZE.0 as f32 * SPRITE_SIZE;
    let height = MAP_SIZE.1 as f32 * SPRITE_SIZE;

    if transform.translation.x > width - WINDOW_SIZE.0 as f32 / 2.0 {
        transform.translation.x = width - WINDOW_SIZE.0 as f32 / 2.0;
    } else if transform.translation.x < WINDOW_SIZE.0 as f32 / 2.0 {
        transform.translation.x = WINDOW_SIZE.0 as f32 / 2.0;
    }

    if transform.translation.y > height - WINDOW_SIZE.1 as f32 / 2.0 {
        transform.translation.y = height - WINDOW_SIZE.1 as f32 / 2.0;
    } else if transform.translation.y < WINDOW_SIZE.1 as f32 / 2.0 {
        transform.translation.y = WINDOW_SIZE.1 as f32 / 2.0;
    }
}
