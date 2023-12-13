use std::time::Duration;
use bevy::asset::{AssetLoader, AsyncReadExt};
use bevy::prelude::*;
use crate::resource::action::melee::{MeleeAct, MeleeInfo};
use crate::resource::action::projectile::{ProjectileAct, ProjectileInfo};
use crate::resource::ResourcePath;

pub mod projectile;
pub mod melee;


pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                projectile::ProjectilePlugin,
                melee::MeleePlugin,
            ))
        ;
    }
}

pub enum ActionType {
    Melee(MeleeAct),
    Projectile(ProjectileAct),
}

pub struct Action {
    pub name: String,
    pub action_level: u32,
    pub cd_time: Duration,
    pub last_use_time: Duration,
    pub range: Vec2,
    pub action_type: ActionType,
}

impl Action {
    pub fn from_melee(info: &MeleeInfo, asset_server: &AssetServer) -> Action {
        let has_effect = !info.effect_animation.is_empty();

        Action {
            name: info.effect_name.clone(),
            action_level: info.effect_level,
            cd_time: Duration::from_secs_f32(info.cd_time),
            last_use_time: Duration::from_secs(0),
            range: Vec2::new(0., info.damage_radius),
            action_type: ActionType::Melee(MeleeAct {
                damage_factor: info.damage_factor,
                damage_center: info.damage_center.clone(),
                damage_radius: info.damage_radius,
                effect_animation: if has_effect { Some(asset_server.load(info.effect_animation.anim_path())) } else { None },
                effect_sound: if has_effect {
                    Some(info.effect_sound.iter().map(|s| asset_server.load(s.skill_audio_path())).collect())
                } else {
                    None
                },
                perform_sound: info.perform_sound.iter().map(|s| asset_server.load(s.skill_audio_path())).collect(),
            }),
        }
    }

    pub fn from_projectile(info: &ProjectileInfo, asset_server: &AssetServer) -> Action {
        Action {
            name: info.effect_name.clone(),
            action_level: info.effect_level,
            cd_time: Duration::from_secs_f32(info.cd_time),
            last_use_time: Duration::from_secs(0),
            range: Vec2::new(info.distance_min, info.distance_max),
            action_type: ActionType::Projectile(ProjectileAct {
                base_damage: info.base_damage,
                damage_factor: info.damage_factor,
                bullet_animation_name: asset_server.load(info.bullet_animation_name.anim_path()),
                fly_speed: info.fly_speed,
                bullet_height_factor: info.bullet_height_factor,
                bullet_height_base: info.bullet_height_base,
                bullet_damage_radius: info.bullet_damage_radius,
                shot_num: info.shot_num,
                perform_sound: info.perform_sound.iter().map(|s| asset_server.load(s.skill_audio_path())).collect(),
            }),
        }
    }
}