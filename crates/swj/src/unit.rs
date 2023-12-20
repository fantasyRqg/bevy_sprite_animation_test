use std::ops::Range;
use std::time::Duration;
use bevy::input::ButtonState;
use bevy::input::mouse::MouseButtonInput;

use bevy::prelude::*;
use rand::Rng;
use serde_json::Value;

use swj_utils::unit_team_system;
use crate::AnimChannel;

use crate::cocos2d_anim::{AnimationFaceDir, AnimationMode, AnimEvent, Cocos2dAnimator, Cocos2dAnimatorPlayer, EventType};
use crate::game::GameStates::{Playing, PrepareLoad};
use crate::map::CurrentMapInfo;
use crate::resource::{ConfigResource, ConfigResourceParse};
use crate::resource::action::{Action, ActionType, DamageEvent};
use crate::resource::action::melee::MeleeDamageCenterType;
use crate::resource::ResourcePath;
use crate::unit::UnitState::Moving;

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<UnitSearchMap>()
            .add_systems(OnEnter(PrepareLoad), load_unit_config)
            .add_systems(Update,
                         (
                             unit_z_order,
                             unit_intent_change,
                             performing_action,
                             enemy_removed,
                             health_system,
                             unit_anim_event,
                             unit_die,
                             debug_system,
                         ).run_if(in_state(Playing)),
            )
            .add_systems(Update, unit_team_system!(
                UnitTeamLeft,
                UnitTeamRight;
                unit_no_attack_sys,
                unit_attack_enemy,
                find_enemy,
                enemy_added,
                action_anim_event,
            ).run_if(in_state(Playing)))
            .add_systems(PreUpdate,
                         (
                             unit_search_prepare_sys,
                         ).run_if(in_state(Playing)),
            )
        ;
    }
}

fn debug_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform), With<Unit>>,
    info_query: Query<(&Cocos2dAnimator, &Cocos2dAnimatorPlayer), With<Unit>>,
    mut mouse_btn: EventReader<MouseButtonInput>,
    mut cursor_move: EventReader<CursorMoved>,
    mut cur_pos: Local<Vec2>,
    mut drag: Local<Option<Entity>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    map_info: Res<CurrentMapInfo>,
) {
    let (camera, camera_trans) = camera_query.single();

    for event in cursor_move.read() {
        if let Some(pos) = camera.viewport_to_world_2d(camera_trans, event.position) {
            *cur_pos = pos;
        }

        if let Some(entity) = *drag {
            if let Ok((_, mut transform)) = query.get_mut(entity) {
                transform.translation.x = cur_pos.x;
                transform.translation.y = cur_pos.y;
            }
        }
    }


    let mut most_close_entity = None;
    let mut min_distance = 1000000.0;

    for event in mouse_btn.read() {
        if event.button == MouseButton::Left && event.state == ButtonState::Pressed {
            for (entity, transform) in query.iter() {
                let pos = transform.translation.truncate();
                let distance = pos.distance(*cur_pos);
                if distance < min_distance {
                    min_distance = distance;
                    most_close_entity = Some(entity);
                }
            }

            *drag = most_close_entity;
        }

        if event.button == MouseButton::Left && event.state == ButtonState::Released {
            *drag = None;
        }
    }

    if let Some(entity) = most_close_entity {
        commands.entity(entity).log_components();
        if let Ok((animator, anim_player)) = info_query.get(entity) {
            info!("anim: {:?}, {:?}", animator, anim_player);
        }
    }

    // let mut entities = vec![];
    //
    // for (entity, player) in query.iter() {
    //     if player.anim_name != "die" {
    //         entities.push(entity);
    //     }
    // }
    //
    // for entity in last_entities.iter() {
    //     if !entities.contains(entity) {
    //         commands.entity(*entity).log_components();
    //     }
    // }
    //
    // *last_entities = entities;
}


