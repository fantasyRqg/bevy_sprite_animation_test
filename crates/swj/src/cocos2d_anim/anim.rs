use bevy::{
    asset::{AssetLoader, io::Reader, LoadContext},
    prelude::*,
    reflect::TypePath,
    utils::BoxedFuture,
};
use bevy::asset::{AssetContainer, AsyncReadExt};
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
    frames: HashMap<String, Vec<MoveBoneFrameData>>,
    interval: f32,
    frame_size: usize,
}


#[derive(Default)]
pub struct Cocos2dAnimAssetLoader;


#[derive(Asset, TypePath, Debug)]
pub struct CocosAnim2dAsset {
    // pub frames: Vec<SpriteFrame>,
    // pub atlas: Handle<TextureAtlas>,
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum CocosAnim2dLoaderError {
    /// An [IO](std::io) Error
    #[error("Could load shader: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for Cocos2dAnimAssetLoader {
    type Asset = CocosAnim2dAsset;
    type Settings = ();
    type Error = CocosAnim2dLoaderError;

    fn load<'a>(&'a self, reader: &'a mut Reader,
                settings: &'a Self::Settings,
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
            for plist_file_name in plist_file_names {
                let sprite_sheet = load_context.load_direct(
                    load_context.path().parent().unwrap().join(plist_file_name.clone()),
                ).await.unwrap();
                let sprite_sheet = sprite_sheet.get::<PlistSpriteFrameAsset>().unwrap();
                plist_file_data.insert(plist_file_name, sprite_sheet.clone());
            }


            let mut texture_data = HashMap::new();

            for ta in anim_data["texture_data"].as_array().unwrap() {
                let ta = ta.as_object().unwrap();
                let name = ta["name"].as_str().unwrap().to_string();
                let plist_file = ta["plistFile"].as_str().unwrap().to_string();
                let px = ta["pX"].as_f64().unwrap() as f32;
                let py = ta["pY"].as_f64().unwrap() as f32;
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


            Ok(CocosAnim2dAsset {})
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ExportJson"]
    }
}
