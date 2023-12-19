use std::time::Duration;

use bevy::prelude::*;
use bevy::utils::HashMap;
use serde_json::Value;

use swj_utils::unit_team_system;

use crate::cocos2d_anim::{AnimationMode, Cocos2dAnimator, Cocos2dAnimatorPlayer};
use crate::game::GameStates::{Playing, PrepareLoad};
use crate::map::CurrentMapInfo;
use crate::resource::{ConfigResource, ConfigResourceParse};
use crate::resource::action::{Action, ActionType};
use crate::resource::ResourcePath;
use crate::unit::UnitState::Moving;

// macro_rules! unit_team_system {
//     () => {
//         generate_combinations!
//     };
// }

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<UnitSearchMap>()
            .add_systems(OnEnter(PrepareLoad), load_unit_config)
            .add_systems(Update,
                         (
                             unit_z_order,
                             unit_intent_change
                         ).run_if(in_state(Playing)),
            )
            .add_systems(Update, unit_team_system!(
                UnitTeamLeft,
                UnitTeamRight;
                unit_no_attack_sys,
                unit_attack_enemy,
                search_enemy,
                who_attack_me_system,
            ).run_if(in_state(Playing)))
            .add_systems(PreUpdate,
                         (
                             unit_search_prepare_sys,
                         ).run_if(in_state(Playing)),
            )
        ;
    }
}


#[derive(Component, Default, Deref, DerefMut)]
struct WhoAttackMe(u32);

fn who_attack_me_system<T: Component>(
    mut query: Query<(Entity, &mut WhoAttackMe), With<T>>,
    enemy_query: Query<&Enemy, Without<T>>,
) {
    let mut enemy_atk_map = HashMap::new();
    for enemy in enemy_query.iter() {
        *enemy_atk_map.entry(enemy.target).or_insert(0) += 1;
    }

    for (entity, mut who_attack_me) in query.iter_mut() {
        if let Some(count) = enemy_atk_map.get(&entity) {
            who_attack_me.0 = *count;
        } else {
            who_attack_me.0 = 0;
        }
    }
}


#[derive(Resource, Default)]
pub struct UnitSearchMap {
    lefts: Vec<(Entity, Vec2)>,
    rights: Vec<(Entity, Vec2)>,
}

