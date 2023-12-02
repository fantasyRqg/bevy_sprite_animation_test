use bevy::{
    asset::{AssetLoader, io::Reader, LoadContext},
    prelude::*,
    reflect::TypePath,
    utils::BoxedFuture,
};
use bevy::asset::AsyncReadExt;
use bevy::utils::thiserror;
use plist::Dictionary;
use plist::Value;
use thiserror::Error;

#[derive(Default)]
pub struct PlistSpriteAssetLoader {}


#[derive(Debug, Clone)]
pub struct SpriteFrame {
    pub(crate) name: String,
    pub(crate) frame: (f32, f32, f32, f32),
    pub(crate) offset: (f32, f32),
    pub(crate) rotated: bool,
    pub(crate) source_color_rect: (f32, f32, f32, f32),
    pub(crate) source_size: (f32, f32),
}

fn parse_frame_from_plist(frame_attr: &Dictionary, frame_name: &str) -> SpriteFrame {
    let mut sf = SpriteFrame {
        name: frame_name.to_string(),
        frame: (0.0f32, 0.0f32, 0.0f32, 0.0f32),
        offset: (0.0f32, 0.0f32),
        rotated: false,
        source_color_rect: (0.0f32, 0.0f32, 0.0f32, 0.0f32),
        source_size: (0.0f32, 0.0f32),
    };

    if let Some(Value::String(s)) = frame_attr.get("frame") {
        let s = s.replace("{", "").replace("}", "");
        let mut frame = s.split(",");
        sf.frame.0 = frame.next().unwrap().parse::<i32>().unwrap() as f32;
        sf.frame.1 = frame.next().unwrap().parse::<i32>().unwrap() as f32;
        sf.frame.2 = frame.next().unwrap().parse::<i32>().unwrap() as f32;
        sf.frame.3 = frame.next().unwrap().parse::<i32>().unwrap() as f32;
    }

    if let Some(Value::String(s)) = frame_attr.get("offset") {
        let s = s.replace("{", "").replace("}", "");
        let mut offset = s.split(",");
        sf.offset.0 = offset.next().unwrap().parse::<i32>().unwrap() as f32;
        sf.offset.1 = offset.next().unwrap().parse::<i32>().unwrap() as f32;
    }

    if let Some(Value::Boolean(b)) = frame_attr.get("rotated") {
        sf.rotated = *b;
    }

    if let Some(Value::String(s)) = frame_attr.get("sourceColorRect") {
        let s = s.replace("{", "").replace("}", "");
        let mut source_color_rect = s.split(",");
        sf.source_color_rect.0 = source_color_rect.next().unwrap().parse::<i32>().unwrap() as f32;
        sf.source_color_rect.1 = source_color_rect.next().unwrap().parse::<i32>().unwrap() as f32;
        sf.source_color_rect.2 = source_color_rect.next().unwrap().parse::<i32>().unwrap() as f32;
        sf.source_color_rect.3 = source_color_rect.next().unwrap().parse::<i32>().unwrap() as f32;
    }

    if let Some(Value::String(s)) = frame_attr.get("sourceSize") {
        let s = s.replace("{", "").replace("}", "");
        let mut source_size = s.split(",");
        sf.source_size.0 = source_size.next().unwrap().parse::<i32>().unwrap() as f32;
        sf.source_size.1 = source_size.next().unwrap().parse::<i32>().unwrap() as f32;
    }

    sf
}


fn load_plist(dict: Value) -> (Vec<SpriteFrame>, String, Vec2) {
    let mut sprite_frames = vec![];

    let mut tex_file_name: String = "".to_string();
    let mut dims: Vec2 = Vec2::new(0.0, 0.0);

    if let Value::Dictionary(dict) = dict {
        if let Some(Value::Dictionary(frames)) = dict.get("frames") {
            for (f_name, v) in frames {
                if let Some(frame_attr) = v.as_dictionary() {
                    sprite_frames.push(parse_frame_from_plist(frame_attr, f_name));
                }
            }
        }

        if let Some(Value::Dictionary(metadata)) = dict.get("metadata") {
            if let Some(Value::String(tex_file)) = metadata.get("realTextureFileName") {
                tex_file_name = tex_file.clone();
            }

            if let Some(Value::String(size)) = metadata.get("size") {
                let size = size.replace("{", "").replace("}", "");
                let mut size = size.split(",");
                let w = size.next().unwrap().parse::<f32>().unwrap();
                let h = size.next().unwrap().parse::<f32>().unwrap();
                dims = Vec2::new(w, h);
            }
        }
    }

    (sprite_frames, tex_file_name, dims)
}


#[derive(Asset, TypePath, Debug, Clone)]
pub struct PlistSpriteFrameAsset {
    pub frames: Vec<SpriteFrame>,
    pub atlas: Handle<TextureAtlas>,
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum PlistSpriteAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could load shader: {0}")]
    Io(#[from] std::io::Error),
}


impl AssetLoader for PlistSpriteAssetLoader {
    type Asset = PlistSpriteFrameAsset;
    type Settings = ();
    type Error = PlistSpriteAssetLoaderError;

    fn load<'a>(&'a self, reader: &'a mut Reader,
                _settings: &'a Self::Settings, load_context: &'a mut LoadContext)
                -> BoxedFuture<'a, Result<Self::Asset, Self::Error>>
    {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let dict = plist::from_bytes(&bytes).unwrap();
            let (sprite_frames, tex_name, dims) = load_plist(dict);
            let tex_img = load_context.load(load_context.path().parent().unwrap().join(tex_name));
            let mut atlas = TextureAtlas::new_empty(tex_img.clone(), dims);

            for sf in sprite_frames.iter().by_ref() {
                let rect = match sf.rotated {
                    true => Rect {
                        min: Vec2::new(sf.frame.0, sf.frame.1),
                        max: Vec2::new(sf.frame.0 + sf.frame.3, sf.frame.1 + sf.frame.2),
                    },
                    false => Rect {
                        min: Vec2::new(sf.frame.0, sf.frame.1),
                        max: Vec2::new(sf.frame.0 + sf.frame.2, sf.frame.1 + sf.frame.3),
                    },
                };
                atlas.add_texture(rect);
            }

            let atlas_handle = load_context.add_labeled_asset("atlas".to_string(), atlas);
            Ok(PlistSpriteFrameAsset { frames: sprite_frames, atlas: atlas_handle })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["plist"]
    }
}
