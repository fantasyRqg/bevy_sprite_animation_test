use bevy::prelude::*;
use serde_json::Value;
use crate::cocos2d_anim::{AnimationMode, Cocos2dAnimator, Cocos2dAnimatorPlayer};

use crate::game::GameStates::{Playing, PrepareLoad};
use crate::map::{CurrentMapInfo, TmxMapBg};
use crate::resource::{ConfigResource, ConfigResourceParse};
use crate::resource::action::Action;
use crate::resource::ResourcePath;

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(PrepareLoad), load_unit_config)
            .add_systems(Update,
                         (
                             unit_z_order,
                             unit_state::<UnitTeamLeft>,
                             unit_state::<UnitTeamRight>,
                         ).run_if(in_state(Playing)),
            )
        ;
    }
}

fn unit_z_order(
    mut query: Query<&mut Transform, With<Unit>>,
    map_info: Res<CurrentMapInfo>,
) {
    let map_height = map_info.size.y;
    for mut transform in query.iter_mut() {
        transform.translation.z = map_height - transform.translation.y;
    }
}


fn load_unit_config(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(ConfigResourceParse {
        handle: asset_server.load("Resources/Configs/Unit.json"),
        parse_fun: parse_unit_cfg,
        ..default()
    });
}

fn move_transform_to(transform: &mut Transform, dest_transform: &Transform, speed: f32) {
    let dir = dest_transform.translation.truncate() - transform.translation.truncate();
    let dir = if dir.length() < speed {
        dir
    } else {
        dir.normalize() * speed
    };

    transform.translation += dir.extend(0.);
}

const MIN_DISTANCE_DIFF: f32 = 0.1;

fn unit_state<T: Component>(
    mut query: Query<(&Unit, &mut UnitStateComponent, &mut Transform, &mut Cocos2dAnimator, &Cocos2dAnimatorPlayer), With<T>>,
    target_query: Query<&Transform, (With<Unit>, Without<T>)>,
) {
    for (unit, mut state, mut transform, mut animator, anim_player) in query.iter_mut() {
        match state.state {
            UnitState::Idle => {
                if anim_player.anim_name != UnitAnimName::Stand.as_str() {
                    animator.new_anim = Some(UnitAnimName::Stand.into());
                    animator.mode = AnimationMode::Loop;
                }
            }
            UnitState::Moving => {
                if anim_player.anim_name != UnitAnimName::Run.as_str() {
                    animator.new_anim = Some(UnitAnimName::Run.into());
                    animator.mode = AnimationMode::Loop;
                }

                match state.intention {
                    UnitIntention::MoveTo(pos) => {
                        move_transform_to(&mut transform, &Transform::from_translation(pos.extend(0.)), unit.move_speed);

                        if transform.translation.truncate().distance(pos) < MIN_DISTANCE_DIFF {
                            state.state = UnitState::Idle;
                        }
                    }
                    UnitIntention::AttackTo(pos) => {
                        if let Some(target) = state.attack_target {
                            if let Ok(target_transform) = target_query.get(target) {
                                move_transform_to(&mut transform, target_transform, unit.move_speed);
                            }

                            if transform.translation.truncate().distance(pos) < MIN_DISTANCE_DIFF {
                                state.state = UnitState::Attacking;
                            }
                        } else {
                            move_transform_to(&mut transform, &Transform::from_translation(pos.extend(0.)), unit.move_speed);

                            if transform.translation.truncate().distance(pos) < MIN_DISTANCE_DIFF {
                                state.state = UnitState::Idle;
                            }
                        }
                    }
                    _ => {}
                }
            }
            UnitState::Attacking => {}
            UnitState::Dead => {
                if anim_player.anim_name != UnitAnimName::Die.as_str() {
                    animator.new_anim = Some(UnitAnimName::Die.into());
                    animator.mode = AnimationMode::Once;
                }
            }
        }
    }
}


pub enum UnitAttackType {
    Melee,
    Range,
    Magic,
}

impl UnitAttackType {
    pub fn from_int(s: i32) -> UnitAttackType {
        match s {
            1 => UnitAttackType::Melee,
            2 => UnitAttackType::Range,
            _ => UnitAttackType::Magic,
        }
    }
}

pub struct UnitActionLevelRule {
    unit_level: u32,
    action_level: u32,
}

pub struct UnitActionRule {
    name: String,
    action: String,
    level_rule: Vec<UnitActionLevelRule>,
}

