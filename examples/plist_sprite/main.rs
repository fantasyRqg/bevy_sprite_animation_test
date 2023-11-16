#![allow(clippy::type_complexity)]

use bevy::utils::thiserror;
use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    prelude::*,
    reflect::TypePath,
    utils::BoxedFuture,
};
use plist::Dictionary;
use serde::Deserialize;
use thiserror::Error;
use plist::Value;

#[derive(Debug)]
struct SpriteFrame {
    name: String,
    frame: (i32, i32, i32, i32),
    offset: (i32, i32),
    rotated: bool,
    source_color_rect: (i32, i32, i32, i32),
    source_size: (i32, i32),
}


fn parse_frame_from_plist(frame_attr: &Dictionary, frame_name: &str) -> SpriteFrame {
    let mut sf = SpriteFrame {
        name: frame_name.to_string(),
        frame: (0, 0, 0, 0),
        offset: (0, 0),
        rotated: false,
        source_color_rect: (0, 0, 0, 0),
        source_size: (0, 0),
    };

    if let Some(Value::String(s)) = frame_attr.get("frame") {
        let s = s.replace("{", "").replace("}", "");
        let mut frame = s.split(",");
        sf.frame.0 = frame.next().unwrap().parse::<i32>().unwrap();
        sf.frame.1 = frame.next().unwrap().parse::<i32>().unwrap();
        sf.frame.2 = frame.next().unwrap().parse::<i32>().unwrap();
        sf.frame.3 = frame.next().unwrap().parse::<i32>().unwrap();
    }

    if let Some(Value::String(s)) = frame_attr.get("offset") {
        let s = s.replace("{", "").replace("}", "");
        let mut offset = s.split(",");
        sf.offset.0 = offset.next().unwrap().parse::<i32>().unwrap();
        sf.offset.1 = offset.next().unwrap().parse::<i32>().unwrap();
    }

    if let Some(Value::Boolean(b)) = frame_attr.get("rotated") {
        sf.rotated = *b;
    }

    if let Some(Value::String(s)) = frame_attr.get("sourceColorRect") {
        let s = s.replace("{", "").replace("}", "");
        let mut source_color_rect = s.split(",");
        sf.source_color_rect.0 = source_color_rect.next().unwrap().parse::<i32>().unwrap();
        sf.source_color_rect.1 = source_color_rect.next().unwrap().parse::<i32>().unwrap();
        sf.source_color_rect.2 = source_color_rect.next().unwrap().parse::<i32>().unwrap();
        sf.source_color_rect.3 = source_color_rect.next().unwrap().parse::<i32>().unwrap();
    }

    if let Some(Value::String(s)) = frame_attr.get("sourceSize") {
        let s = s.replace("{", "").replace("}", "");
        let mut source_size = s.split(",");
        sf.source_size.0 = source_size.next().unwrap().parse::<i32>().unwrap();
        sf.source_size.1 = source_size.next().unwrap().parse::<i32>().unwrap();
    }

    sf
}


fn load_plist(dict: Value) {
    let mut sprite_frames = vec![];

    if let Value::Dictionary(dict) = dict {
        if let Some(Value::Dictionary(frames)) = dict.get("frames") {
            for (f_name, v) in frames {
                if let Some(frame_attr) = v.as_dictionary() {
                    sprite_frames.push(parse_frame_from_plist(frame_attr, f_name));
                }
            }
        }
    }


    for sf in sprite_frames {
        println!("{:?}", sf);
    }
}

#[derive(Resource)]
struct SpriteRes {
    raw_frames: Vec<SpriteFrame>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, setup)
        .add_systems(Update, animate_sys)
        .run();
}


fn setup(mut commands: Commands,
         sprite_res: Res<SpriteRes>,
         asset_server: Res<AssetServer>,
         mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn(Camera2dBundle::default());
    // commands.spawn((
    //     SpriteSheetBundle {
    //         texture_atlas: texture_atlas_handle,
    //         sprite: TextureAtlasSprite::new(animation_indices.first),
    //         transform: Transform::from_scale(Vec3::splat(6.0)),
    //         ..default()
    //     },
    //     animation_indices,
    // ));
}

fn animate_sys(
    time: Res<Time>,
    texture_atlas: Res<Assets<TextureAtlas>>,
    mut query: Query<(&mut TextureAtlasSprite, &Handle<TextureAtlas>)>,
) {
    for (mut sprite, tex) in &mut query {
        let tex = texture_atlas.get(tex).unwrap();
        sprite.index = (sprite.index + 1) % tex.len();
    }
}

pub struct PlistSpriteAssetLoader {
    texture_atlases: ResMut<'a, Assets<TextureAtlas>>,
}

#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct PlistSpriteAsset {
    pub value: i32,
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum PlistSpriteAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could load shader: {0}")]
    Io(#[from] std::io::Error),
}


impl AssetLoader for PlistSpriteAssetLoader {
    type Asset = PlistSpriteAsset;
    type Settings = ();
    type Error = PlistSpriteAssetLoaderError;

    fn load<'a>(&'a self, reader: &'a mut Reader,
                settings: &'a Self::Settings, load_context: &'a mut LoadContext)
                -> BoxedFuture<'a, Result<Self::Asset, Self::Error>>
    {
        todo!()
    }

    fn extensions(&self) -> &[&str] {
        todo!()
    }
}
