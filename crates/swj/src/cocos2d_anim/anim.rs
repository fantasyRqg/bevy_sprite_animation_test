use bevy::{
    asset::{AssetLoader, io::Reader, LoadContext},
    prelude::*,
    reflect::TypePath,
    utils::BoxedFuture,
};
use bevy::asset::{AssetContainer, AsyncReadExt};
use bevy::math::vec2;
use bevy::utils::{HashMap, thiserror};
use serde_json::Value;
use thiserror::Error;

use crate::cocos2d_anim::sprite_sheet::PlistSpriteFrameAsset;

struct TextureData {
    plist_file: String,
    px: f32,
    py: f32,
}

struct DisplayData {
    name: String,
    xy: Vec2,
    scale: Vec2,
}

struct BoneData {
    display_data: Vec<DisplayData>,
    translate: Vec3,
    scale: Vec2,
}

struct MoveBoneFrameData {
    translate: Vec3,
    scale: Vec2,
    evt: Option<String>,
    color: Option<Color>,
    di: usize,
    fi: usize,
}

struct MoveBoneData {
    layers: HashMap<String, Vec<MoveBoneFrameData>>,
    interval: f32,
    frame_size: usize,
}


#[derive(Default)]
pub struct Cocos2dAnimAssetLoader;

#[derive(Debug)]
pub struct Cocos2dAnimFrame {
    pub translate: Vec3,
    pub scale: Vec2,
    pub rotated: bool,

    pub evt: Option<String>,
    pub color: Option<Color>,

    pub fi: usize,

    pub sprite_atlas: Handle<TextureAtlas>,
    pub sprite_idx: usize,
}

#[derive(Debug)]
pub struct Cocos2dAnimMove {
    pub interval: f32,
    pub frame_size: usize,
    pub layers: HashMap<String, Vec<Cocos2dAnimFrame>>,
}