#[derive(Component, Default, Deref, DerefMut)]
struct WhoAttackMe(u32);

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
    fn get_sort_key(pos: &Vec2, atk_me: u32, left: bool) -> f32 {
        let offset_factor = 105.5;
        let x_offset = if left {
            -(atk_me as f32 * offset_factor)
        } else {
            atk_me as f32 * offset_factor
        };

        pos.x + x_offset
    }

    let mut lefts = Vec::with_capacity(left_query.iter().count());
    for (entity, transform, atk_me) in left_query.iter() {
        let pos = transform.translation.truncate();
        lefts.push((entity, pos, get_sort_key(&pos, atk_me.0, true)));
    }
    lefts.sort_by(|a, b| {
        let a_key = a.2;
        let b_key = b.2;
        a_key.partial_cmp(&b_key).unwrap().reverse()
    });
    search_map.lefts = lefts.iter().map(|(entity, pos, _)| (*entity, *pos)).collect();

    let mut rights = Vec::with_capacity(right_query.iter().count());
    for (entity, transform, atk_me) in right_query.iter() {
        let pos = transform.translation.truncate();
        rights.push((entity, pos, get_sort_key(&pos, atk_me.0, false)));
    }
    rights.sort_by(|a, b| {
        let a_key = a.2;
        let b_key = b.2;
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

fn move_transform_to(transform: &mut Transform, dest_pos: &Vec2, speed: f32) -> Option<AnimationFaceDir> {
    let dir = *dest_pos - transform.translation.truncate();
    let dir = if dir.length() < speed {
        dir
    } else {
        dir.normalize() * speed
    };

    transform.translation += dir.extend(0.);


    if dir.x.abs() > 1e-5 {
        if dir.x > 0.0 {
            Some(AnimationFaceDir::Right)
        } else {
            Some(AnimationFaceDir::Left)
        }
    } else {
        None
    }
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

                let mut face_dir = None;

                match *intent {
                    UnitIntent::MoveTo(pos) => {
                        face_dir = move_transform_to(&mut transform, &pos, unit.move_speed);

                        if transform.translation.truncate().distance(pos) < MIN_DISTANCE_DIFF {
                            *state = UnitState::Idle;
                        }
                    }
                    UnitIntent::AttackTo(pos) => {
                        face_dir = move_transform_to(&mut transform, &pos, unit.move_speed);

                        if transform.translation.truncate().distance(pos) < MIN_DISTANCE_DIFF {
                            *state = UnitState::Idle;
                        }
                    }
                    _ => {}
                }

                if let Some(face_dir) = face_dir {
                    if face_dir != animator.face_dir {
                        animator.face_dir = face_dir;
                    }
                }
            }
            UnitState::Attacking => {}
        }
    }
}

