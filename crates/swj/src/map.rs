use bevy::{
    asset::{AssetLoader, io::Reader, LoadContext},
    prelude::*,
    reflect::TypePath,
    utils::BoxedFuture,
};
use bevy::asset::AsyncReadExt;
use bevy::input::mouse::MouseWheel;
use bevy::input::touch::TouchPhase;
use bevy::input::touchpad::TouchpadMagnify;
use bevy::math::vec2;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::texture::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy::sprite::Anchor;
use bevy::utils::HashMap;
use serde_xml::value::{Content, Element};
use thiserror::Error;

use crate::game::GameStates::PrepareScene;

pub struct MapPlugin;


impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<TmxMapAsset>()
            .init_asset_loader::<TmxMapAssetLoader>()
            .init_resource::<CameraMoveLimit>()
            .init_resource::<CurrentMapInfo>()
            .add_systems(Update,
                         (
                             mac_view_move,
                             touch_view_move,
                             (
                                 map_system,
                                 mv_camera_to_map,
                             ).run_if(in_state(PrepareScene)),
                         ),
            )
        ;
    }
}

#[derive(Component)]
pub struct TmxMap {
    pub handle: Handle<TmxMapAsset>,
}

#[derive(Component, Deref, DerefMut)]
pub struct TmxMapBg(pub Vec2);


#[derive(Resource, Default, Debug)]
struct CameraMoveLimit {
    scale_min: f32,
    scale_max: f32,
    rect: Rect,
    win_size: Vec2,
}

fn mac_view_move(
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

            adjust_camera(&camera_move_limit, &mut transform, &project);
        }
    }

    for mwe in mouse_wheel_events.read() {
        for (mut transform, project) in query.iter_mut() {
            // info!("{:?}", mwe);
            transform.translation.x -= mwe.x;
            transform.translation.y += mwe.y;

            adjust_camera(camera_move_limit.as_ref(), transform.as_mut(), project.as_ref());
        }
    }
}


