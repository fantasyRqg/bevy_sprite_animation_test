#![allow(clippy::type_complexity)]

use bevy::{
    prelude::*,
};
use bevy::prelude::shape::Quad;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::texture::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};

use swj::map::{MapPlugin, TmxMap, TmxMapAsset, TmxMapBg};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(ImagePlugin::default_nearest())
        ).add_plugins(MapPlugin)
        .init_resource::<MapAsset>()
        .init_resource::<BgImage>()
        .add_systems(Startup, setup)
        .add_systems(Update, (check_map_load,
                              file_drag_and_drop_system,
                              check_bg_load,
        ))
        .run();
}

#[derive(Resource, Deref, DerefMut, Default)]
struct MapAsset(Handle<TmxMapAsset>);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut map_asset: ResMut<MapAsset>,
) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            far: 2000.0,
            near: -3000.0,
            ..default()
        },
        ..default()
    });

    map_asset.0 = asset_server.load("Resources/UI/stages/stage1/game_scene_stage1.tmx");
}

fn check_map_load(
    mut commands: Commands,
    map_asset: Res<MapAsset>,
    mut events: EventReader<AssetEvent<TmxMapAsset>>,
    tmx_map_asset: Res<Assets<TmxMapAsset>>,
) {
    for event in events.read() {
        if event.is_loaded_with_dependencies(&map_asset.0) {
            commands.spawn(TmxMap {
                handle: map_asset.0.clone()
            });
        }
    }
}




#[derive(Resource, Default, Deref, DerefMut)]
struct BgImage(Handle<Image>);

fn file_drag_and_drop_system(
    mut events: EventReader<FileDragAndDrop>,
    asset_server: Res<AssetServer>,
    mut bg_image: ResMut<BgImage>,
) {
    for event in events.read() {
        if let FileDragAndDrop::DroppedFile { window, path_buf } = event {
            info!("Dropped file: {:?}", path_buf);
            let sampler_desc = ImageSamplerDescriptor {
                address_mode_u: ImageAddressMode::Repeat,
                address_mode_v: ImageAddressMode::Repeat,
                ..default()
            };

            let settings = move |s: &mut ImageLoaderSettings| {
                s.sampler = ImageSampler::Descriptor(sampler_desc.clone());
            };

            bg_image.0 = asset_server.load_with_settings(path_buf.clone(), settings);
        }
    }
}

fn check_bg_load(
    mut commands: Commands,
    mut query: Query<(Entity, &TmxMapBg)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    textures: ResMut<Assets<Image>>,
    bg_image: Res<BgImage>,
    mut asset_event: EventReader<AssetEvent<Image>>,
) {
    for ae in asset_event.read() {
        if !ae.is_loaded_with_dependencies(&bg_image.0) {
            continue;
        }

        for (entity, bg) in query.iter_mut() {
            let mut mesh: Mesh = Quad::new(bg.0).into();

            let bg_image_handle = textures.get(&bg_image.0).unwrap();

            if let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0) {
                for uv in uvs.iter_mut() {
                    uv[0] *= bg.x / bg_image_handle.width() as f32;
                    uv[1] *= bg.y / bg_image_handle.height() as f32;
                }
            }

            let material = materials.add(ColorMaterial::from(bg_image.clone()));

            commands.spawn(ColorMesh2dBundle {
                mesh: meshes.add(mesh).into(),
                transform: Transform::from_translation(Vec3::new(bg.x / 2.0, bg.y / 2.0, 0.0)),
                material,
                ..Default::default()
            }).insert(TmxMapBg(Vec2::from(bg.0)));
            commands.entity(entity).despawn_recursive();
        }
    }
}