fn unit_attack_enemy<T: Component>(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &Unit, &mut UnitState, &mut Enemy, &mut Transform, &mut Cocos2dAnimator, &Cocos2dAnimatorPlayer), (With<T>, Without<PerformingAction>)>,
    enemy_query: Query<(&Unit, &Transform), (Without<T>, With<UnitState>)>,
) {
    for (entity, unit, mut state, mut enemy, mut transform, mut animator, anim_player) in query.iter_mut() {
        match *state {
            UnitState::Moving => {
                if anim_player.anim_name != UnitAnimName::Run.as_str() {
                    animator.new_anim = Some(UnitAnimName::Run.into());
                    animator.mode = AnimationMode::Loop;
                }

                let Enemy { target, attack_range } = *enemy;
                let (enemy_unit, enemy_transform) = if let Ok(target) = enemy_query.get(target) {
                    target
                } else {
                    commands.entity(entity).remove::<Enemy>();
                    continue;
                };
                let target_pos = enemy_transform.translation.truncate();
                let my_pos = transform.translation.truncate();
                let dir = target_pos - my_pos;
                let dir = if dir.length() < attack_range {
                    dir
                } else {
                    dir.normalize() * attack_range
                };
                let target_pos = target_pos - dir;
                // info!("move to target {:?}, attack range: {}, {:?} -> {:?}", entity,transform.translation.truncate(), target_pos, attack_range);
                let face_dir = move_transform_to(&mut transform, &target_pos, unit.move_speed);
                if let Some(face_dir) = face_dir {
                    if face_dir != animator.face_dir {
                        animator.face_dir = face_dir;
                    }
                }

                if my_pos.distance(target_pos) < 1e-2 {
                    *state = UnitState::Attacking;
                }
            }
            UnitState::Attacking => {
                if let Ok((enemy_unit, enemy_transform)) = enemy_query.get(enemy.target) {
                    let distance = transform.translation.truncate().distance(enemy_transform.translation.truncate());
                    let mut action = None;
                    for idx in 0..unit.actions.len() {
                        let (name, act) = &unit.actions[idx];
                        if !act.is_cd_over(time.elapsed()) {
                            continue;
                        }

                        match act.action_type {
                            ActionType::Melee(ref melee) => {
                                if distance < get_melee_attack_range(unit, enemy_unit) {
                                    action = Some((idx, name, act));
                                    break;
                                }
                            }
                            ActionType::Projectile(ref projectile) => {
                                if act.range.contains(&distance) {
                                    action = Some((idx, name, act));
                                    break;
                                }
                            }
                        }
                    }

                    // find some action that satisfy the distance
                    if let Some((idx, name, act)) = action {
                        if anim_player.anim_name != name.as_str() {
                            animator.new_anim = Some(name.clone().into());
                            animator.mode = AnimationMode::Once;
                        }
                        commands.entity(entity).insert(PerformingAction {
                            idx,
                            name: name.clone(),
                        });

                        if transform.translation.x > enemy_transform.translation.x {
                            animator.face_dir = AnimationFaceDir::Left;
                        } else {
                            animator.face_dir = AnimationFaceDir::Right;
                        }

                        continue;
                    }

                    // if not then find the farthest action, and move to the range
                    let farthest_range = get_farthest_attack_range(unit, enemy_unit, time.elapsed());
                    *state = Moving;
                    enemy.attack_range = farthest_range;
                    // info!("can not perform action on target, keep moving, {:?} -> {:?}",entity,enemy.target);
                } else {
                    // info!("not find target unit");
                    commands.entity(entity).remove::<Enemy>();
                    *state = Moving;
                }
            }
            _ => {
                commands.entity(entity).remove::<Enemy>();
            }
        }
    }
}

#[derive(Component)]
struct PerformingAction {
    idx: usize,
    name: String,
}

fn performing_action(
    time: Res<Time>,
    mut query: Query<(&mut Unit, &PerformingAction, &mut Cocos2dAnimator), (Added<PerformingAction>, Without<UnitDead>)>,
) {
    for (mut unit, action, mut animator) in query.iter_mut() {
        let (name, ref mut action) = &mut unit.actions[action.idx];
        action.last_use_time = time.elapsed();
        animator.new_anim = Some(name.clone());
        animator.mode = AnimationMode::Once;
        animator.event_channel = Some(AnimChannel::UnitAction.into());
    }
}


fn action_anim_event<T: Component + UnitTeam>(
    mut commands: Commands,
    mut events: EventReader<AnimEvent>,
    mut damage_event: EventWriter<DamageEvent>,
    unit_search_map: Res<UnitSearchMap>,
    query: Query<(&Unit, &UnitDamage, &Transform, &PerformingAction, &Enemy), With<T>>,
) {
    let mut rng = rand::thread_rng();

    for AnimEvent { entity, channel, evt_type } in events.read() {
        let channel = AnimChannel::from(*channel);
        if !matches!(channel, AnimChannel::UnitAction) {
            continue;
        }

        match evt_type {
            EventType::Custom(msg) => {
                if msg != "perform" {
                    continue;
                }
                if let Ok((unit, unit_damage, transform, pa, enemy)) = query.get(entity.clone()) {
                    let (name, action) = &unit.actions[pa.idx];
                    match action.action_type {
                        ActionType::Melee(ref act) => {
                            let pos = match act.damage_center {
                                MeleeDamageCenterType::Target => {
                                    T::enemy_units(&unit_search_map).iter().find(|(e, _)| {
                                        *e == enemy.target
                                    }).unwrap().1.clone()
                                }
                                MeleeDamageCenterType::Src => {
                                    transform.translation.truncate()
                                }
                            };

                            let mut enemies = if act.damage_radius > 1e-1 {
                                vec![enemy.target]
                            } else {
                                let mut enemies = T::enemies_in_range(&action.range,
                                                                      &pos,
                                                                      T::enemy_units(&unit_search_map),
                                                                      100);
                                if !enemies.contains(&enemy.target) {
                                    enemies.push(enemy.target);
                                }
                                enemies
                            };


                            let damage = unit_damage.damage + rng.gen_range(unit_damage.damage_fluctuation_range.clone());
                            let damage = damage * act.damage_factor * 10.;

                            // info!("send damage event to enemies: {:?}", enemies);

                            damage_event.send_batch(
                                enemies.iter()
                                    .map(|e| {
                                        DamageEvent {
                                            src: entity.clone(),
                                            target: *e,
                                            damage,
                                        }
                                    })
                            );
                        }
                        ActionType::Projectile(ref act) => {}
                    }
                }
            }
            EventType::End => {
                commands.entity(entity.clone()).remove::<PerformingAction>();
            }
        }
    }
}

