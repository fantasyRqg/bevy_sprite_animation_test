use std::collections::HashMap;
use bevy::math::vec2;
use bevy::prelude::*;
use bevy::prelude::TimerMode::Repeating;
use bevy::utils::Instant;
use rand::prelude::*;
use crate::AnimChannel;
use crate::cocos2d_anim::anim::Cocos2dAnimAsset;
use crate::cocos2d_anim::{AnimationFaceDir, AnimationMode, Cocos2dAnimator};
use crate::game::GameStates;
use crate::game::GameStates::{Loading, Playing, PrepareLoad, PrepareScene};
use crate::map::{TmxMap, TmxMapAsset};
use crate::resource::{ConfigLoaded, ConfigResource, ConfigResourceParse, ResourcePath};
use crate::unit::{get_unit_resources, Unit, UnitAnimName, UnitIntention, UnitTeamLeft};


pub struct ClashPlugin;

impl Plugin for ClashPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<LoadedUnits>()
            .init_resource::<LoadedResource>()
            .init_resource::<UnitGenRes>()
            .add_systems(Update, (
                check_preload_finished.run_if(in_state(PrepareLoad)),
                check_res_load_finished.run_if(in_state(Loading)),
                generate_unit.run_if(in_state(Playing)),
            ))
            .add_systems(OnEnter(PrepareScene), prepare_scene)
        ;
    }
}

#[derive(Resource, Default)]
struct LoadedUnits {
    left_units: Vec<String>,
    right_units: Vec<String>,
}

#[derive(Resource, Default)]
struct LoadedResource {
    anims: HashMap<Handle<Cocos2dAnimAsset>, bool>,
    audios: HashMap<Handle<AudioSource>, bool>,

    map_loaded: bool,
    map_handle: Handle<TmxMapAsset>,
}

fn check_preload_finished(
    mut state: ResMut<NextState<GameStates>>,
    mut loaded_units: ResMut<LoadedUnits>,
    mut loaded_res: ResMut<LoadedResource>,
    config_res: Res<ConfigResource>,
    asset_server: Res<AssetServer>,
    mut cfg_event: EventReader<ConfigLoaded>,
) {
    info!("check_preload_finished");
    if cfg_event.len() == 0 {
        return;
    }


    state.set(Loading);

    let left_units = vec![
        "archer_soldier",
        "malitia_warrior",
    ]
        .iter()
        .map(|s| s.to_string()).collect();

    let right_units = vec![
        "barbarian_archer",
        "barbarian_infantry",
    ]
        .iter()
        .map(|s| s.to_string()).collect();

    loaded_units.right_units = right_units;
    loaded_units.left_units = left_units;


    for un in loaded_units.left_units.iter().chain(loaded_units.right_units.iter()) {
        let (anims, audios) = get_unit_resources(un, &config_res);
        for anim in anims {
            loaded_res.anims.insert(asset_server.load(anim), false);
        }
        for audio in audios {
            loaded_res.audios.insert(asset_server.load(audio), false);
        }
    }

    loaded_res.map_loaded = false;
    loaded_res.map_handle = asset_server.load("Resources/UI/stages/stage1/game_scene_stage1.tmx");
}

fn check_res_load_finished(
    mut anim_events: EventReader<AssetEvent<Cocos2dAnimAsset>>,
    mut audio_events: EventReader<AssetEvent<AudioSource>>,
    mut map_events: EventReader<AssetEvent<TmxMapAsset>>,
    mut state: ResMut<NextState<GameStates>>,
    mut loaded_res: ResMut<LoadedResource>,
) {
    for evt in anim_events.read() {
        for (handle, value) in loaded_res.anims.iter_mut() {
            if evt.is_loaded_with_dependencies(handle) {
                // info!("anim loaded: {:?}", handle);
                *value = true;
            }
        }
    }

    for evt in audio_events.read() {
        for (handle, value) in loaded_res.audios.iter_mut() {
            if evt.is_loaded_with_dependencies(handle) {
                *value = true;
            }
        }
    }


    for evt in map_events.read() {
        if evt.is_loaded_with_dependencies(&loaded_res.map_handle) {
            // info!("map loaded: {:?}", loaded_res.map_handle);
            loaded_res.map_loaded = true;
        }
    }


    if loaded_res.anims.values().all(|v| *v)
        && loaded_res.audios.values().all(|v| *v)
        && loaded_res.map_loaded
    {
        info!("all res loaded");
        state.set(PrepareScene);
    }
}

