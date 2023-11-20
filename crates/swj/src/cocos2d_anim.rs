use std::f32::consts::{FRAC_2_PI, FRAC_PI_2};
use std::f64::consts::FRAC_PI_4;
use bevy::utils::{HashMap, thiserror};
use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    prelude::*,
    reflect::TypePath,
    utils::BoxedFuture,
};
use bevy::asset::{AsyncReadExt};
use bevy::math::{vec2, vec3};
use plist::Dictionary;

use thiserror::Error;
use plist::Value;
use crate::game::GameStates;


pub(crate) struct Cocos2dAnimPlugin;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) enum CocosAnimSet {
    Update,
    AdjustSprite,
}

impl Plugin for Cocos2dAnimPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<PlistSpriteFrameAsset>()
            .configure_sets(Update, (CocosAnimSet::Update, CocosAnimSet::AdjustSprite).chain())
            .init_asset_loader::<PlistSpriteAssetLoader>()
            .init_asset_loader::<PlistSpriteAssetLoader>()
            .add_systems(OnEnter(GameStates::Loading), load_sprites)
            .add_systems(Update, sprite_check_setup.run_if(in_state(GameStates::Loading)))
            .add_systems(Update,
                         (
                             (
                                 animate_sprite.in_set(CocosAnimSet::Update),
                                 update_sprite.in_set(CocosAnimSet::AdjustSprite),
                             ),
                         ).run_if(in_state(GameStates::Playing)),
            )
        ;
    }
}

fn load_sprites(mut commands: Commands,
                asset_server: Res<AssetServer>,
) {
    commands.insert_resource(PlistSpriteCollection {
        plist_sprites: asset_server.load("textures/raw/archer_soldier.plist"),
    });
}

fn sprite_check_setup(
    mut commands: Commands,
    mut events: EventReader<AssetEvent<PlistSpriteFrameAsset>>,
    sprite_collection: Res<PlistSpriteCollection>,
    plist_sprites: Res<Assets<PlistSpriteFrameAsset>>,
    mut next_state: ResMut<NextState<GameStates>>,
) {
    for event in events.read() {
        if event.is_loaded_with_dependencies(sprite_collection.plist_sprites.id()) {
            let ps = plist_sprites.get(sprite_collection.plist_sprites.id()).unwrap();
            commands.spawn((
                SpriteSheetBundle {
                    texture_atlas: ps.atlas.clone(),
                    sprite: TextureAtlasSprite::new(0),
                    ..default()
                },
                PlistAnimation {
                    timer: Timer::from_seconds(1.0 / 30.0, TimerMode::Repeating),
                    plist_frame: sprite_collection.plist_sprites.clone(),
                    ..default()
                },
            ));

            next_state.set(GameStates::Playing);
        }
    }
}


#[derive(Resource)]
struct PlistSpriteCollection {
    plist_sprites: Handle<PlistSpriteFrameAsset>,
}

#[derive(Component)]
pub(crate) struct PlistAnimation {
    timer: Timer,
    plist_frame: Handle<PlistSpriteFrameAsset>,
    last_rotated: f32,
    last_offset: Vec2,
}

impl Default for PlistAnimation {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(1.0 / 30.0, TimerMode::Repeating),
            plist_frame: Handle::default(),
            last_rotated: 0.0,
            last_offset: Vec2::new(0.0, 0.0),
        }
    }
}


fn update_sprite(
    mut query: Query<(&mut Transform, &TextureAtlasSprite, &mut PlistAnimation)>,
    plist_sprite: Res<Assets<PlistSpriteFrameAsset>>,
) {
    for (mut transform, sprite, mut pa) in &mut query {
        let ps = plist_sprite.get(pa.plist_frame.id()).unwrap();
        let sf = &ps.frames[sprite.index];

        transform.translation -= Vec3::from((pa.last_offset, 0.0));
        if pa.last_rotated.abs() > 1e-3 {
            transform.rotate_z(-pa.last_rotated);
        }

        if sprite.flip_x {
            if sf.rotated {
                pa.last_rotated = -FRAC_PI_2;
            } else {
                pa.last_rotated = 0.0;
            }
            pa.last_offset = vec2(-sf.offset.0, sf.offset.1);
        } else {
            if sf.rotated {
                pa.last_rotated = FRAC_PI_2;
            } else {
                pa.last_rotated = 0.0;
            }
            pa.last_offset = vec2(sf.offset.0, sf.offset.1);
        }

        transform.rotate_local_z(pa.last_rotated);
        transform.translation += Vec3::from((pa.last_offset, 0.0));
    }
}

fn animate_sprite(
    time: Res<Time>,
    texture_atlas: Res<Assets<TextureAtlas>>,
    mut query: Query<(&mut TextureAtlasSprite, &mut PlistAnimation, &Handle<TextureAtlas>)>,
) {
    for (mut sprite, mut pa, tex) in &mut query {
        pa.timer.tick(time.delta());
        if pa.timer.just_finished() {
            let tex = texture_atlas.get(tex).unwrap();
            sprite.index = (sprite.index + 1) % tex.len();
        }
    }
}

#[derive(Default)]
pub struct PlistSpriteAssetLoader {}


#[derive(Debug)]
struct SpriteFrame {
    name: String,
    frame: (f32, f32, f32, f32),
    offset: (f32, f32),
    rotated: bool,
    source_color_rect: (f32, f32, f32, f32),
    source_size: (f32, f32),
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

#[derive(Asset, TypePath, Debug)]
pub struct PlistSpriteFrameAsset {
    frames: Vec<SpriteFrame>,
    atlas: Handle<TextureAtlas>,
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
                        max: Vec2::new((sf.frame.0 + sf.frame.3), (sf.frame.1 + sf.frame.2)),
                    },
                    false => Rect {
                        min: Vec2::new(sf.frame.0, sf.frame.1),
                        max: Vec2::new((sf.frame.0 + sf.frame.2), (sf.frame.1 + sf.frame.3)),
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
