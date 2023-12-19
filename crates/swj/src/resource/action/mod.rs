use std::ops::Range;
use std::time::Duration;

use bevy::prelude::*;

use crate::game::GameStates::Playing;
use crate::resource::action::melee::{MeleeAct, MeleeInfo};
use crate::resource::action::projectile::{ProjectileAct, ProjectileInfo};
use crate::resource::ResourcePath;
use crate::unit::UnitHealth;

pub mod projectile;
pub mod melee;


pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<DamageEvent>()
            .add_event::<HealEvent>()
            .add_plugins((
                projectile::ProjectilePlugin,
                melee::MeleePlugin,
            ))
            .add_systems(Update,
                         (
                             damage_system,
                             heal_system
                         ).run_if(in_state(Playing)),
            )
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
    pub range: Range<f32>,
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
            range: Range { start: 0., end: info.damage_radius },
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
            range: Range { start: info.distance_min, end: info.distance_max },
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

    pub fn is_cd_over(&self, time: Duration) -> bool {
        time - self.last_use_time > self.cd_time
    }
}


#[derive(Event)]
pub struct DamageEvent {
    pub src: Entity,
    pub target: Entity,
    pub damage: f32,
}

fn damage_system(
    mut events: EventReader<DamageEvent>,
    mut health: Query<&mut UnitHealth>,
) {
    for DamageEvent { src: _, target, damage } in events.read() {
        if let Ok(mut health) = health.get_mut(*target) {
            health.health -= *damage;
        }
    }
}

#[derive(Event)]
pub struct HealEvent {
    pub src: Entity,
    pub target: Entity,
    pub heal: f32,
}

fn heal_system(
    mut events: EventReader<HealEvent>,
    mut health: Query<&mut UnitHealth>,
) {
    for HealEvent { src: _, target, heal } in events.read() {
        if let Ok(mut health) = health.get_mut(*target) {
            health.health += *heal;
        }
    }
}