fn unit_search_prepare_sys(
    mut search_map: ResMut<UnitSearchMap>,
    left_query: Query<(Entity, &Transform, &WhoAttackMe), (With<UnitTeamLeft>, With<Unit>)>,
    right_query: Query<(Entity, &Transform, &WhoAttackMe), (With<UnitTeamRight>, With<Unit>)>,
) {
    fn get_sort_key(a: &(Entity, Vec2, u32), left: bool) -> f32 {
        let x_offset = if left {
            -(a.2 as f32 * 1.5)
        } else {
            a.2 as f32 * 1.5
        };

        a.1.x + x_offset
    }

    let mut lefts = Vec::with_capacity(left_query.iter().count());
    for (entity, transform, atk_me) in left_query.iter() {
        lefts.push((entity, transform.translation.truncate(), atk_me.0));
    }
    lefts.sort_by(|a, b| {
        let a_key = get_sort_key(a, true);
        let b_key = get_sort_key(b, true);
        a_key.partial_cmp(&b_key).unwrap().reverse()
    });
    search_map.lefts = lefts.iter().map(|(entity, pos, _)| (*entity, *pos)).collect();

    let mut rights = Vec::with_capacity(right_query.iter().count());
    for (entity, transform, atk_me) in right_query.iter() {
        rights.push((entity, transform.translation.truncate(), atk_me.0));
    }
    rights.sort_by(|a, b| {
        let a_key = get_sort_key(a, false);
        let b_key = get_sort_key(b, false);
        a_key.partial_cmp(&b_key).unwrap()
    });
    search_map.rights = rights.iter().map(|(entity, pos, _)| (*entity, *pos)).collect();
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

fn move_transform_to(transform: &mut Transform, dest_pos: &Vec2, speed: f32) {
    let dir = *dest_pos - transform.translation.truncate();
    let dir = if dir.length() < speed {
        dir
    } else {
        dir.normalize() * speed
    };

    transform.translation += dir.extend(0.);
}

const MIN_DISTANCE_DIFF: f32 = 0.1;

fn unit_no_attack_sys<T: Component>(
    mut query: Query<(&Unit, &mut UnitState, &UnitIntent, &mut Transform, &mut Cocos2dAnimator, &Cocos2dAnimatorPlayer), (With<T>, Without<Enemy>)>,
) {
    for (unit, mut state, intent, mut transform, mut animator, anim_player) in query.iter_mut() {
        match *state {
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

                match *intent {
                    UnitIntent::MoveTo(pos) => {
                        move_transform_to(&mut transform, &pos, unit.move_speed);

                        if transform.translation.truncate().distance(pos) < MIN_DISTANCE_DIFF {
                            *state = UnitState::Idle;
                        }
                    }
                    UnitIntent::AttackTo(pos) => {
                        move_transform_to(&mut transform, &pos, unit.move_speed);

                        if transform.translation.truncate().distance(pos) < MIN_DISTANCE_DIFF {
                            *state = UnitState::Idle;
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

fn unit_attack_enemy<T: Component>(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &Unit, &mut UnitState, &mut Enemy, &mut Transform, &mut Cocos2dAnimator, &Cocos2dAnimatorPlayer), (With<T>, Without<PerformingAction>)>,
    enemy_query: Query<(&Unit, &Transform), Without<T>>,
) {
    for (entity, unit, mut state, mut enemy, mut transform, mut animator, anim_player) in query.iter_mut() {
        match *state {
            UnitState::Moving => {
                if anim_player.anim_name != UnitAnimName::Run.as_str() {
                    animator.new_anim = Some(UnitAnimName::Run.into());
                    animator.mode = AnimationMode::Loop;
                }

                let Enemy { target, attack_range } = *enemy;
                let target_transform = enemy_query.get(target).unwrap().1;
                let target_pos = target_transform.translation.truncate();
                let dir = target_pos - transform.translation.truncate();
                let dir = if dir.length() < attack_range {
                    dir
                } else {
                    dir.normalize() * attack_range
                };
                let target_pos = target_pos - dir;
                move_transform_to(&mut transform, &target_pos, unit.move_speed);

                if transform.translation.truncate().distance(target_transform.translation.truncate()) < attack_range {
                    *state = UnitState::Attacking;
                }
            }
            UnitState::Attacking => {
                if let Ok((enemy_unit, enemy_transform)) = enemy_query.get(enemy.target) {
                    let distance = transform.translation.truncate().distance(enemy_transform.translation.truncate());
                    let mut action = None;
                    for (name, act) in &unit.actions {
                        if !act.is_cd_over(time.elapsed()) {
                            continue;
                        }

                        match act.action_type {
                            ActionType::Melee(ref melee) => {
                                if distance < get_melee_attack_range(unit, enemy_unit) {
                                    action = Some((name, act));
                                    break;
                                }
                            }
                            ActionType::Projectile(ref projectile) => {
                                if act.range.contains(&distance) {
                                    action = Some((name, act));
                                    break;
                                }
                            }
                        }

                        // find some action that satisfy the distance
                        if let Some((name, act)) = action {
                            if anim_player.anim_name != name.as_str() {
                                animator.new_anim = Some(name.clone().into());
                                animator.mode = AnimationMode::Once;
                            }
                            commands.entity(entity).insert(PerformingAction(name.clone()));

                            continue;
                        }

                        // if not then find the farthest action, and move to the range
                        let farthest_range = get_farthest_attack_range(unit, enemy_unit, time.elapsed());
                        *state = Moving;
                        enemy.attack_range = farthest_range;
                    }
                } else {
                    commands.entity(entity).remove::<Enemy>();
                }
            }
            _ => {
                commands.entity(entity).remove::<Enemy>();
            }
        }
    }
}

#[derive(Component)]
struct PerformingAction(String);


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
pub enum UnitState {
    Idle,
    Moving,
    Attacking,
    Dead,
}

#[derive(Component)]
pub enum UnitIntent {
    StandAt(Vec2),
    MoveTo(Vec2),
    AttackTo(Vec2),
}


#[derive(Component)]
pub struct SearchEnemy;

fn search_enemy<T: Component + UnitTeam>(
    time: Res<Time>,
    mut commands: Commands,
    query: Query<(Entity, &Unit, &Transform), (With<T>, With<SearchEnemy>, Without<Enemy>)>,
    enemy_query: Query<(&Unit, &Transform), Without<T>>,
    search_map: Res<UnitSearchMap>,
) {
    let enemy_units = T::enemy_units(&search_map);
    for (entity, unit, transform) in query.iter() {
        let enemy = T::enemy_in_view_range(unit.view_range, &transform.translation.truncate(), enemy_units);
        if let Some(enemy) = enemy {
            if let Ok((enemy_unit, _)) = enemy_query.get(enemy) {
                commands.entity(entity).insert(Enemy {
                    target: enemy,
                    attack_range: get_farthest_attack_range(unit, enemy_unit, time.elapsed()),
                });
            }
        }
    }
}

fn get_melee_attack_range(unit: &Unit, enemy: &Unit) -> f32 {
    (unit.body_radius + enemy.body_radius) / 2.0
}

fn get_farthest_attack_range(unit: &Unit, enemy: &Unit, elapsed_time: Duration) -> f32 {
    let mut max_range = 0.;
    for (_, action) in &unit.actions {
        if !action.is_cd_over(elapsed_time) {
            continue;
        }

        match action.action_type {
            ActionType::Melee(ref melee) => {
                let range = get_melee_attack_range(unit, enemy);
                max_range = range.max(max_range);
            }
            ActionType::Projectile(ref projectile) => {
                let range = action.range.end;
                max_range = range.max(max_range);
            }
        }
    }

    max_range
}

impl UnitIntent {
    pub fn move_to(&mut self, pos: Vec2) {
        *self = UnitIntent::MoveTo(pos);
    }

    pub fn attack_to(&mut self, pos: Vec2) {
        *self = UnitIntent::AttackTo(pos);
    }
}

fn unit_intent_change(
    mut commands: Commands,
    mut query: Query<(Entity, &mut UnitState, &UnitIntent), Changed<UnitIntent>>,
) {
    for (entity, mut state, intent) in query.iter_mut() {
        match intent {
            UnitIntent::MoveTo(_) => {
                *state = UnitState::Moving;
                commands.entity(entity).remove::<SearchEnemy>();
            }
            UnitIntent::AttackTo(_) => {
                *state = UnitState::Moving;
                commands.entity(entity).insert(SearchEnemy);
            }
            _ => {}
        }
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
    pub state: UnitState,
    pub health: UnitHealth,
    pub damage: UnitDamage,
    pub intent: UnitIntent,
    who_attack_me: WhoAttackMe,
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
                move_speed: unit_info.move_speed * 0.02,
                body_radius: unit_info.body_radius,
                body_width: unit_info.body_width,
                body_height: unit_info.body_height,
            },
            state: UnitState::Idle,
            health: UnitHealth {
                health: unit_info.health_base + unit_info.health_factor * level as f32,
                health_recovery_speed: unit_info.health_recovery_speed,
            },
            damage: UnitDamage {
                damage: unit_info.damage_base + unit_info.damage_factor * level as f32,
                damage_fluctuation_range: Vec2::new(unit_info.damage_min_bias, unit_info.damage_max_bias),
            },
            intent: UnitIntent::StandAt(Vec2::ZERO),
            who_attack_me: WhoAttackMe(0),
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
pub struct Enemy {
    pub target: Entity,
    pub attack_range: f32,
}


#[derive(Component)]
pub struct UnitTeamLeft;

#[derive(Component)]
pub struct UnitTeamRight;

trait UnitTeam {
    fn team_units(search_map: &UnitSearchMap) -> &Vec<(Entity, Vec2)>;
    fn enemy_units(search_map: &UnitSearchMap) -> &Vec<(Entity, Vec2)>;
    fn enemy_in_view_range(view_range: f32, pos: &Vec2, enemy_units: &Vec<(Entity, Vec2)>) -> Option<Entity>;
}

impl UnitTeam for UnitTeamLeft {
    fn team_units(search_map: &UnitSearchMap) -> &Vec<(Entity, Vec2)> {
        &search_map.lefts
    }

    fn enemy_units(search_map: &UnitSearchMap) -> &Vec<(Entity, Vec2)> {
        &search_map.rights
    }

    fn enemy_in_view_range(view_range: f32, pos: &Vec2, enemy_units: &Vec<(Entity, Vec2)>) -> Option<Entity> {
        for (entity, enemy_pos) in enemy_units {
            if pos.x + view_range < enemy_pos.x {
                break;
            }

            if pos.x - view_range > enemy_pos.x {
                continue;
            }

            return Some(entity.clone());
        }

        None
    }
}

impl UnitTeam for UnitTeamRight {
    fn team_units(search_map: &UnitSearchMap) -> &Vec<(Entity, Vec2)> {
        &search_map.rights
    }

    fn enemy_units(search_map: &UnitSearchMap) -> &Vec<(Entity, Vec2)> {
        &search_map.lefts
    }

    fn enemy_in_view_range(view_range: f32, pos: &Vec2, enemy_units: &Vec<(Entity, Vec2)>) -> Option<Entity> {
        for (entity, enemy_pos) in enemy_units {
            if pos.x - view_range > enemy_pos.x {
                break;
            }

            if pos.x + view_range < enemy_pos.x {
                continue;
            }

            return Some(entity.clone());
        }

        None
    }
}
