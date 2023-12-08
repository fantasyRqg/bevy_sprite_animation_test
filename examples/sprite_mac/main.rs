#![allow(clippy::type_complexity)]

use bevy::{
    prelude::*,
};
use bevy::input::mouse::MouseWheel;
use bevy::input::touchpad::TouchpadMagnify;
use bevy::math::vec2;
use bevy::prelude::shape::Quad;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::texture::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};

use swj::game::GamePlugin;
use swj::map::{MapPlugin, TmxMap, TmxMapAsset, TmxMapBg};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(ImagePlugin::default_nearest())
        ).add_plugins(MapPlugin)
        .init_resource::<MapAsset>()
        .init_resource::<BgImage>()
        .init_resource::<CameraMoveLimit>()
        .add_systems(Startup, setup)
        .add_systems(Update, (check_map_load,
                              view_move,
                              file_drag_and_drop_system,
                              check_bg_load,
                              // debug_gizmos,
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

#[derive(Resource, Default)]
struct CameraMoveLimit {
    scale_min: f32,
    scale_max: f32,
    rect: Rect,
    win_size: Vec2,
}

fn check_map_load(
    mut commands: Commands,
    map_asset: Res<MapAsset>,
    mut events: EventReader<AssetEvent<TmxMapAsset>>,
    tmx_map_asset: Res<Assets<TmxMapAsset>>,
    mut query: Query<(&mut Transform, &mut OrthographicProjection)>,
    mut win_query: Query<&Window>,
    mut camera_move_limit: ResMut<CameraMoveLimit>,
) {
    for event in events.read() {
        if event.is_loaded_with_dependencies(&map_asset.0) {
            commands.spawn(TmxMap {
                handle: map_asset.0.clone()
            });


            let map = tmx_map_asset.get(&map_asset.0).unwrap();
            let w = map.width;
            let h = map.height;

            let win = win_query.single();
            let win_w = win.width();
            let win_h = win.height();

            camera_move_limit.rect = Rect::new(0.0, 0.0, w, h);
            camera_move_limit.win_size = Vec2::new(win_w, win_h);
            camera_move_limit.scale_max = h / win_h;
            camera_move_limit.scale_min = camera_move_limit.scale_max / 4.0;

            for (mut transform, mut project) in query.iter_mut() {
                project.scale = h / win_h;

                let t_w = win_w * project.scale;
                transform.translation.y = h / 2.0;
                transform.translation.x = t_w / 2.0;

                // project.scale = 1.0;
                // project.viewport_origin
                // transform.translation.x = w / 2.0;
                // transform.translation.y = h / 2.0;
            }
        }
    }
}

fn debug_gizmos(
    mut gizmos: Gizmos,
) {
    gizmos.rect_2d(Vec2::new(0.0, 0.0), 0.0, Vec2::new(1280.0, 720.0), Color::RED);
    let h = 1126.0;
    let w = h * 1280.0 / 720.0;
    // info!("-- w: {}, h: {}", w, h);
    gizmos.rect_2d(Vec2::new(w, h) / 2.0, 0.0, Vec2::new(w, h), Color::GREEN);
}


fn view_move(
    mut query: Query<(&mut Transform, &mut OrthographicProjection)>,
    mut touchpad_magnify_events: EventReader<TouchpadMagnify>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    camera_move_limit: Res<CameraMoveLimit>,
) {
    for tme in touchpad_magnify_events.read() {
        for (mut transform, mut project) in query.iter_mut() {
            project.scale -= tme.0 * 2.0;

            if project.scale < camera_move_limit.scale_min {
                project.scale = camera_move_limit.scale_min;
            } else if project.scale > camera_move_limit.scale_max {
                project.scale = camera_move_limit.scale_max;
            }

            adjust_camera(camera_move_limit.as_ref(), transform.as_mut(), project.as_ref());
        }
    }

    for mwe in mouse_wheel_events.read() {
        for (mut transform, mut project) in query.iter_mut() {
            // info!("{:?}", mwe);
            transform.translation.x -= mwe.x;
            transform.translation.y += mwe.y;

            adjust_camera(camera_move_limit.as_ref(), transform.as_mut(), project.as_ref());
        }
    }
}

fn adjust_camera(camera_move_limit: &CameraMoveLimit, transform: &mut Transform, project: &OrthographicProjection) {
    let view_size = camera_move_limit.win_size * project.scale;
    let view_rect = Rect::new(
        transform.translation.x - view_size.x / 2.0,
        transform.translation.y - view_size.y / 2.0,
        transform.translation.x + view_size.x / 2.0,
        transform.translation.y + view_size.y / 2.0,
    );

    let map_rect = camera_move_limit.rect;

    if view_rect.min.x < map_rect.min.x {
        transform.translation.x = map_rect.min.x + view_size.x / 2.0;
    } else if view_rect.max.x > map_rect.max.x {
        transform.translation.x = map_rect.max.x - view_size.x / 2.0;
    }

    if view_rect.min.y < map_rect.min.y {
        transform.translation.y = map_rect.min.y + view_size.y / 2.0;
    } else if view_rect.max.y > map_rect.max.y {
        transform.translation.y = map_rect.max.y - view_size.y / 2.0;
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