pub struct UnitInfo {
    pub unit_name: String,
    pub moving_sound: String,
    pub attack_type: UnitAttackType,
    pub can_change: bool,
    pub unit_type: UnitType,
    pub body_width: f32,
    pub body_height: f32,
    pub animation_name: String,
    pub body_radius: f32,
    pub move_speed: f32,
    pub health_base: f32,
    pub health_factor: f32,
    pub health_recovery_speed: f32,
    pub view: f32,
    pub damage_base: f32,
    pub damage_factor: f32,
    pub damage_min_bias: f32,
    pub damage_max_bias: f32,
    pub level_max: u32,
    pub die_skill: String,
    pub run_skill: String,
    pub decoration: String,
    pub actions: Vec<UnitActionRule>,
}

fn parse_unit_cfg(content: &str, config_res: &mut ConfigResource) {
    let json: Value = serde_json::from_str(content).unwrap();
    config_res.units.clear();
    for unit in json.as_array().unwrap() {
        let unit_info = UnitInfo {
            unit_name: unit["UnitName"].as_str().unwrap().to_string(),
            moving_sound: unit["MovingSound"].as_str().unwrap().to_string(),
            attack_type: UnitAttackType::from_int(unit["AttackType"].as_i64().unwrap() as i32),
            can_change: unit["CanChange"].as_str().unwrap() == "yes",
            unit_type: UnitType::from_str(unit["UnitType"].as_str().unwrap()).unwrap(),
            body_width: unit["BodyWidth"].as_f64().unwrap() as f32,
            body_height: unit["BodyHeight"].as_f64().unwrap() as f32,
            animation_name: unit["AnimationName"].as_str().unwrap().to_string(),
            body_radius: unit["BodyRadius"].as_f64().unwrap() as f32,
            move_speed: unit["MoveSpeed"].as_f64().unwrap() as f32,
            health_base: unit["HealthBase"].as_f64().unwrap() as f32,
            health_factor: unit["HealthFactor"].as_f64().unwrap() as f32,
            health_recovery_speed: unit["HealthRecoverySpeed"].as_f64().unwrap() as f32,
            view: unit["View"].as_f64().unwrap() as f32,
            damage_base: unit["DamageBase"].as_f64().unwrap() as f32,
            damage_factor: unit["DamageFactor"].as_f64().unwrap() as f32,
            damage_min_bias: unit["DamageMinBias"].as_f64().unwrap() as f32,
            damage_max_bias: unit["DamageMaxBias"].as_f64().unwrap() as f32,
            level_max: unit["LevelMax"].as_i64().unwrap() as u32,
            die_skill: unit["DieSkill"].as_str().unwrap().to_string(),
            run_skill: unit["RunSkill"].as_str().unwrap().to_string(),
            decoration: unit["Decoration"].as_str().unwrap().to_string(),
            actions: unit["Actions"].as_array().unwrap()
                .iter()
                .map(|action| {
                    UnitActionRule {
                        name: action["Name"].as_str().unwrap().to_string(),
                        action: action["Effect"].as_str().unwrap().to_string(),
                        level_rule: action["EffectLevel"].as_array().unwrap()
                            .iter().map(|level_rule| {
                            UnitActionLevelRule {
                                unit_level: level_rule["UnitLevel"].as_i64().unwrap() as u32,
                                action_level: level_rule["Level"].as_i64().unwrap() as u32,
                            }
                        }).collect(),
                    }
                }).collect(),
        };

        config_res.units.insert(unit_info.unit_name.clone(), unit_info);
    }
}


pub fn get_unit_resources(unit_name: &str, config_res: &ConfigResource) -> (Vec<String>, Vec<String>) {
    let unit_info = config_res.units.get(unit_name).expect(format!("unit {} not found", unit_name).as_str());
    let mut anims = vec![];
    let mut audios = vec![];

    if unit_info.animation_name.is_empty() {
        warn!("unit {} has no animation", unit_name);
        return (anims, audios);
    }

    anims.push(unit_info.animation_name.anim_path());

    for action in &unit_info.actions {
        if config_res.melees.contains_key(&action.action) {
            let info = config_res.melees.get(&action.action).unwrap().first().unwrap();
            if !info.effect_animation.is_empty() {
                anims.push(info.effect_animation.anim_path());
            }
            for sound in &info.perform_sound {
                audios.push(sound.skill_audio_path());
            }
        } else if config_res.projectiles.contains_key(&action.action) {
            let info = config_res.projectiles.get(&action.action).unwrap().first().unwrap();
            if info.bullet_animation_name.is_empty() {
                continue;
            }
            anims.push(info.bullet_animation_name.anim_path());
            for sound in &info.perform_sound {
                audios.push(sound.skill_audio_path());
            }
        }
    }

    (anims, audios)
}

