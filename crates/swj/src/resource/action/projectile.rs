use bevy::color::palettes::basic::RED;
use bevy::prelude::*;
use serde_json::Value;
use crate::cocos2d_anim::anim::Cocos2dAnimAsset;
use crate::cocos2d_anim::{AnimationMode, Cocos2dAnimator};

use crate::game::GameStates::PrepareLoad;
use crate::map::CurrentMapInfo;
use crate::resource::{ConfigResource, ConfigResourceParse};
use crate::resource::action::DamageEvent;
use crate::unit::{get_unit_aim_body_offset, Unit, UnitState, UnitType};

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(PrepareLoad), load_projectile_config)
            .add_systems(Update,
                         (
                             projectile_fly_to_fixed_target,
                             projectile_finish_fly,
                             projectile_miss,
                             // debug_projectile,
                         ))
        ;
    }
}


fn debug_projectile(
    mut gizmos: Gizmos,
    query: Query<(&Projectile, &ProjectileToFixedTarget, &Transform)>,
) {
    for (projectile, to_target, transform) in query.iter() {
        gizmos.line_2d(
            to_target.src_pos,
            to_target.dest_pos,
            RED,
        );
    }
}

fn projectile_fly_to_fixed_target(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &Projectile, &ProjectileToFixedTarget, &mut Transform), With<ProjectileFly>>,
    map_info: Res<CurrentMapInfo>,
) {
    for (entity, projectile, to_target, mut transform) in query.iter_mut() {
        let delta_time = if time.elapsed_seconds() - to_target.start_time > to_target.fly_duration {
            commands.entity(entity).remove::<ProjectileFly>();
            to_target.fly_duration
        } else {
            time.elapsed_seconds() - to_target.start_time
        };
        let delta_pos = to_target.init_velocity * delta_time + 0.5 * to_target.acceleration * delta_time * delta_time;
        let pos = to_target.src_pos + delta_pos;

        transform.translation = pos.extend(map_info.size.y - pos.y + 1000.);
        let cur_velocity = to_target.init_velocity + to_target.acceleration * delta_time;
        transform.rotation = Quat::from_rotation_z(cur_velocity.y.atan2(cur_velocity.x));

        // info!("projectile_fly_to_fixed_target: delta_time: {}, delta_pos: {:?}, pos: {:?}, cur_velocity: {:?}", delta_time, delta_pos, pos, cur_velocity);
    }
}

fn projectile_finish_fly(
    mut commands: Commands,
    mut rm_fly: RemovedComponents<ProjectileFly>,
    mut damage_writer: EventWriter<DamageEvent>,
    mut query: Query<(Entity, &Projectile, &mut Transform, &mut Cocos2dAnimator, &ProjectileToFixedTarget), (Without<ProjectileFly>, Without<UnitState>)>,
    unit_query: Query<(&Unit, &Transform), With<UnitState>>,
    query_all: Query<&Projectile>,
    map_info: Res<CurrentMapInfo>,
) {
    let projectile_count = query_all.iter().count();
    for entity in rm_fly.read() {
        if let Ok((entity, projectile, mut transform, mut animator, fixed_target)) = query.get_mut(entity) {
            let pos = transform.translation.truncate();
            transform.translation.z = map_info.size.y - pos.y;
            match projectile.target {
                TargetType::Unit(target_entity) => {
                    let unit_pos = if let Ok((unit, unit_transform)) = unit_query.get(target_entity) {
                        unit_transform.translation.truncate() + get_unit_aim_body_offset(unit)
                    } else {
                        Vec2::ZERO
                    };

                    // info!("projectile_finish_fly: pos: {:?}, unit_pos: {:?}, distance: {}, tolerance: {}, fixed target: {:?}",
                    //     pos, unit_pos, unit_pos.distance(pos), projectile.tolerance, fixed_target.dest_pos);
                    if unit_pos.distance(pos) > projectile.tolerance {
                        //miss
                        if projectile_count > 2000 {
                            commands.entity(entity).despawn_recursive();
                            continue;
                        }

                        animator.new_anim = Some("missEnd".to_string());
                        animator.mode = AnimationMode::Once;

                        commands.entity(entity)
                            .insert(ProjectileMiss(Timer::from_seconds(5.5, TimerMode::Once)));
                    } else {
                        //hit
                        damage_writer.send(DamageEvent {
                            src: projectile.src,
                            target: target_entity,
                            damage: projectile.damage,
                        });
                        commands.entity(entity).despawn_recursive();
                    }
                }
                TargetType::Position(target_pos) => {
                    panic!("not implement");
                }
            }
        }
    }
}

#[derive(Component, Deref, DerefMut)]
struct ProjectileMiss(Timer);

fn projectile_miss(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ProjectileMiss)>,
) {
    for (entity, mut miss) in query.iter_mut() {
        miss.tick(time.delta());
        if miss.just_finished() {
            commands.entity(entity).despawn_recursive();
        }
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
    pub damage: f32,
    pub target: TargetType,
    pub src: Entity,
    pub tolerance: f32,
}

#[derive(Component)]
pub struct ProjectileToFixedTarget {
    pub acceleration: Vec2,
    pub init_velocity: Vec2,
    pub dest_pos: Vec2,
    pub src_pos: Vec2,
    pub fly_duration: f32,
    pub start_time: f32,
}

#[derive(Component)]
pub struct ProjectileFly;

#[derive(Debug)]
pub struct ProjectileAct {
    pub base_damage: f32,
    pub damage_factor: f32,
    pub bullet_animation_handle: Handle<Cocos2dAnimAsset>,
    pub fly_speed: f32,
    pub bullet_height_factor: f32,
    pub bullet_height_base: f32,
    pub bullet_damage_radius: f32,
    // pub buff_level: Vec<u32>,
    // pub buff_list: Vec<String>,
    pub shot_num: i32,
    pub perform_sound: Vec<Handle<AudioSource>>,
}