use bevy::prelude::*;
use serde_json::Value;
use crate::cocos2d_anim::anim::Cocos2dAnimAsset;
use crate::game::GameStates::PrepareLoad;
use crate::resource::{ConfigResource, ConfigResourceParse};
use crate::unit::UnitType;


pub struct MeleePlugin;

impl Plugin for MeleePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(PrepareLoad), load_melee_config)
        ;
    }
}

fn load_melee_config(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(ConfigResourceParse {
        handle: asset_server.load("Resources/Configs/MeleeHit.json"),
        parse_fun: parse_melee_cfg,
        ..default()
    });
}

#[derive(Debug,Clone)]
pub enum MeleeDamageCenterType {
    Target,
    Src,
}

impl MeleeDamageCenterType {
    fn from_str(s: &str) -> Option<MeleeDamageCenterType> {
        match s {
            "target" => Some(MeleeDamageCenterType::Target),
            "src" => Some(MeleeDamageCenterType::Src),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct MeleeInfo {
    pub effect_name: String,
    pub perform_sound: Vec<String>,
    pub effect_level: u32,
    pub cd_time: f32,
    pub type_allow: Vec<UnitType>,
    pub damage_factor: f32,
    pub damage_center: MeleeDamageCenterType,
    pub damage_radius: f32,
    pub buff_level: Vec<u32>,
    pub buff_list: Vec<String>,
    pub effect_animation: String,
    pub effect_sound: Vec<String>,
}

pub struct MeleeAct {
    pub damage_factor: f32,
    pub damage_center: MeleeDamageCenterType,
    pub damage_radius: f32,
    pub effect_animation: Option<Handle<Cocos2dAnimAsset>>,
    pub effect_sound: Option<Vec<Handle<AudioSource>>>,
    pub perform_sound: Vec<Handle<AudioSource>>,
}


fn parse_melee_cfg(content: &str, config_res: &mut ConfigResource) {
    let json: Value = serde_json::from_str(content).unwrap();

    for m in json.as_array().unwrap() {
        let info = MeleeInfo {
            effect_name: m["EffectName"].as_str().unwrap().to_string(),
            perform_sound: m["PerformSound"].as_str().unwrap().split(",").map(|s| s.to_string()).collect(),
            effect_level: m["EffectLevel"].as_u64().unwrap() as u32,
            cd_time: m["CDTime"].as_f64().unwrap() as f32,
            type_allow: m["TypeAllow"].as_str().unwrap().split(",")
                .map(|s| UnitType::from_str(s))
                .filter(|r| r.is_some())
                .map(|r| r.unwrap())
                .collect(),
            damage_factor: m["DamageFactor"].as_f64().unwrap() as f32,
            damage_center: MeleeDamageCenterType::from_str(m["DamageCenter"].as_str().unwrap()).unwrap(),
            damage_radius: m["DamageRadius"].as_f64().unwrap() as f32,
            buff_level: match m.get("BuffLevel") {
                Some(m) => m.as_str().unwrap().split(";")
                    .map(|s| s.parse())
                    .filter(|r| r.is_ok())
                    .map(|r| r.unwrap())
                    .collect(),
                None => vec![]
            },
            buff_list: match m.get("BuffList") {
                Some(m) => m.as_str().unwrap().split(";")
                    .map(|s| s.to_string())
                    .filter(|s| !s.is_empty())
                    .collect(),
                None => vec![]
            },
            effect_animation: match m.get("EffectAnimation") {
                Some(m) => m.as_str().unwrap().to_string(),
                None => "".to_string()
            },
            effect_sound: match m.get("EffectSound") {
                Some(m) => m.as_str().unwrap().split(",").map(|s| s.to_string()).collect(),
                None => vec![]
            },
        };

        if !config_res.melees.contains_key(&info.effect_name) {
            config_res.melees.insert(info.effect_name.clone(), vec![]);
        }

        config_res.melees.get_mut(&info.effect_name).unwrap().push(info);
    }
}