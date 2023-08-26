use bevy::prelude::*;
use crate::world::tile::prelude::*;

#[allow(unused_imports)]
pub mod prelude {
    pub use super::{
        ResourceStat,
        PlayerResources,
    };
}

// TODO: Add necessery comments.
pub struct ResourcePlugin;

impl Plugin for ResourcePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::Startup, setup_player_resources);
    }
}

pub struct ResourceStat {
    pub material: ResourceMaterial,
    pub quantity: usize,
}

impl ResourceStat {
    pub fn new(material: ResourceMaterial, quantity: usize) -> Self {
        Self {
            material,
            quantity,
        }
    }
}

#[derive(Resource)]
pub struct PlayerResources {
    pub resources: [ResourceStat; 6],
}

impl Default for PlayerResources {
    fn default() -> Self {
        Self {
            resources: [
                ResourceStat::new(ResourceMaterial::Dirt, 0),
                ResourceStat::new(ResourceMaterial::Stone, 0),
                ResourceStat::new(ResourceMaterial::Coal, 0),
                ResourceStat::new(ResourceMaterial::Iron, 0),
                ResourceStat::new(ResourceMaterial::Gold, 0),
                ResourceStat::new(ResourceMaterial::Crystal, 0),
            ]
        }
    }
}

pub fn setup_player_resources(mut commands: Commands) {
    commands.insert_resource(PlayerResources::default());
}