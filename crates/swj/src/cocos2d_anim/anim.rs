use bevy::{
    asset::{AssetLoader, io::Reader, LoadContext},
    prelude::*,
    reflect::TypePath,
    utils::BoxedFuture,
};
use bevy::asset::AsyncReadExt;
use bevy::utils::{HashMap, thiserror};
use plist::Dictionary;
use serde_json::Value;
use thiserror::Error;
use crate::cocos2d_anim::sprite_sheet::{PlistSpriteAssetLoaderError, PlistSpriteFrameAsset};


struct TextureData {
    size: Vec2,
    plist_file: String,
    px: f32,
    py: f32,
}

struct DisplayData {
    name: String,
    texture_data: TextureData,
}

struct BoneData {
    name: String,
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

            Ok(CocosAnim2dAsset {})
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ExportJson"]
    }
}