#[derive(Asset, TypePath, Debug)]
pub struct Cocos2dAnimAsset {
    pub animation: HashMap<String, Cocos2dAnimMove>,
    plist_handles: Vec<Handle<PlistSpriteFrameAsset>>,
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum Cocos2dAnimLoaderError {
    /// An [IO](std::io) Error
    #[error("Could load asset: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for Cocos2dAnimAssetLoader {
    type Asset = Cocos2dAnimAsset;
    type Settings = ();
    type Error = Cocos2dAnimLoaderError;

    fn load<'a>(&'a self, reader: &'a mut Reader,
                _settings: &'a Self::Settings,
                load_context: &'a mut LoadContext)
                -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let json_str = String::from_utf8(bytes).unwrap();
            let anim_data: HashMap<String, Value> = serde_json::from_str(&json_str).unwrap();

            let plist_file_names = anim_data["config_file_path"].as_array().unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect::<Vec<String>>();

            let mut plist_file_data: HashMap<String, PlistSpriteFrameAsset> = HashMap::new();
            let mut plist_handles = Vec::new();
            for plist_file_name in plist_file_names.iter() {
                let sprite_sheet = load_context.load_direct(
                    load_context.path().parent().unwrap().join(plist_file_name.clone()),
                ).await.unwrap();

                let sprite_sheet = sprite_sheet.get::<PlistSpriteFrameAsset>().unwrap();
                plist_file_data.insert(plist_file_name.clone(), sprite_sheet.clone());
            }

            for plist_file_name in plist_file_names.iter() {
                let handle = load_context.load(load_context.path().parent().unwrap().join(plist_file_name.clone()));
                plist_handles.push(handle);
            }


            let mut texture_data = HashMap::new();

            for ta in anim_data["texture_data"].as_array().unwrap() {
                let ta = ta.as_object().unwrap();
                let name = ta["name"].as_str().unwrap().to_string();
                let mut plist_file = ta["plistFile"].as_str().unwrap().to_string();
                let px = ta["pX"].as_f64().unwrap() as f32;
                let py = ta["pY"].as_f64().unwrap() as f32;

                if plist_file.is_empty() {
                    for (plist_name, plist_frame) in plist_file_data.iter() {
                        if plist_frame.frames.iter().any(|sf| sf.name == name) {
                            plist_file = plist_name.clone();
                            break;
                        }
                    }

                    if plist_file.is_empty() {
                        warn!("plist_file is empty and can't be found in all loaded plist files.\
                        Animation file: {}, name: {}",
                               load_context.path().to_str().unwrap(),
                               name);
                    }
                }

                let data = TextureData {
                    plist_file,
                    px,
                    py,
                };

                texture_data.insert(name, data);
            }

            let mut bone_data = HashMap::new();

            for bd in anim_data["armature_data"].as_array().unwrap().first().unwrap()["bone_data"].as_array().unwrap() {
                let bd = bd.as_object().unwrap();
                let name = bd["name"].as_str().unwrap().to_string();
                let translate = Vec3::new(
                    bd["x"].as_f64().unwrap() as f32,
                    bd["y"].as_f64().unwrap() as f32,
                    bd["z"].as_f64().unwrap() as f32,
                );
                let scale = Vec2::new(
                    bd["cX"].as_f64().unwrap() as f32,
                    bd["cY"].as_f64().unwrap() as f32,
                );
                let mut display_data = Vec::new();

                for dd in bd["display_data"].as_array().unwrap() {
                    let dd = dd.as_object().unwrap();
                    let name = dd["name"].as_str().unwrap().to_string();
                    let name = name.replace(".png", "");

                    let skin_data = dd["skin_data"].as_array().unwrap().first().unwrap();
                    let xy = Vec2::new(
                        skin_data["x"].as_f64().unwrap() as f32,
                        skin_data["y"].as_f64().unwrap() as f32,
                    );
                    let scale = Vec2::new(
                        skin_data["cX"].as_f64().unwrap() as f32,
                        skin_data["cY"].as_f64().unwrap() as f32,
                    );

                    display_data.push(DisplayData {
                        name,
                        xy,
                        scale,
                    });
                }


                let data = BoneData {
                    display_data,
                    translate,
                    scale,
                };

                bone_data.insert(name, data);
            }


            let mut move_bone_data = HashMap::new();

            for mbd in anim_data["animation_data"].as_array().unwrap().first().unwrap()["mov_data"].as_array().unwrap() {
                let mbd = mbd.as_object().unwrap();
                let name = mbd["name"].as_str().unwrap().to_string();
                let interval = mbd["sc"].as_f64().unwrap() as f32;
                let interval = interval * 60.0;
                let interval = 1.0 / interval * 2.0;
                let frame_size = mbd["dr"].as_f64().unwrap() as usize + 1;
                let mut layers = HashMap::new();

                if mbd.get("mov_bone_data").is_none() {
                    continue;
                }

                for layer in mbd["mov_bone_data"].as_array().unwrap() {
                    let layer = layer.as_object().unwrap();
                    let name = layer["name"].as_str().unwrap().to_string();
                    let mut layer_data = Vec::new();

                    for frame in layer["frame_data"].as_array().unwrap() {
                        let frame = frame.as_object().unwrap();
                        let di = frame["dI"].as_f64().unwrap() as usize;
                        let fi = frame["fi"].as_f64().unwrap() as usize;
                        let translate = Vec3::new(
                            frame["x"].as_f64().unwrap() as f32,
                            frame["y"].as_f64().unwrap() as f32,
                            frame["z"].as_f64().unwrap() as f32,
                        );
                        let scale = Vec2::new(
                            frame["cX"].as_f64().unwrap() as f32,
                            frame["cY"].as_f64().unwrap() as f32,
                        );
                        let evt = if let Some(evt) = frame.get("evt") {
                            evt.as_str().map(|s| s.to_string())
                        } else {
                            None
                        };
                        let color = if let Some(color) = frame.get("color") {
                            color.as_object().map(|a| {
                                Color::rgba(
                                    a["a"].as_f64().unwrap() as f32 / 255.0,
                                    a["r"].as_f64().unwrap() as f32 / 255.0,
                                    a["g"].as_f64().unwrap() as f32 / 255.0,
                                    a["b"].as_f64().unwrap() as f32 / 255.0,
                                )
                            })
                        } else {
                            None
                        };


                        layer_data.push(MoveBoneFrameData {
                            translate,
                            scale,
                            evt,
                            color,
                            di,
                            fi,
                        });
                    }

                    layers.insert(name, layer_data);
                }

                let data = MoveBoneData {
                    layers,
                    interval,
                    frame_size,
                };

                move_bone_data.insert(name, data);
            }


            let mut animation = HashMap::new();

            for (name, mbd) in move_bone_data {
                let mut layer_map = HashMap::new();

                for (layer_name, layer_data) in mbd.layers {
                    let mut frames = Vec::new();

                    for frame_data in layer_data {
                        let bd = &bone_data[&layer_name];
                        let dd = &bd.display_data[frame_data.di];
                        let tex_data = &texture_data[&dd.name];
                        let sprite_sheet = &plist_file_data[&tex_data.plist_file];
                        let sprite_idx = sprite_sheet.frames.iter().position(|sf| sf.name == dd.name).unwrap();
                        let sprite_frame = &sprite_sheet.frames[sprite_idx];
                        let mut offset = Vec2::from(sprite_frame.offset);

                        let frame_size = Vec2::from(sprite_frame.source_size);
                        let frame_center = frame_size / 2.0;
                        let anchor = frame_size * vec2(tex_data.px, tex_data.py);
                        let anchor = anchor.round();
                        offset = offset - (anchor - frame_center);


                        let mut translate = offset.extend(0.0);
                        translate += dd.xy.extend(0.0);
                        translate += bd.translate;
                        translate += frame_data.translate;

                        let mut scale = bd.scale;
                        scale *= dd.scale;
                        scale *= frame_data.scale;


                        let frame = Cocos2dAnimFrame {
                            translate,
                            scale,
                            rotated: sprite_frame.rotated,
                            evt: frame_data.evt,
                            color: frame_data.color,
                            fi: frame_data.fi,
                            sprite_atlas: sprite_sheet.atlas.clone(),
                            sprite_idx,
                        };

                        frames.push(frame);
                    }

                    layer_map.insert(layer_name, frames);
                }

                let data = Cocos2dAnimMove {
                    interval: mbd.interval,
                    frame_size: mbd.frame_size,
                    layers: layer_map,
                };

                animation.insert(name, data);
            }


            Ok(Cocos2dAnimAsset {
                animation,
                plist_handles,
            })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["exportjson"]
    }
}
