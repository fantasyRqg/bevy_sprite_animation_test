use std::f32::consts::{FRAC_PI_2, PI};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::ui::PrepareNextFrameMaterials;
use bevy::utils::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, hanle_key)
        .run();
}


#[derive(Component)]
struct Elm;

fn setup(
    mut commands: Commands,
) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(100.0, 100.0)),
                color: Color::RED,
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        },
        Elm
    ))
        .with_children(|parent| {
            parent.spawn(SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(20.0, 20.0)),
                    color: Color::GREEN,
                    ..default()
                },
                transform: Transform::from_xyz(-30.0, 30.0, 0.0),
                ..default()
            });
        });
}

fn hanle_key(
    mut query: Query<&mut Transform, With<Elm>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    for mut transform in query.iter_mut() {
        if keyboard_input.just_pressed(KeyCode::A) {
            
        }
        // if keyboard_input.pressed(KeyCode::D) {
        //     transform.translation.x += 1.0;
        // }
        // if keyboard_input.pressed(KeyCode::W) {
        //     transform.translation.y += 1.0;
        // }
        // if keyboard_input.pressed(KeyCode::S) {
        //     transform.translation.y -= 1.0;
        // }
    }
}