fn touch_view_move(
    mut touch_events: EventReader<TouchInput>,
    camera_move_limit: Res<CameraMoveLimit>,
    mut camera_query: Query<(&mut Transform, &mut OrthographicProjection)>,
    mut touch_tracker: Local<HashMap<u64, Vec2>>,
    win_query: Query<&Window>,
) {
    let win = win_query.single();
    let win_size = vec2(win.resolution.width(), win.resolution.height());
    for evt in touch_events.read() {
        for (mut transform, mut project) in camera_query.iter_mut() {
            match evt.phase {
                TouchPhase::Started => {
                    if touch_tracker.len() == 2 {
                        continue;
                    }

                    touch_tracker.insert(evt.id, evt.position);
                }
                TouchPhase::Moved => {
                    if touch_tracker.len() == 1 {
                        // translate
                        if let Some(last_pos) = touch_tracker.get_mut(&evt.id) {
                            let pos = evt.position;
                            let delta = pos - *last_pos;
                            transform.translation.x -= delta.x * project.scale;
                            transform.translation.y += delta.y * project.scale;
                            *last_pos = pos;

                            adjust_camera(camera_move_limit.as_ref(), transform.as_mut(), project.as_ref());
                        }
                    } else if touch_tracker.len() == 2 {
                        // scale

                        let mv_touch = if let Some(pos) = touch_tracker.get(&evt.id) {
                            *pos
                        } else {
                            continue;
                        };

                        let fixed_touch = *touch_tracker.iter().find(|(id, _)| **id != evt.id).unwrap().1;

                        let new_mv_touch = evt.position;

                        let scale = ((fixed_touch - new_mv_touch).length() - (fixed_touch - mv_touch).length()) / (fixed_touch - mv_touch).length();
                        touch_tracker.insert(evt.id, new_mv_touch);

                        let view_center = win_size / 2.0;
                        let mut touch_center = (fixed_touch + new_mv_touch) / 2.0;
                        // touch_center.y = win_size.y - touch_center.y;

                        let delta = (touch_center - view_center) * project.scale;
                        let delta = (touch_center - view_center) * (project.scale - scale) - delta;
                        transform.translation.x -= delta.x;
                        transform.translation.y += delta.y;
                        project.scale -= scale;

                        if project.scale < camera_move_limit.scale_min {
                            project.scale = camera_move_limit.scale_min;
                        } else if project.scale > camera_move_limit.scale_max {
                            project.scale = camera_move_limit.scale_max;
                        }

                        adjust_camera(&camera_move_limit, &mut transform, &project);
                    }
                }
                TouchPhase::Canceled | TouchPhase::Ended => {
                    touch_tracker.remove(&evt.id);
                }
            }
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


fn mv_camera_to_map(
    mut camera_move_limit: ResMut<CameraMoveLimit>,
    mut camera_query: Query<(&mut Transform, &mut OrthographicProjection)>,
    win_query: Query<&Window>,
    tmx_map_asset: Res<Assets<TmxMapAsset>>,
    query: Query<&TmxMap, Added<TmxMap>>,
) {
    for tmx_map in query.iter() {
        let map = tmx_map_asset.get(&tmx_map.handle).unwrap();
        let w = map.width;
        let h = map.height;

        let win = win_query.single();
        let win_w = win.width();
        let win_h = win.height();

        camera_move_limit.rect = Rect::new(0.0, 0.0, w, h);
        camera_move_limit.win_size = Vec2::new(win_w, win_h);
        camera_move_limit.scale_max = h / win_h;
        camera_move_limit.scale_min = camera_move_limit.scale_max / 4.0;

        for (mut transform, mut project) in camera_query.iter_mut() {
            project.scale = h / win_h;

            let t_w = win_w * project.scale;
            transform.translation.y = h / 2.0;
            transform.translation.x = t_w / 2.0;
        }
    }
}

#[derive(Resource, Default)]
pub struct CurrentMapInfo {
    pub size: Vec2,
}

fn map_system(
    mut commands: Commands,
    tmx_map_asset: Res<Assets<TmxMapAsset>>,
    query: Query<&TmxMap, Added<TmxMap>>,
    mut current_map_info: ResMut<CurrentMapInfo>,
    textures: ResMut<Assets<Image>>,
) {
    for tmx_map in query.iter() {
        let tmx_map = tmx_map_asset.get(&tmx_map.handle).unwrap();
        for element in tmx_map.elements.iter() {
            commands.spawn(SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(element.size),
                    anchor: Anchor::BottomLeft,
                    ..default()
                },
                texture: element.image.clone(),
                transform: Transform::from_translation(element.translate),
                ..Default::default()
            });
        }

        current_map_info.size = Vec2::new(tmx_map.width, tmx_map.height);


        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Vec2::new(tmx_map.width, tmx_map.height).into(),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(tmx_map.width / 2.0, tmx_map.height / 2.0, 0.0)),
                texture: tmx_map.background.clone(),
                ..default()
            },
            ImageScaleMode::Tiled {
                tile_x: true,
                tile_y: true,
                stretch_value: 1.0,
            },
            TmxMapBg(Vec2::new(tmx_map.width, tmx_map.height))
        ));
    }
}

#[derive(Debug, Clone)]
struct MapElement {
    size: Vec2,
    translate: Vec3,
    image: Handle<Image>,
}

#[derive(Asset, TypePath, Debug, Clone)]
pub struct TmxMapAsset {
    pub width: f32,
    pub height: f32,
    elements: Vec<MapElement>,
    background: Handle<Image>,
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum TmxMapAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could load asset: {0}")]
    Io(#[from] std::io::Error),
}


#[derive(Default)]
pub struct TmxMapAssetLoader {}

impl AssetLoader for TmxMapAssetLoader {
    type Asset = TmxMapAsset;
    type Settings = ();
    type Error = TmxMapAssetLoaderError;

    fn load<'a>(&'a self, reader: &'a mut Reader, _settings: &'a Self::Settings,
                load_context: &'a mut LoadContext)
                -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut tmx_str = String::new();
            reader.read_to_string(&mut tmx_str).await?;

