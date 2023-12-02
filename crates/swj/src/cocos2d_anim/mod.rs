use std::f32::consts::FRAC_PI_2;

use bevy::{
    prelude::*,
    reflect::TypePath,
};
use bevy::math::vec3;
use bevy::utils::thiserror;
use thiserror::Error;

use crate::cocos2d_anim::sprite_sheet::{PlistSpriteAssetLoader, PlistSpriteFrameAsset, SpriteFrame};

pub mod sprite_sheet;
mod anim;

pub(crate) struct Cocos2dAnimPlugin;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) enum CocosAnimSet {
    Update,
    AdjustSprite,
}

impl Plugin for Cocos2dAnimPlugin {
    fn build(&self, app: &mut App) {
        app
            .configure_sets(Update, (CocosAnimSet::Update, CocosAnimSet::AdjustSprite).chain())
            .init_asset::<PlistSpriteFrameAsset>()
            .init_asset_loader::<PlistSpriteAssetLoader>()
            .add_systems(Update,
                         (
                             (
                                 animate_sprite.in_set(CocosAnimSet::Update),
                                 update_sprite.in_set(CocosAnimSet::AdjustSprite),
                             ),
                         ),
            )
        ;
    }
}


#[derive(Component)]
pub struct PlistAnimation {
    pub timer: Timer,
    pub plist_frame: Handle<PlistSpriteFrameAsset>,
    pub transform: Transform,
    pub animated: bool,
}

impl Default for PlistAnimation {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(1.0 / 30.0, TimerMode::Repeating),
            plist_frame: Handle::default(),
            transform: Transform::default(),
            animated: true,
        }
    }
}


fn update_sprite(
    mut query: Query<(&mut Transform, &TextureAtlasSprite, &mut PlistAnimation)>,
    plist_sprite: Res<Assets<PlistSpriteFrameAsset>>,
) {
    for (mut transform, sprite, pa) in &mut query {
        let ps = plist_sprite.get(pa.plist_frame.id()).unwrap();
        let sf = &ps.frames[sprite.index];

        *transform = pa.transform.clone();

        if sprite.flip_x {
            if sf.rotated {
                transform.rotate_local_z(-FRAC_PI_2);
            }
            transform.translation += vec3(-sf.offset.0, sf.offset.1, 0.0);
        } else {
            if sf.rotated {
                transform.rotate_local_z(FRAC_PI_2);
            }
            transform.translation += vec3(sf.offset.0, sf.offset.1, 0.0);
        }
    }
}

fn animate_sprite(
    time: Res<Time>,
    texture_atlas: Res<Assets<TextureAtlas>>,
    mut query: Query<(&mut TextureAtlasSprite, &mut PlistAnimation, &Handle<TextureAtlas>)>,
) {
    for (mut sprite, mut pa, tex) in &mut query {
        if !pa.animated {
            continue;
        }
        pa.timer.tick(time.delta());
        if pa.timer.just_finished() {
            let tex = texture_atlas.get(tex).unwrap();
            sprite.index = (sprite.index + 1) % tex.len();
        }
    }
}


