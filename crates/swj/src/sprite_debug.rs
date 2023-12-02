use bevy::app::AppExit;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use crate::pf_controller::{Projectile, SpawnEvent, Unit};


pub struct SpriteDebugPlugin;


impl Plugin for SpriteDebugPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Startup, (
                setup_ui,
            ))
            .add_systems(Update, (
                update_sprite_count::<TextUnitCount, Unit>,
                update_sprite_count::<TextProjectileCount, Projectile>,
                update_fps,
                btn_reset_game,
                btn_add_unit,
                btn_add_projectile,
            ))
        ;
    }
}

#[derive(Component)]
struct TextUnitCount;

#[derive(Component)]
struct TextProjectileCount;

#[derive(Component)]
struct TextFps;

#[derive(Component)]
struct TextResetGame;

fn setup_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                left: Val::Px(10.0),
                top: Val::Px(10.0),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_sections([
                    TextSection::new(
                        "Unit Count: ",
                        TextStyle {
                            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                            font_size: 20.0,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::from_style(TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 20.0,
                        color: Color::WHITE,
                    }),
                ]),
                TextUnitCount,
            ));


            parent.spawn((
                TextBundle::from_sections([
                    TextSection::new(
                        "Projectile Count: ",
                        TextStyle {
                            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                            font_size: 20.0,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::from_style(TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 20.0,
                        color: Color::WHITE,
                    }),
                ]),
                TextProjectileCount,
            ));

            parent.spawn((
                TextBundle::from_sections([
                    TextSection::new(
                        "FPS: ",
                        TextStyle {
                            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                            font_size: 20.0,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::from_style(TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 20.0,
                        color: Color::WHITE,
                    }),
                ]),
                TextFps,
            ));
        });

    commands.spawn(NodeBundle {
        style: Style {
            right: Val::Px(10.0),
            bottom: Val::Px(10.0),
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::ColumnReverse,
            ..Default::default()
        },
        ..Default::default()
    })
        .with_children(|parent| {
            parent.spawn((
                TextResetGame,
                ButtonBundle {
                    style: Style {
                        width: Val::Px(150.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    border_color: BorderColor(Color::GRAY),
                    ..default()
                }
            ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Remove All",
                        TextStyle {
                            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                            font_size: 20.0,
                            color: Color::rgb(0.2, 0.1, 0.1),
                        },
                    ));
                });


            parent.spawn((
                TextUnitCount,
                ButtonBundle {
                    style: Style {
                        width: Val::Px(150.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    border_color: BorderColor(Color::GRAY),
                    ..default()
                }
            ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Add Unit +500",
                        TextStyle {
                            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                            font_size: 20.0,
                            color: Color::rgb(0.2, 0.1, 0.1),
                        },
                    ));
                });


            parent.spawn((
                TextProjectileCount,
                ButtonBundle {
                    style: Style {
                        width: Val::Px(150.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    border_color: BorderColor(Color::GRAY),
                    ..default()
                }
            ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Add Projectile +500",
                        TextStyle {
                            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                            font_size: 20.0,
                            color: Color::rgb(0.2, 0.1, 0.1),
                        },
                    ));
                });
        });
}


fn update_fps(
    mut query_ui: Query<&mut Text, With<TextFps>>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let mut text = query_ui.single_mut();
    let fps = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS).unwrap();
    if let Some(fps) = fps.smoothed() {
        text.sections[1].value = format!("{:.2}", fps);
    }
}

fn update_sprite_count<T: Component, E: Component>(
    mut query_ui: Query<&mut Text, With<T>>,
    query_sprite: Query<&E>,
) {
    let num = query_sprite.iter().count();
    let mut text = query_ui.single_mut();
    text.sections[1].value = num.to_string();
}


fn btn_reset_game(
    mut commands: Commands,
    query_unit: Query<Entity, Or<(With<Unit>, With<Projectile>)>>,
    btn_query: Query<&Interaction, (Changed<Interaction>, With<TextResetGame>)>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    let interaction = btn_query.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        for entity in query_unit.iter() {
            commands.entity(entity).despawn_recursive();
        }

        // app_exit_events.send(AppExit);
    }
}

fn btn_add_unit(
    mut events: EventWriter<SpawnEvent>,
    btn_query: Query<&Interaction, (Changed<Interaction>, With<TextUnitCount>)>,
) {
    let interaction = btn_query.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        events.send(SpawnEvent::Unit(500));
    }
}

fn btn_add_projectile(
    mut events: EventWriter<SpawnEvent>,
    btn_query: Query<&Interaction, (Changed<Interaction>, With<TextProjectileCount>)>,
) {
    let interaction = btn_query.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        events.send(SpawnEvent::Projectile(500));
    }
}