            let tmx_data: Element = serde_xml::from_str(&tmx_str).unwrap();
            let tile_column: i32 = tmx_data.attributes["width"].first().unwrap().parse().unwrap();
            let tile_row: i32 = tmx_data.attributes["height"].first().unwrap().parse().unwrap();
            let tile_width: f32 = tmx_data.attributes["tilewidth"].first().unwrap().parse().unwrap();
            let tile_height: f32 = tmx_data.attributes["tileheight"].first().unwrap().parse().unwrap();

            let map_width = tile_column as f32 * tile_width;
            let map_height = tile_row as f32 * tile_height;
            // println!("tile_column: {}, tile_row: {}, tile_width: {}, tile_height: {}", tile_column, tile_row, tile_width, tile_height);

            let mut elements = Vec::new();

            let mut background = None;

            if let Content::Members(members) = tmx_data.members {
                if let Some(tilesets) = members.get("tileset") {
                    for tileset in tilesets {
                        if tileset.attributes.contains_key("name") && tileset.attributes["name"].first().unwrap() == "game_map_bg" {
                            if let Content::Members(tileset) = &tileset.members {
                                if let Some(image) = tileset.get("image") {
                                    let bg_img = image[0].attributes["source"].first().unwrap();
                                    // let bg_w = image[0].attributes["width"].first().unwrap().parse::<f32>().unwrap();
                                    // let bg_h = image[0].attributes["height"].first().unwrap().parse::<f32>().unwrap();

                                    let sampler_desc = ImageSamplerDescriptor {
                                        address_mode_u: ImageAddressMode::Repeat,
                                        address_mode_v: ImageAddressMode::Repeat,
                                        ..default()
                                    };

                                    let settings = move |s: &mut ImageLoaderSettings| {
                                        s.sampler = ImageSampler::Descriptor(sampler_desc.clone());
                                    };


                                    let bg_path = load_context.path().parent().unwrap().parent().unwrap().join(bg_img);
                                    let handle = load_context.load_with_settings(bg_path, settings);

                                    background = Some(handle);
                                    // for c in 0..tile_column {
                                    //     for r in 0..tile_row {
                                    //         elements.append(&mut vec![
                                    //             MapElement {
                                    //                 size: Vec2::new(bg_w, bg_h),
                                    //                 translate: Vec3::new(c as f32 * tile_width, r as f32 * tile_height, 0.0),
                                    //                 image: handle.clone(),
                                    //             }
                                    //         ]);
                                    //     }
                                    // }
                                }
                            }
                        }
                    }
                }

                if let Some(objectgroup) = members.get("objectgroup") {
                    for og in objectgroup {
                        if og.attributes["name"].first().unwrap() == "decoration" {
                            let decoration = og;
                            if let Content::Members(decoration) = decoration.members.clone() {
                                if let Some(deco_objs) = decoration.get("object") {
                                    for deco in deco_objs {
                                        let name = deco.attributes["name"].first().unwrap();
                                        let x = deco.attributes["x"].first().unwrap().parse::<f32>().unwrap();
                                        let y = deco.attributes["y"].first().unwrap().parse::<f32>().unwrap();
                                        let width = deco.attributes["width"].first().unwrap().parse::<i32>().unwrap();
                                        let height = deco.attributes["height"].first().unwrap().parse::<i32>().unwrap();


                                        let z = if matches!(name.as_str(),"tudui_1.png"|"tudui_2.png"|"liefeng.png") {
                                            2.0
                                        } else {
                                            y
                                        };

                                        let y = map_height - y;

                                        elements.append(&mut vec![
                                            MapElement {
                                                size: Vec2::new(width as f32, height as f32),
                                                translate: Vec3::new(x, y, z),
                                                image: load_context.load(load_context.path().parent().unwrap().parent().unwrap().join(name)),
                                            }
                                        ]);
                                    }
                                }
                            }

                            break;
                        }
                    }
                }
            }


            Ok(TmxMapAsset {
                width: map_width,
                height: map_height,
                elements,
                background: background.expect("background image not found"),
            })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["tmx"]
    }
}
