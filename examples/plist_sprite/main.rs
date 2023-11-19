#![allow(clippy::type_complexity)]

use bevy::utils::{HashMap, thiserror};
use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    prelude::*,
    reflect::TypePath,
    utils::BoxedFuture,
};
use bevy::asset::AsyncReadExt;
use plist::Dictionary;
use serde::Deserialize;
use thiserror::Error;
use plist::Value;


#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
enum GameStates {
    #[default]
    Loading,
    Playing,
}

// #[derive(Asset, TypePath, Debug, Deserialize)]
// struct PlistSpriteAsset {
//     sprite_frames: Vec<SpriteFrame>,
//     texture_atlas: Handle<TextureAtlas>,
// }

#[derive(Resource)]
struct PlistSpriteCollection {
    plist_sprites: Handle<PlistSpriteFrameAsset>,
}


fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_state::<GameStates>()
        .add_systems(Startup, setup)
        .add_systems(OnEnter(GameStates::Loading), load_sprites)
        .add_systems(Update, sprite_check_setup.run_if(in_state(GameStates::Loading)))
        .add_systems(OnEnter(GameStates::Playing), spawn_sprite_setup)
        .add_systems(Update, animate_sprite.run_if(in_state(GameStates::Playing)))
        .run();
}


fn load_sprites(mut commands: Commands,
                asset_server: Res<AssetServer>,
) {
    commands.insert_resource(PlistSpriteCollection {
        plist_sprites: asset_server.load("textures/raw/archer_soldier.plist"),
    });
}

fn sprite_check_setup() {}

fn setup(mut commands: Commands,
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

fn spawn_sprite_setup() {}

fn animate_sprite(
    time: Res<Time>,
    texture_atlas: Res<Assets<TextureAtlas>>,
    mut query: Query<(&mut TextureAtlasSprite, &Handle<TextureAtlas>)>,
) {
    for (mut sprite, tex) in &mut query {
        let tex = texture_atlas.get(tex).unwrap();
        sprite.index = (sprite.index + 1) % tex.len();
    }
}

pub struct PlistSpriteAssetLoader<'a> {
    texture_atlases: &'a mut Assets<TextureAtlas>,
}

impl<'a> FromWorld for PlistSpriteAssetLoader<'a> {
    fn from_world(world: &mut World) -> Self {
        let mut texture_atlases = world.get_resource_mut::<Assets<TextureAtlas>>().unwrap();
        PlistSpriteAssetLoader { texture_atlases: &mut *texture_atlases }
    }
}


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

#[derive(Asset, TypePath, Debug)]
pub struct PlistSpriteFrameAsset {
    frame_num: usize,
    atlas: Handle<TextureAtlas>,
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum PlistSpriteAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could load shader: {0}")]
    Io(#[from] std::io::Error),
}


impl AssetLoader for PlistSpriteAssetLoader<'static> {
    type Asset = PlistSpriteFrameAsset;
    type Settings = ();
    type Error = PlistSpriteAssetLoaderError;

    fn load<'a>(&'a self, reader: &'a mut Reader,
                settings: &'a Self::Settings, load_context: &'a mut LoadContext)
                -> BoxedFuture<'a, Result<Self::Asset, Self::Error>>
    {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let dict = plist::from_bytes(&bytes).unwrap();
            let (sprite_frames, tex_name, dims) = load_plist(dict);
            let tex_img = load_context.load(tex_name);
            let mut atlas = TextureAtlas::new_empty(tex_img.clone(), dims);

            for sf in sprite_frames.iter().by_ref() {
                let rect = Rect {
                    min: Vec2::new(sf.frame.0 as f32, sf.frame.1 as f32),
                    max: Vec2::new((sf.frame.0 + sf.frame.2) as f32, (sf.frame.1 + sf.frame.3) as f32),
                };
                atlas.add_texture(rect);
            }

            let atlas_handle = self.texture_atlases.add(atlas);

            Ok(PlistSpriteFrameAsset { frame_num: sprite_frames.len(), atlas: atlas_handle })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["plist"]
    }
}
