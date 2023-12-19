use bevy::app::App;
use bevy::math::vec2;
use bevy::prelude::*;

use crate::clash::ClashPlugin;
use crate::cocos2d_anim::Cocos2dAnimPlugin;
use crate::map::MapPlugin;
use crate::resource::ResourcePlugin;
use crate::unit::UnitPlugin;

#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum GameStates {
    #[default]
    PrepareLoad,
    Loading,
    PrepareScene,
    Playing,
    Exited,
}


pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameStates>()
            .add_plugins((
                Cocos2dAnimPlugin,
                // PfControllerPlugin,
                // SpriteDebugPlugin,
                ResourcePlugin,
                UnitPlugin,
                MapPlugin,
                ClashPlugin,
            ))

            .add_systems(Startup, setup)
            // .add_systems(Update, (
            //     debug_pos
            // ))
        ;
    }
}


fn setup(mut commands: Commands,
         _asset_server: Res<AssetServer>,
         _texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            far: 2000.0,
            near: -3000.0,
            ..default()
        },
        ..default()
    });
}

fn debug_pos(
    mut gizmos: Gizmos
) {
    let x_offset = 100.0;
    gizmos.rect_2d(vec2(-x_offset / 2.0, 1126.0 / 2.0 - 80.), 0., vec2(x_offset, 500.0), Color::RED);
    gizmos.rect_2d(vec2(4672.0 + x_offset / 2.0, 1126.0 / 2.0), 0., vec2(x_offset, 600.0), Color::RED);
}