use bevy::{
    asset::{AssetLoader, io::Reader, LoadContext},
    prelude::*,
    reflect::TypePath,
    utils::BoxedFuture,
};
use bevy::asset::AsyncReadExt;
use bevy::math::{vec2, vec3};
use bevy::prelude::shape::Quad;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::texture::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy::sprite::Anchor;
use bevy::utils::thiserror;
use plist::Dictionary;
use plist::Value;
use serde_xml::value::{Content, Element};
use thiserror::Error;


pub struct MapPlugin;


impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<TmxMapAsset>()
            .init_asset_loader::<TmxMapAssetLoader>()
            .add_systems(Update, map_system);
    }
}

#[derive(Component)]
pub struct TmxMap {
    pub handle: Handle<TmxMapAsset>,
}

#[derive(Component, Deref, DerefMut)]
pub struct TmxMapBg(pub Vec2);

fn map_system(
    mut commands: Commands,
    tmx_map_asset: Res<Assets<TmxMapAsset>>,
    query: Query<&TmxMap, Added<TmxMap>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut textures: ResMut<Assets<Image>>,
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

        let mut mesh: Mesh = Quad::new(Vec2::new(tmx_map.width, tmx_map.height)).into();

        let bg_image = textures.get(&tmx_map.background).unwrap();

        if let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0) {
            for uv in uvs.iter_mut() {
                uv[0] *= tmx_map.width / bg_image.width() as f32;
                uv[1] *= tmx_map.height / bg_image.height() as f32;
            }
        }

        let material = materials.add(ColorMaterial::from(tmx_map.background.clone()));

        commands.spawn(ColorMesh2dBundle {
            mesh: meshes.add(mesh).into(),
            transform: Transform::from_translation(Vec3::new(tmx_map.width / 2.0, tmx_map.height / 2.0, 0.0)),
            material,
            ..Default::default()
        }).insert(TmxMapBg(Vec2::new(tmx_map.width, tmx_map.height)));
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

    fn load<'a>(&'a self, reader: &'a mut Reader, settings: &'a Self::Settings,
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
                                    let bg_w = image[0].attributes["width"].first().unwrap().parse::<f32>().unwrap();
                                    let bg_h = image[0].attributes["height"].first().unwrap().parse::<f32>().unwrap();

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
