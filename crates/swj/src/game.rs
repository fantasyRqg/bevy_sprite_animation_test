use bevy::app::App;
use bevy::prelude::*;
use crate::cocos2d_anim::Cocos2dAnimPlugin;
use crate::pf_controller::PfControllerPlugin;
use crate::sprite_debug::SpriteDebugPlugin;

#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum GameStates {
    #[default]
    Loading,
    Playing,
}


pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameStates>()
            .add_plugins((
                Cocos2dAnimPlugin,
                PfControllerPlugin,
                SpriteDebugPlugin,
            ))

            .add_systems(Startup, setup)
        ;
    }
}


fn setup(mut commands: Commands,
         asset_server: Res<AssetServer>,
         mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn(Camera2dBundle::default());
    // commands.spawn((
    //     SpriteSheetBundle {
    //         texture_atlas: texture_atlas_handle,
    //         sprite: TextureAtlasSprite::new(animation_indices.first),
    //         transform: Transform::from_scale(Vec3::splat(6.0)),
    //         ..default()
    //     },
    //     animation_indices,
    // ));
}