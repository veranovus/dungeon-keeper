use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod globals;
mod camera;
mod util;
mod tileset;
mod world;
mod turn_system;
mod pawn;
mod player;
mod ui;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        width: globals::WINDOW_SIZE.0 as f32,
                        height: globals::WINDOW_SIZE.1 as f32,
                        title: globals::WINDOW_TITLE.to_string(),
                        resizable: false,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .add_plugin(EguiPlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(util::UtilPlugin)
        .add_plugin(tileset::TilesetPlugin)
        .add_plugin(world::WorldPlugin)
        .add_plugin(turn_system::TurnSystemPlugin)
        .add_plugin(pawn::PawnPlugin)
        .add_plugin(player::PlayerPlugin)
        .add_plugin(ui::UIPlugin)
        .run();
}
