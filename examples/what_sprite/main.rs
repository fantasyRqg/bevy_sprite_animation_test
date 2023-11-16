#![allow(clippy::type_complexity)]

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, setup)
        .run();
}


fn setup(mut commands: Commands,
         asset_server: Res<AssetServer>,
         mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle: Handle<Image> = asset_server.load("textures/archer/archer.png");
    println!("texture_handle: {:?}", texture_handle);
    let texture_handle: Handle<Image> = asset_server.load("textures/gabe/gabe-idle-run.png");
    println!("texture_handle: {:?}", texture_handle);
    let texture_handle = asset_server.load("textures/archer/archer.png");
    println!("texture_handle: {:?}", texture_handle);
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle.clone(), Vec2::new(176.0, 148.0), 4, 13, None, None);
    let tex_handle = texture_atlases.add(texture_atlas);
    println!("tex_handle: {:?}", tex_handle);
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(176.0, 148.0), 4, 13, None, None);
    let tex_handle = texture_atlases.add(texture_atlas);
    println!("tex_handle: {:?}", tex_handle);
    commands.spawn(Camera2dBundle::default());
    commands.spawn(SpriteSheetBundle {
        texture_atlas: tex_handle.clone(),
        // transform: Transform::from_scale(Vec3::splat(0.5)),
        ..default()
    });

    commands.spawn(SpriteSheetBundle {
        texture_atlas: tex_handle.clone(),
        transform: Transform {
            translation: Vec3::new(-80.0, 0.0, 0.0),
            scale: Vec3::splat(0.5),
            ..default()
        },
        ..default()
    });

    commands.spawn(SpriteSheetBundle {
        texture_atlas: tex_handle.clone(),
        transform: Transform::from_translation(Vec3::new(80.0, 0.0, 0.0)),
        sprite: TextureAtlasSprite {
            index: 1,
            custom_size: Some(Vec2::new(100.0, 200.0)),
            ..default()
        },
        ..default()
    });
}