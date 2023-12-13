use bevy::prelude::*;
use serde_json::Value;
use crate::cocos2d_anim::anim::Cocos2dAnimAsset;

use crate::game::GameStates::PrepareLoad;
use crate::resource::{ConfigResource, ConfigResourceParse};
use crate::unit::UnitType;

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(PrepareLoad), load_projectile_config)
        ;
    }
}


pub struct ProjectileInfo {
    pub effect_name: String,
    pub perform_sound: Vec<String>,
    pub bomb_sound: Vec<String>,
    pub effect_level: u32,
    pub cd_time: f32,
    pub type_allow: Vec<UnitType>,
    pub base_damage: f32,
    pub damage_factor: f32,
    pub distance_min: f32,
    pub distance_max: f32,
    pub bullet_animation_name: String,
    pub fly_speed: f32,
    pub bullet_height_factor: f32,
    pub bullet_height_base: f32,
    pub bullet_damage_radius: f32,
    pub buff_level: Vec<u32>,
    pub buff_list: Vec<String>,
    pub fly_effect: String,
    pub shot_num: i32,
    // src,
    // bombSkill,
    // Level,
    // BombEffect,
    // MissSkill,
    // MissSkillLevel,
    // ChangeTarget,
    // HasTarget,
    // Shader,
    // AOEHeightRange,
    // DelayRange,
    // MotionStreak,
    // Scale,
}

fn load_projectile_config(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    info!("load_projectile_config");
    commands.spawn(ConfigResourceParse {
        handle: asset_server.load("Resources/Configs/ShotBullet.json"),
        parse_fun: parse_projectile_cfg,
        ..default()
    });
}


fn parse_projectile_cfg(content: &str, config_res: &mut ConfigResource) {
    config_res.projectiles.clear();
    let cfgs: Value = serde_json::from_str(content).unwrap();
    for cfg in cfgs.as_array().unwrap() {
        let projectile_cfg = ProjectileInfo {
            effect_name: cfg["EffectName"].as_str().unwrap().to_string(),
            perform_sound: match cfg.get("PerformSound") {
                Some(v) => v.as_str().unwrap().split(",").map(|s| s.to_string()).collect(),
                None => vec![],
            },
            // bomb_sound: cfg["BombSound"].as_str().unwrap().split(",").map(|s| s.to_string()).collect(),
            bomb_sound: match cfg.get("BombSound") {
                Some(v) => v.as_str().unwrap().split(",").map(|s| s.to_string()).collect(),
                None => vec![],
            },
            effect_level: cfg["EffectLevel"].as_i64().unwrap() as u32,
            cd_time: cfg["CDTime"].as_f64().unwrap() as f32,
            type_allow: cfg["TypeAllow"].as_str().unwrap().split(",").map(|s| UnitType::from_str(s).unwrap()).collect(),
            base_damage: cfg["BaseDamage"].as_f64().unwrap() as f32,
            damage_factor: cfg["DamageFactor"].as_f64().unwrap() as f32,
            distance_min: cfg["DistanceMin"].as_f64().unwrap() as f32,
            distance_max: cfg["DistanceMax"].as_f64().unwrap() as f32,
            bullet_animation_name: cfg["BulletAnimationName"].as_str().unwrap().to_string(),
            fly_speed: cfg["FlySpeed"].as_f64().unwrap() as f32,
            bullet_height_factor: cfg["BulletHeightFactor"].as_f64().unwrap() as f32,
            bullet_height_base: cfg["BulletHeightBase"].as_f64().unwrap() as f32,
            bullet_damage_radius: cfg["BulletDamageRadius"].as_f64().unwrap() as f32,
            buff_level: match cfg.get("BuffLevel") {
                Some(v) => v.as_str().unwrap()
                    .split(";")
                    .map(|s| s.parse::<u32>())
                    .filter(|r| r.is_ok())
                    .map(|r| r.unwrap())
                    .collect(),
                None => vec![],
            },
            buff_list: match cfg.get("BuffList") {
                Some(v) => v.as_str().unwrap().split(",").map(|s| s.to_string()).collect(),
                None => vec![],
            },
            fly_effect: match cfg.get("FlyEffect") {
                Some(v) => v.as_str().unwrap().to_string(),
                None => "".to_string(),
            },
            shot_num: cfg["ShotNum"].as_i64().unwrap() as i32,
        };

        if !config_res.projectiles.contains_key(&projectile_cfg.effect_name) {
            config_res.projectiles.insert(projectile_cfg.effect_name.clone(), vec![]);
        }
        config_res.projectiles.get_mut(&projectile_cfg.effect_name).unwrap().push(projectile_cfg);
    }
}


pub enum TargetType {
    Unit(Entity),
    Position(Vec2),
}

#[derive(Component)]
pub struct Projectile {
    pub speed: f32,
    pub damage: f32,
    pub target: TargetType,
    pub emitter: Entity,
}

pub struct ProjectileAct{
    pub base_damage: f32,
    pub damage_factor: f32,
    pub bullet_animation_name: Handle<Cocos2dAnimAsset>,
    pub fly_speed: f32,
    pub bullet_height_factor: f32,
    pub bullet_height_base: f32,
    pub bullet_damage_radius: f32,
    // pub buff_level: Vec<u32>,
    // pub buff_list: Vec<String>,
    pub shot_num: i32,
    pub perform_sound: Vec<Handle<AudioSource>>,
}