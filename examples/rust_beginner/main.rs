use bevy::prelude::*;
use bevy::prelude::shape::Quad;
use bevy::sprite::MaterialMesh2dBundle;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());

    let mut mesh: Mesh = Quad::new(Vec2::new(300., 5.)).into();

    let img: Handle<Image> = asset_server.load("Resources/Animations/shot_tail_oragne.png");

    let vertex_colors: Vec<[f32; 4]> = vec![
        [1., 1., 1., 0.],
        [1., 1., 1., 0.],
        [1., 1., 1., 1.],
        [1., 1., 1., 1.],
    ];

    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vertex_colors);

    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes
            .add(mesh)
            .into(),
        material: materials.add(ColorMaterial::from(img)),
        transform: Transform::from_translation(Vec3::new(50., 0., 0.)),
        ..default()
    });
}