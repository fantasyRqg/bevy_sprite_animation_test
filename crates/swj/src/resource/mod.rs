use bevy::asset::{Asset, AssetLoader, AsyncReadExt, BoxedFuture, LoadContext};
use bevy::asset::io::Reader;
use bevy::prelude::*;
use bevy::prelude::{Plugin, TypePath};
use bevy::utils::{HashMap};
use thiserror::Error;
use crate::game::GameStates;
use crate::game::GameStates::{Loading, PrepareLoad};
use crate::resource::action::melee::MeleeInfo;
use crate::resource::action::projectile::ProjectileInfo;
use crate::unit::UnitInfo;

pub mod action;


pub struct ResourcePlugin;

impl Plugin for ResourcePlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<ConfigAsset>()
            .init_resource::<ConfigResource>()
            .init_asset_loader::<ConfigAssetLoader>()
            .add_plugins((
                action::ActionPlugin,
            ))
            .add_systems(Update,
                         (
                             config_res_parse,
                             check_all_config_loaded,
                         ).run_if(in_state(PrepareLoad)),
            )
        ;
    }
}


#[derive(Component)]
pub struct ConfigResourceParse {
    pub handle: Handle<ConfigAsset>,
    pub parse_fun: fn(&str, &mut ConfigResource),
    pub loaded: bool,
}

impl Default for ConfigResourceParse {
    fn default() -> Self {
        ConfigResourceParse {
            handle: Handle::default(),
            parse_fun: |_, _| {},
            loaded: false,
        }
    }
}

fn config_res_parse(
    mut asset_events: EventReader<AssetEvent<ConfigAsset>>,
    mut config_res: ResMut<ConfigResource>,
    config_asset: Res<Assets<ConfigAsset>>,
    mut query: Query<&mut ConfigResourceParse>,
) {
    for event in asset_events.read() {
        for mut arp in query.iter_mut() {
            if event.is_loaded_with_dependencies(&arp.handle) {
                let config = config_asset.get(&arp.handle).unwrap();
                (arp.parse_fun)(&config.content, &mut config_res);
                arp.loaded = true;
            }
        }
    }
}

fn check_all_config_loaded(
    mut commands: Commands,
    query: Query<(Entity, &ConfigResourceParse)>,
    mut next_state: ResMut<NextState<GameStates>>,
    state: Res<State<GameStates>>,
) {
    info!("check_all_config_loaded {} -- {:?} ", query.iter().count(), state);
    if query.iter().count() == 0 {
        return;
    }

    for (_, arp) in query.iter() {
        if !arp.loaded {
            return;
        }
    }

    for (entity, _) in query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    next_state.set(Loading);
}

#[derive(Resource, Default)]
pub struct ConfigResource {
    pub projectiles: HashMap<String, Vec<ProjectileInfo>>,
    pub melees: HashMap<String, Vec<MeleeInfo>>,
    pub units: HashMap<String, UnitInfo>,
}


#[derive(Default)]
pub struct ConfigAssetLoader;

#[derive(Asset, TypePath, Debug)]
pub struct ConfigAsset {
    pub content: String,
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ConfigLoaderError {
    /// An [IO](std::io) Error
    #[error("Could load asset: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for ConfigAssetLoader {
    type Asset = ConfigAsset;
    type Settings = ();
    type Error = ConfigLoaderError;

    fn load<'a>(&'a self, reader: &'a mut Reader,
                _settings: &'a Self::Settings,
                _load_context: &'a mut LoadContext)
                -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut content = String::new();
            reader.read_to_string(&mut content).await?;
            Ok(ConfigAsset {
                content,
            })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}


pub trait ResourcePath {
    fn anim_path(&self) -> String;
    fn skill_audio_path(&self) -> String;
}

impl ResourcePath for String {
    fn anim_path(&self) -> String {
        format!("Resources/Animations/{}.ExportJson", self)
    }

    fn skill_audio_path(&self) -> String {
        format!("Resources/SkillSounds/{}", self)
    }
}