pub enum UnitAttackType {
    Melee,
    Magic,
    Ranger,
}

impl UnitAttackType {
    pub fn from_int(s: i32) -> UnitAttackType {
        match s {
            1 => UnitAttackType::Melee,
            2 => UnitAttackType::Ranger,
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
}

#[derive(Component)]
pub enum UnitIntent {
    StandAt(Vec2),
    MoveTo(Vec2),
    AttackTo(Vec2),
}


#[derive(Component)]
pub struct SearchEnemy;

fn find_enemy<T: Component + UnitTeam>(
    time: Res<Time>,
    mut commands: Commands,
    query: Query<(Entity, &Unit, &Transform), (With<T>, With<SearchEnemy>, Without<Enemy>)>,
    enemy_query: Query<&Unit, Without<T>>,
    search_map: Res<UnitSearchMap>,
) {
    let enemy_units = T::enemy_units(&search_map);
    for (entity, unit, transform) in query.iter() {
        let enemy = T::enemy_in_view_range(unit.view_range, &transform.translation.truncate(), enemy_units);
        if let Some(enemy) = enemy {
            if let Ok(enemy_unit) = enemy_query.get(enemy) {
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

    max_range - 0.1
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
    pub damage_fluctuation_range: Range<f32>,
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
                damage_fluctuation_range: Range { start: unit_info.damage_min_bias, end: unit_info.damage_max_bias },
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

fn enemy_added<T: Component>(
    query: Query<&Enemy, (Added<Enemy>, With<T>)>,
    mut enemy_query: Query<&mut WhoAttackMe, Without<T>>,
) {
    for enemy in query.iter() {
        if let Ok(mut who_attack_me) = enemy_query.get_mut(enemy.target) {
            who_attack_me.0 += 1;
        }
    }
}

fn enemy_removed(
    mut removed: RemovedComponents<Enemy>,
    mut enemy_query: Query<&mut WhoAttackMe>,
) {
    for entity in removed.read() {
        if let Ok(mut who_attack_me) = enemy_query.get_mut(entity) {
            if who_attack_me.0 > 0 {
                who_attack_me.0 -= 1;
            }
        }
    }
}


#[derive(Component)]
pub struct UnitTeamLeft;

#[derive(Component)]
pub struct UnitTeamRight;

trait UnitTeam {
    fn team_units(search_map: &UnitSearchMap) -> &Vec<(Entity, Vec2)>;
    fn enemy_units(search_map: &UnitSearchMap) -> &Vec<(Entity, Vec2)>;
    fn enemy_in_view_range(view_range: f32, pos: &Vec2, enemy_units: &Vec<(Entity, Vec2)>) -> Option<Entity>;

    fn enemies_in_range(range: &Range<f32>, pos: &Vec2, enemy_units: &Vec<(Entity, Vec2)>, max_units: usize) -> Vec<Entity>;
    fn teammates_in_range(range: &Range<f32>, pos: &Vec2, team_units: &Vec<(Entity, Vec2)>, max_units: usize) -> Vec<Entity>;
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

    fn enemies_in_range(range: &Range<f32>, pos: &Vec2, enemy_units: &Vec<(Entity, Vec2)>, max_units: usize) -> Vec<Entity> {
        let mut enemies = vec![];
        for (entity, enemy_pos) in enemy_units {
            if pos.x + range.end < enemy_pos.x {
                break;
            }

            if pos.x - range.end > enemy_pos.x {
                continue;
            }

            info!("any one reach here ?");
            if !range.contains(&pos.distance(*enemy_pos)) {
                continue;
            }

            enemies.push(entity.clone());
            if enemies.len() >= max_units {
                break;
            }
        }

        enemies
    }

    fn teammates_in_range(range: &Range<f32>, pos: &Vec2, team_units: &Vec<(Entity, Vec2)>, max_units: usize) -> Vec<Entity> {
        let mut teammates = vec![];
        for (entity, team_pos) in team_units {
            if pos.x + range.end < team_pos.x {
                break;
            }

            if pos.x - range.end > team_pos.x {
                continue;
            }

            if !range.contains(&pos.distance(*team_pos)) {
                continue;
            }

            teammates.push(entity.clone());
            if teammates.len() >= max_units {
                break;
            }
        }

        teammates
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

    fn enemies_in_range(range: &Range<f32>, pos: &Vec2, enemy_units: &Vec<(Entity, Vec2)>, max_units: usize) -> Vec<Entity> {
        let mut enemies = vec![];
        for (entity, enemy_pos) in enemy_units {
            if pos.x - range.end > enemy_pos.x {
                break;
            }

            if pos.x + range.end < enemy_pos.x {
                continue;
            }

            if !range.contains(&pos.distance(*enemy_pos)) {
                continue;
            }

            enemies.push(entity.clone());
            if enemies.len() >= max_units {
                break;
            }
        }

        enemies
    }

    fn teammates_in_range(range: &Range<f32>, pos: &Vec2, team_units: &Vec<(Entity, Vec2)>, max_units: usize) -> Vec<Entity> {
        let mut teammates = vec![];
        for (entity, team_pos) in team_units {
            if pos.x - range.end > team_pos.x {
                break;
            }

            if pos.x + range.end < team_pos.x {
                continue;
            }

            if !range.contains(&pos.distance(*team_pos)) {
                continue;
            }

            teammates.push(entity.clone());
            if teammates.len() >= max_units {
                break;
            }
        }

        teammates
    }
}

#[derive(Bundle)]
struct LivingUintBundle {
    intent: UnitIntent,
    enemy: Enemy,
    state: UnitState,
    who_attack_me: WhoAttackMe,
    search_enemy: SearchEnemy,
    perform_action: PerformingAction,
}

#[derive(Component)]
struct UnitDead;

fn health_system(
    mut commands: Commands,
    mut query: Query<(Entity, &UnitHealth), (Changed<UnitHealth>)>,
) {
    for (entity, health) in query.iter_mut() {
        if health.health <= 0.0 {
            commands.entity(entity)
                .remove::<LivingUintBundle>()
                .insert(UnitDead);
        }
    }
}

fn unit_die(
    mut query: Query<&mut Cocos2dAnimator, (Added<UnitDead>, With<Unit>)>,
) {
    for mut animator in query.iter_mut() {
        animator.new_anim = Some(UnitAnimName::Die.into());
        animator.mode = AnimationMode::Once;
        animator.event_channel = Some(AnimChannel::Unit.into());
    }
}

fn unit_anim_event(
    mut commands: Commands,
    mut events: EventReader<AnimEvent>,
    mut query: Query<(Option<&mut UnitState>, Option<&UnitIntent>, &Cocos2dAnimatorPlayer)>,
) {
    for evt in events.read() {
        let AnimEvent { entity, channel, evt_type } = evt;
        let channel = AnimChannel::from(*channel);
        if !matches!(channel, AnimChannel::Unit) {
            continue;
        }

        match evt_type {
            EventType::End => {
                if let Ok((mut state, intent, anim_player)) = query.get_mut(entity.clone()) {
                    match anim_player.anim_name.as_str() {
                        "die" => {
                            commands.entity(entity.clone()).despawn_recursive();
                        }
                        "born" => {
                            let intent = if let Some(intent) = intent {
                                intent
                            } else {
                                warn!("unit born but no intent");
                                continue;
                            };

                            let mut state = if let Some(state) = state {
                                state
                            } else {
                                warn!("unit born but no state");
                                continue;
                            };

                            match *intent {
                                UnitIntent::MoveTo(_) => {
                                    *state = Moving;
                                }
                                UnitIntent::AttackTo(_) => {
                                    *state = Moving;
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}