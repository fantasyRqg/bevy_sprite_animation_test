use std::f32::consts::PI;
use std::iter::FlatMap;
use bevy::prelude::*;
use crate::cocos2d_anim::CocosAnimSet;
use crate::cocos2d_anim::PlistAnimation;
use crate::game::GameStates;

pub(crate) struct PfControllerPlugin;

impl Plugin for PfControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update,
                        (
                            move_sprite.in_set(CocosAnimSet::Update),
                        ).run_if(in_state(GameStates::Playing)),
        );
    }
}


fn move_sprite(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &mut TextureAtlasSprite), With<PlistAnimation>>,
) {
    // keyboard_input.get_pressed().for_each(|k| {
    //     info!("key: {:?}", k);
    // });
    for (mut transform, mut sprite) in query.iter_mut() {
        let mut translation = Vec3::splat(0.0);
        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            translation.x -= 1.0;
            sprite.flip_x = true;
        }

        if keyboard_input.pressed(KeyCode::ArrowRight) {
            translation.x += 1.0;
            sprite.flip_x = false;
        }

        if keyboard_input.pressed(KeyCode::ArrowUp) {
            translation.y += 1.0;
        }

        if keyboard_input.pressed(KeyCode::ArrowDown) {
            translation.y -= 1.0;
        }

        // println!("translation: {:?}", translation);
        if let Some(translation) = translation.try_normalize() {
            transform.translation += translation.normalize();
            // println!("transform: {:?}", transform.translation);
        }
    }
}