fn prepare_scene(
    mut commands: Commands,
    loaded_resource: Res<LoadedResource>,
    mut state: ResMut<NextState<GameStates>>,
) {
    commands.spawn(TmxMap {
        handle: loaded_resource.map_handle.clone()
    });

    state.set(Playing);
}


#[derive(Resource)]
struct UnitGenRes {
    timer: Timer,
    left_gen_speed: usize,
    right_gen_speed: usize,
    left_gen_rect: Rect,
    right_gen_rect: Rect,
}

const MAP_SIZE: Vec2 = Vec2 { x: 4672.0, y: 1126.0 };

impl Default for UnitGenRes {
    fn default() -> Self {
        let x_offset = 100.0;
        let left_centre = Vec2::new(-x_offset / 2.0, MAP_SIZE.y / 2.0 - 80.);
        let left_size = vec2(x_offset, 500.0);

        let right_centre = vec2(MAP_SIZE.x + x_offset / 2.0, MAP_SIZE.y / 2.0);
        let right_size = vec2(x_offset, 600.0);

        UnitGenRes {
            timer: Timer::from_seconds(0.2, Repeating),
            left_gen_speed: 5,
            right_gen_speed: 5,
            left_gen_rect: Rect::new(left_centre.x - left_size.x / 2.0, left_centre.y - left_size.y / 2.0, left_centre.x + left_size.x / 2.0, left_centre.y + left_size.y / 2.0),
            right_gen_rect: Rect::new(right_centre.x - right_size.x / 2.0, right_centre.y - right_size.y / 2.0, right_centre.x + right_size.x / 2.0, right_centre.y + right_size.y / 2.0),
        }
    }
}


fn random_pos_in_rect(rect: &Rect, rng: &mut ThreadRng) -> Vec2 {
    let x = rng.gen_range(rect.min.x..rect.max.x);
    let y = rng.gen_range(rect.min.y..rect.max.y);
    vec2(x, y)
}

fn generate_unit(
    time: Res<Time>,
    mut commands: Commands,
    mut unit_gen_res: ResMut<UnitGenRes>,
    loaded_units: Res<LoadedUnits>,
    config_res: Res<ConfigResource>,
    asset_server: Res<AssetServer>,
) {
    if !unit_gen_res.timer.tick(time.delta()).just_finished() {
        return;
    }


    let mut rng = thread_rng();

    let left_gen_speed = unit_gen_res.left_gen_speed;
    let right_gen_speed = unit_gen_res.right_gen_speed;

    let left_gen_rect = unit_gen_res.left_gen_rect;
    let right_gen_rect = unit_gen_res.right_gen_rect;


    let left_gen_count = rng.gen_range(0..left_gen_speed);
    let right_gen_count = rng.gen_range(0..right_gen_speed);

    for _ in 0..left_gen_count {
        let pos = random_pos_in_rect(&left_gen_rect, &mut rng);
        let name = loaded_units.right_units.choose(&mut rng).unwrap();
        let mut unit = Unit::new(name, 1, &config_res, &asset_server);
        unit.intention = UnitIntention::AttackTo(vec2(MAP_SIZE.x, pos.y));
        let unit_info = config_res.units.get(name).unwrap();
        commands.spawn((
            SpatialBundle {
                transform: Transform::from_translation(pos.extend(0.)),
                ..default()
            },
            unit,
            UnitTeamLeft,
            Cocos2dAnimator {
                anim_handle: asset_server.load(unit_info.animation_name.anim_path()),
                face_dir: AnimationFaceDir::Right,
                event_channel: Some(AnimChannel::Unit.into()),
                new_anim: Some(UnitAnimName::Born.into()),
                mode: AnimationMode::Once,
                ..default()
            },
        ));
    }

    for _ in 0..right_gen_count {
        let pos = random_pos_in_rect(&right_gen_rect, &mut rng);
        let name = loaded_units.right_units.choose(&mut rng).unwrap();
        let mut unit = Unit::new(name, 1, &config_res, &asset_server);
        unit.intention = UnitIntention::AttackTo(vec2(0., pos.y));
        let unit_info = config_res.units.get(name).unwrap();
        commands.spawn((
            SpatialBundle {
                transform: Transform::from_translation(pos.extend(0.)),
                ..default()
            },
            unit,
            Cocos2dAnimator {
                anim_handle: asset_server.load(unit_info.animation_name.anim_path()),
                face_dir: AnimationFaceDir::Left,
                event_channel: Some(AnimChannel::Unit.into()),
                new_anim: Some(UnitAnimName::Born.into()),
                mode: AnimationMode::Once,
                ..default()
            },
        ));
    }
}