#[derive(Debug)]
pub enum UnitType {
    Ground,
    Air,
}

impl UnitType {
    pub fn from_str(s: &str) -> Option<UnitType> {
        match s {
            "ground" => Some(UnitType::Ground),
            "air" | "sky" => Some(UnitType::Air),
            _ => None,
        }
    }
}

enum UnitState {
    Idle,
    Moving,
    Attacking,
    Dead,
}

pub enum UnitIntention {
    StandAt(Vec2),
    MoveTo(Vec2),
    AttackTo(Vec2),
}

#[derive(Component)]
pub struct Unit {
    pub actions: Vec<(String, Action)>,
    pub view_range: f32,

    pub move_speed: f32,

    pub body_radius: f32,
    pub body_width: f32,
    pub body_height: f32,
}

#[derive(Component)]
pub struct UnitStateComponent {
    pub state: UnitState,
    pub search_enemy: bool,
    pub attack_target: Option<Entity>,
    pub intention: UnitIntention,
}

impl UnitStateComponent {
    pub fn move_to(&mut self, pos: Vec2) {
        self.state = UnitState::Moving;
        self.intention = UnitIntention::MoveTo(pos);
    }

    pub fn attack_to(&mut self, pos: Vec2) {
        self.state = UnitState::Moving;
        self.intention = UnitIntention::AttackTo(pos);
    }
}

#[derive(Component)]
pub struct UnitDamage {
    pub damage: f32,
    pub damage_fluctuation_range: Vec2,
}

#[derive(Component)]
pub struct UnitHealth {
    pub health: f32,
    pub health_recovery_speed: f32,
}


#[derive(Bundle)]
pub struct UnitBundle {
    pub unit: Unit,
    pub state: UnitStateComponent,
    pub health: UnitHealth,
    pub damage: UnitDamage,
}


impl UnitBundle {
    pub fn new(name: &str, level: u32, config_res: &ConfigResource, asset_server: &AssetServer) -> UnitBundle {
        let unit_info = config_res.units.get(name).expect(format!("unit {} not found", name).as_str());
        let mut actions = vec![];
        for action in &unit_info.actions {
            if config_res.melees.contains_key(&action.action) {
                let info = config_res.melees.get(&action.action).unwrap().first().unwrap();
                actions.push((action.name.clone(), Action::from_melee(info, asset_server)));
            } else if config_res.projectiles.contains_key(&action.action) {
                let info = config_res.projectiles.get(&action.action).unwrap().first().unwrap();
                actions.push((action.name.clone(), Action::from_projectile(info, asset_server)));
            }
        }

        // sort by cd time, so longer cd time action will be used first
        actions.sort_by(|a, b| a.1.cd_time.cmp(&b.1.cd_time).reverse());

        UnitBundle {
            unit: Unit {
                actions,
                view_range: unit_info.view,
                move_speed: unit_info.move_speed * 0.03,
                body_radius: unit_info.body_radius,
                body_width: unit_info.body_width,
                body_height: unit_info.body_height,
            },
            state: UnitStateComponent {
                state: UnitState::Idle,
                search_enemy: false,
                attack_target: None,
                intention: UnitIntention::StandAt(Vec2::ZERO),
            },
            health: UnitHealth {
                health: unit_info.health_base + unit_info.health_factor * level as f32,
                health_recovery_speed: unit_info.health_recovery_speed,
            },
            damage: UnitDamage {
                damage: unit_info.damage_base + unit_info.damage_factor * level as f32,
                damage_fluctuation_range: Vec2::new(unit_info.damage_min_bias, unit_info.damage_max_bias),
            },
        }
    }
}

pub enum UnitAnimName {
    Born,
    Stand,
    Run,
    Die,
    Action(String),
}

impl From<String> for UnitAnimName {
    fn from(value: String) -> Self {
        match value.as_str() {
            "born" => UnitAnimName::Born,
            "stand" => UnitAnimName::Stand,
            "run" => UnitAnimName::Run,
            "die" => UnitAnimName::Die,
            _ => UnitAnimName::Action(value),
        }
    }
}

impl From<UnitAnimName> for String {
    fn from(value: UnitAnimName) -> Self {
        value.as_str().to_string()
    }
}

impl UnitAnimName {
    pub fn as_str(&self) -> &str {
        match self {
            UnitAnimName::Born => "born",
            UnitAnimName::Stand => "stand",
            UnitAnimName::Run => "run",
            UnitAnimName::Die => "die",
            UnitAnimName::Action(s) => s.as_str(),
        }
    }
}


#[derive(Component)]
pub struct UnitTeamLeft;

#[derive(Component)]
pub struct UnitTeamRight;