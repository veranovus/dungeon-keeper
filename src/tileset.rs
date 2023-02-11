use bevy::prelude::*;

use crate::globals;

pub struct TilesetPlugin;

impl Plugin for TilesetPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, setup_tileset);
    }
}

// NOTE: Path to the tileset image.
const TILESET_PATH: &str = "tileset.png";

// NOTE: A resource that holds a reference to sprite atlas.
#[derive(Resource)]
pub struct Tileset(Handle<TextureAtlas>);

// NOTE: Loads the default tileset that is used for every
//       sprite, and creates the `Tileset` resource.
fn setup_tileset(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture = assets.load(TILESET_PATH);
    let texture_atlas = TextureAtlas::from_grid(
        texture, 
        Vec2::new(
            globals::SPRITE_IMAGE_SIZE.0 as f32, 
            globals::SPRITE_IMAGE_SIZE.1 as f32,
        ), 
        globals::ATLAS_SIZE.0, 
        globals::ATLAS_SIZE.1, 
        None, 
        None,
    );
    let atlas_handle = texture_atlases.add(texture_atlas);

    commands.insert_resource(Tileset(atlas_handle));
}

// NOTE: Spawns a default sprite entity with given properties.
pub fn spawn_sprite_from_tileset(
    commands: &mut Commands,
    tileset: &Tileset,
    index: usize,
    translation: Vec3,
    scale: Vec3,
    color: Color,
) -> Entity {
    // NOTE: Create the sprite with given texture index, and color.
    let mut sprite = TextureAtlasSprite::new(index);
    sprite.color = color;
    sprite.anchor = globals::DEFAULT_SPRITE_ANCHOR;

    // NOTE: Create a basic `SpriteSheet` entity.
    return commands
        .spawn(SpriteSheetBundle {
            sprite,
            texture_atlas: tileset.0.clone(),
            transform: Transform {
                translation,
                scale,
                ..Default::default()
            },
            ..Default::default()
        })
        .id();
}
