use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::window::CursorMoved;
use bevy::winit::WinitPlugin;
use swj::game::GamePlugin;

use crate::mac_winit::MyWinitPlugin;

mod mac_winit;

fn move_system(
    mut mouse_events: EventReader<CursorMoved>,
    mut query: Query<&mut Transform, With<Elm>>,
) {
    for mv in mouse_events.read() {
        for mut t in &mut query {
            t.translation.x = mv.position.x;
            t.translation.y = mv.position.y;
        }
    }
}

#[derive(Component)]
struct Elm;

fn setup_system(mut commands: Commands,
                mut meshes: ResMut<Assets<Mesh>>,
                mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(50.).into()).into(),
            material: materials.add(ColorMaterial::from(Color::PURPLE)),
            ..default()
        },
        Elm
    ));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.build().disable::<WinitPlugin>())
        .add_plugins(MyWinitPlugin {})
        // .add_plugins(DefaultPlugins)
        .add_plugins(GamePlugin)
        // .add_systems(Startup, setup_system)
        // .add_systems(Update, move_system)
        .add_systems(Update, debug_system)
        .run();
}

fn debug_system(
    ui_scale: Res<UiScale>
) {
    // println!("ui_scale: {:?}", ui_scale.0);
}