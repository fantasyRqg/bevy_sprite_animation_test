mod progress_map;
mod player;

use bevy::asset::LoadState;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use crate::game::GameStates;
use crate::game::GameStates::{Loading, Playing, PrepareLoad, PrepareScene};
use crate::rpg::player::PlayerPlugin;
use crate::rpg::progress_map::ProgressMapPlugin;

pub struct RpgPlugin;


impl Plugin for RpgPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                PlayerPlugin,
                ProgressMapPlugin,
            ))
            .init_resource::<LoadingAssets>()
            .add_systems(OnEnter(Loading), reset_asset_track)
            .add_systems(Update, check_assets_ready.run_if(in_state(Loading)))
            // .add_systems(OnEnter(PrepareScene), prepare_scene)
        ;
    }
}


#[derive(Resource, Default)]
struct LoadingAssets {
    assets: Vec<UntypedHandle>,
}

fn reset_asset_track(
    mut loading_assets: ResMut<LoadingAssets>,
) {
    loading_assets.assets.clear();
    // next_state.set(Loading);

    info!("Loading assets...")
}

fn check_assets_ready(
    mut next_state: ResMut<NextState<GameStates>>,
    loading_assets: Res<LoadingAssets>,
    asset_server: Res<AssetServer>,
) {
    let mut load_finished = true;

    info!("Checking assets...");

    for a in loading_assets.assets.iter() {
        let load_state = asset_server.get_load_state(a);
        match load_state {
            None => {
                load_finished = false;
                warn!("Asset not found: {:?}", a);
                break;
            }
            Some(load_state) => {
                match load_state {
                    LoadState::NotLoaded | LoadState::Loading => {
                        load_finished = false;
                        break;
                    }
                    LoadState::Loaded => {}
                    LoadState::Failed => {
                        load_finished = false;
                        warn!("Asset failed to load: {:?}", a);

                        // todo: handle failure
                        break;
                    }
                }
            }
        }
    }

    if load_finished {
        next_state.set(PrepareScene);
        info!("Assets loaded");
    }
}

#[derive(SystemParam)]
struct AssetLoader<'w, 's> {
    asset_server: Res<'w, AssetServer>,
    loading_assets: ResMut<'w, LoadingAssets>,
    _marker: std::marker::PhantomData<&'s ()>,
}

impl<'w, 's> AssetLoader<'w, 's> {
    fn load<T: Asset>(&mut self, path: &str) -> Handle<T> {
        let asset: Handle<T> = self.asset_server.load(path.to_string());
        self.loading_assets.assets.push(asset.clone().untyped());
        asset
    }
}