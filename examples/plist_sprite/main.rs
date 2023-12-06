#![allow(clippy::type_complexity)]

use bevy::{
    prelude::*,
};
use bevy::math::vec2;
use bevy::ui::AlignItems::Center;
use bevy::ui::FlexDirection::Column;

use swj::cocos2d_anim::{AnimationMode, AnimEvent, Cocos2dAnimator, Cocos2dAnimPlugin};
use swj::cocos2d_anim::anim::Cocos2dAnimAsset;
use swj::cocos2d_anim::sprite_sheet::PlistSpriteFrameAsset;

fn main() {
    App::new()
        .add_state::<GameStates>()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(Cocos2dAnimPlugin)
        .init_resource::<AnimDataRes>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            check_load.run_if(in_state(GameStates::Loading)),
            anim_evt.run_if(in_state(GameStates::Playing)),
            debug_plist,
            debug_anchor,
            btn_system,
            file_drag_and_drop_system,
        ))
        .add_systems(OnEnter(GameStates::Playing), spawn_anim)
        .run();
}


#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
enum GameStates {
    #[default]
    Loading,
    Playing,
}

#[derive(Resource, Default)]
struct AnimDataRes {
    anim: Handle<Cocos2dAnimAsset>,
    plist: Handle<PlistSpriteFrameAsset>,
}


fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut anim_data: ResMut<AnimDataRes>,
) {
    commands.spawn(Camera2dBundle::default());
    anim_data.anim = asset_server.load("Resources/Animations/archer_soldier.ExportJson");
}

fn check_load(mut events: EventReader<AssetEvent<Cocos2dAnimAsset>>,
              anim_data: Res<AnimDataRes>,
              mut game_state: ResMut<NextState<GameStates>>,
) {
    for evt in events.read() {
        if evt.is_loaded_with_dependencies(&anim_data.anim) {
            info!("Anim loaded");
            game_state.set(GameStates::Playing);
        }
    }
}

#[derive(Component, Deref, DerefMut)]
struct AnimButton(String);

#[derive(Component)]
struct RemovableElm;

fn spawn_anim(
    mut commands: Commands,
    anim_data: Res<AnimDataRes>,
    animations: Res<Assets<Cocos2dAnimAsset>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Cocos2dAnimator {
            duration: None,
            anim_handle: anim_data.anim.clone(),
            new_anim: Some("run".to_string()),
            mode: AnimationMode::Loop,
            event_channel: Some(0),
        },
        SpatialBundle {
            transform: Transform {
                // scale: Vec3::splat(3.5),
                ..default()
            },
            ..default()
        },
        RemovableElm
    ));


    let animation = &animations.get(&anim_data.anim).unwrap().animation;

    commands.spawn((
        NodeBundle {
            style: Style {
                left: Val::Px(10.0),
                top: Val::Px(10.0),
                flex_direction: Column,
                align_items: Center,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(10.0),
                ..default()
            },
            ..default()
        },
        RemovableElm
    )).with_children(|parent| {
        for (name, _) in animation.iter() {
            parent.spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(100.0),
                        height: Val::Px(30.0),
                        align_items: Center,
                        justify_content: JustifyContent::Center,
                        margin: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::ANTIQUE_WHITE),
                    ..default()
                },
                AnimButton(name.clone()),
            )).with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    name,
                    TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 24.,
                        color: Color::BLACK,
                    },
                ));
            });
        }
    });
}

fn btn_system(
    btn_query: Query<(&Interaction, &AnimButton), Changed<Interaction>>,
    mut anim_query: Query<&mut Cocos2dAnimator>,
) {
    for (interaction, anim_btn) in btn_query.iter() {
        if matches!(interaction,Interaction::Pressed) {
            for mut anim in anim_query.iter_mut() {
                anim.switch_anim(anim_btn);
            }
        }
    }
}

fn anim_evt(
    mut events: EventReader<AnimEvent>
) {
    // for evt in events.read() {
    //     info!("AnimEvent: {:?}", evt);
    // }
}


fn debug_plist(
    texture_atlas: Res<Assets<TextureAtlas>>,
    sprite_sheet: Res<Assets<PlistSpriteFrameAsset>>,
    animation: Res<Assets<Cocos2dAnimAsset>>,
) {
    // for atlas in texture_atlas.iter() {
    //     info!("Atlas: {:?}", atlas.1.texture);
    // }
    //
    // for sheet in sprite_sheet.iter() {
    //     info!("Sheet: {:?}", sheet.0);
    // }

    // for anim in animation.iter() {
    //     info!("Anim: {:?}", anim);
    // }
}

fn debug_anchor(
    mut gizmos: Gizmos,
) {
    gizmos.line_2d(vec2(-10.0, 0.0), vec2(10.0, 0.0), Color::RED);
    gizmos.line_2d(vec2(0.0, -10.0), vec2(0.0, 10.0), Color::RED);
}


fn file_drag_and_drop_system(
    mut commands: Commands,
    mut events: EventReader<FileDragAndDrop>,
    asset_server: Res<AssetServer>,
    mut anim_data: ResMut<AnimDataRes>,
    mut game_state: ResMut<NextState<GameStates>>,
    mut rm_query: Query<Entity, With<RemovableElm>>,
) {
    for event in events.read() {
        if let FileDragAndDrop::DroppedFile { window, path_buf } = event {
            for re in rm_query.iter_mut() {
                commands.entity(re).despawn_recursive();
            }

            anim_data.anim = asset_server.load(path_buf.to_str().unwrap().to_string());
            game_state.set(GameStates::Loading);

            break;
        }
    }
}