use std::collections::HashMap;
use std::path::Path;

use bevy::prelude::*;
use rand::prelude::*;
use crate::cocos2d_anim::anim::Cocos2dAnimAsset;

use crate::cocos2d_anim::{AnimationFaceDir, AnimationMode, Cocos2dAnimator};
use crate::game::GameStates;

pub(crate) struct PfControllerPlugin;

impl Plugin for PfControllerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<SpawnEvent>()
            .insert_resource(UnitInfoRes::default())
            .insert_resource(ProjectileRes::default())
            .insert_resource(WindowSize(Vec2::new(0.0, 0.0)))
            .add_systems(Startup, load_sprite)
            .add_systems(Update,
                         (
                             move_sprite,
                             projectile_point,
                             spawn_event,
                             check_anim_load.run_if(in_state(GameStates::Loading)),
                             // debug_projectile,
                         ),
            )
            .add_systems(OnEnter(GameStates::Playing), setup_unit_projectile)
        ;
    }
}


fn setup_unit_projectile(
    mut commands: Commands,
    unit_infos: Res<UnitInfoRes>,
    projectile_info: Res<ProjectileRes>,
    windows: Query<&Window>,
    mut window_size: ResMut<WindowSize>,
) {
    let mut units = Vec::with_capacity(100);
    let mut rng = rand::thread_rng();
    let (window_width, window_height) = {
        let window = windows.single();
        (window.width(), window.height())
    };


    let hw = window_width / 2.0;
    let hh = window_height / 2.0;

    window_size.0 = Vec2::new(hw, hh);

    for _ in 0..100 {
        let unit = new_unit(&unit_infos, &mut rng, hw, hh);

        units.push(unit);
    }

    let mut projectiles = Vec::with_capacity(1000);
    for _ in 0..1000 {
        let projectile = new_projectile(&projectile_info.infos, &mut rng, hw, hh);
        projectiles.push(projectile);
    }

    commands.spawn_batch(units);
    commands.spawn_batch(projectiles);
}

fn new_projectile(projectile_info: &HashMap<String, ProjectileInfo>,
                  rng: &mut ThreadRng,
                  hw: f32, hh: f32)
                  -> (Cocos2dAnimator, SpatialBundle, MoveElement, Projectile) {
    let anim_handle = &projectile_info.values().nth(rng.gen_range(0..projectile_info.len())).unwrap().anim_handle;
    let dir = Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)).normalize();
    let projectile = (
        Cocos2dAnimator {
            duration: None,
            anim_handle: anim_handle.clone(),
            new_anim: Some("fly".to_string()),
            mode: AnimationMode::Loop,
            event_channel: Some(0),
            face_dir: AnimationFaceDir::Right,
        },
        SpatialBundle {
            transform: Transform {
                translation: Vec3::new(
                    rng.gen_range(-hw..hw),
                    rng.gen_range(-hh..hh),
                    0.0,
                ),
                scale: Vec3::splat(0.5),
                ..default()
            },
            ..default()
        },
        MoveElement {
            dir: dir,
            speed: rng.gen_range(1.0..5.0),
            flippable: false,
        },
        Projectile {},
    );

    projectile
}

fn new_unit(unit_infos: &Res<UnitInfoRes>,
            rng: &mut ThreadRng,
            hw: f32, hh: f32) -> (Cocos2dAnimator, SpatialBundle, MoveElement, Unit) {
    let unit_info = unit_infos.infos.values().nth(rng.gen_range(0..unit_infos.infos.len())).unwrap();
    let anim_handle = &unit_info.anim_handle;
    let unit = (
        Cocos2dAnimator {
            duration: None,
            anim_handle: anim_handle.clone(),
            new_anim: Some("run".to_string()),
            mode: AnimationMode::Loop,
            event_channel: Some(0),
            face_dir: AnimationFaceDir::Right,
        },
        SpatialBundle {
            transform: Transform {
                translation: Vec3::new(
                    rng.gen_range(-hw..hw),
                    rng.gen_range(-hh..hh),
                    0.0,
                ),
                scale: Vec3::splat(0.5),
                ..default()
            },
            ..default()
        },
        MoveElement {
            dir: Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)).normalize(),
            speed: rng.gen_range(1.0..5.0),
            flippable: true,
        },
        Unit {},
    );
    unit
}

#[derive(Resource, Deref, DerefMut)]
struct WindowSize(Vec2);

#[derive(Component)]
pub struct Unit {}

#[derive(Component)]
pub struct Projectile {}

#[derive(Component)]
struct MoveElement {
    dir: Vec2,
    speed: f32,
    flippable: bool,
}


struct UnitInfo {
    anim_file: String,
    anim_handle: Handle<Cocos2dAnimAsset>,
    loaded: bool,
}

#[derive(Resource)]
struct UnitInfoRes {
    infos: HashMap<String, UnitInfo>,
}

struct ProjectileInfo {
    anim_file: String,
    anim_handle: Handle<Cocos2dAnimAsset>,
    loaded: bool,
}

#[derive(Resource)]
struct ProjectileRes {
    infos: HashMap<String, ProjectileInfo>,
}

impl Default for ProjectileRes {
    fn default() -> Self {
        let anim_json = vec![
            "Resources/Animations/Rifleman_zd.ExportJson",
            "Resources/Animations/ammu_arrow01.ExportJson",
            "Resources/Animations/MagicBallPurple.ExportJson",
            // "Resources/Animations/TrollsArrow.ExportJson",
            "Resources/Animations/ammu_spear02.ExportJson",
            "Resources/Animations/ArrowBlueAnim.ExportJson",
            "Resources/Animations/yellow_fire_ball.ExportJson",
            "Resources/Animations/spear_white.ExportJson",
            "Resources/Animations/ammu_spear01.ExportJson",
            "Resources/Animations/multi_arrow_oragne.ExportJson",
            "Resources/Animations/Rifleman_zd_2.ExportJson",
            "Resources/Animations/grenade_fangyan.ExportJson",
            "Resources/Animations/magic_ball_green.ExportJson",
            "Resources/Animations/SmallBlueMagicBall.ExportJson",
            // "Resources/Animations/FireRain.ExportJson",
            "Resources/Animations/FireBall.ExportJson",
            // "Resources/Animations/ArrowAnimForSkKing.ExportJson",
            // "Resources/Animations/ArrowAnim.ExportJson",
            "Resources/Animations/yellow_boom.ExportJson",
            "Resources/Animations/mgaic_ball_purple_02.ExportJson",
            "Resources/Animations/b_archer_arrow_02.ExportJson",
            "Resources/Animations/shotgun_fangyan.ExportJson",
            "Resources/Animations/hailong_spear01.ExportJson",
            // "Resources/Animations/ArtillerySupport.ExportJson",
            "Resources/Animations/bomb_fangyan_01.ExportJson",
            "Resources/Animations/DarkArrow.ExportJson",
            "Resources/Animations/ammu_arrow02.ExportJson",
            "Resources/Animations/b_archer_arrow_01.ExportJson",
            "Resources/Animations/IceBall.ExportJson",
            "Resources/Animations/hailong_spear_thunder_blow.ExportJson",
            "Resources/Animations/burning_rock.ExportJson",
            "Resources/Animations/MageBall.ExportJson",
            // "Resources/Animations/MagicBallBlue.ExportJson",
            "Resources/Animations/MagicBallGreenNew.ExportJson",
            // "Resources/Animations/MageBallYellow.ExportJson",
            "Resources/Animations/StormHammer.ExportJson",
            "Resources/Animations/throw_bomb.ExportJson",
            "Resources/Animations/magic_ball_red.ExportJson",
            "Resources/Animations/ThunderHummerBullet.ExportJson",
            "Resources/Animations/MultiBlueArrowForWenling.ExportJson",
            "Resources/Animations/ammu_rock.ExportJson",
            "Resources/Animations/HellfireRockfallGreen.ExportJson",
            "Resources/Animations/arrow_red.ExportJson",
            "Resources/Animations/BlackArrow.ExportJson",
        ];

        let projectile_infos = anim_json.iter()
            .map(|&anim_file| {
                let path = Path::new(anim_file);
                let file_name = path.file_stem().unwrap().to_str().unwrap().to_string();
                (file_name, ProjectileInfo {
                    anim_file: anim_file.to_string(),
                    anim_handle: Handle::default(),
                    loaded: false,
                })
            })
            .collect::<HashMap<String, ProjectileInfo>>();

        Self {
            infos: projectile_infos,
        }
    }
}

impl Default for UnitInfoRes {
    fn default() -> Self {
        let anim_json = vec![
            "Resources/Animations/mohe.ExportJson",
            "Resources/Animations/duanhe.ExportJson",
            "Resources/Animations/boss_pig_red.ExportJson",
            "Resources/Animations/barbarian_infantry.ExportJson",
            "Resources/Animations/demolisher.ExportJson",
            "Resources/Animations/malitia_archer.ExportJson",
            "Resources/Animations/wenling.ExportJson",
            "Resources/Animations/summon_big_spider_c3.ExportJson",
            "Resources/Animations/beast_master.ExportJson",
            "Resources/Animations/fire_stone_monster.ExportJson",
            "Resources/Animations/knife_shield_soldier.ExportJson",
            "Resources/Animations/barbarian_boss.ExportJson",
            "Resources/Animations/huge_bear.ExportJson",
            "Resources/Animations/summoned_spirte_purple.ExportJson",
            "Resources/Animations/summon_big_spider_c1.ExportJson",
            "Resources/Animations/haitang.ExportJson",
            "Resources/Animations/boss_skeleton.ExportJson",
            "Resources/Animations/crab.ExportJson",
            "Resources/Animations/barbarian_archer.ExportJson",
            "Resources/Animations/magic_armer.ExportJson",
            "Resources/Animations/summon_big_spider.ExportJson",
            "Resources/Animations/dread_lord.ExportJson",
            "Resources/Animations/cavalry.ExportJson",
            "Resources/Animations/white_wolf_king.ExportJson",
            "Resources/Animations/boss_beetle.ExportJson",
            "Resources/Animations/barbarian_tuo.ExportJson",
            "Resources/Animations/devil_blades.ExportJson",
            "Resources/Animations/spear_soldier.ExportJson",
            "Resources/Animations/devil_guard.ExportJson",
            "Resources/Animations/cabalist.ExportJson",
            "Resources/Animations/malitia_warrior.ExportJson",
            "Resources/Animations/devil_firebrand.ExportJson",
            "Resources/Animations/wolf_knight.ExportJson",
            "Resources/Animations/shanmu.ExportJson",
            "Resources/Animations/babarian_gaint.ExportJson",
            "Resources/Animations/skeleton_mage.ExportJson",
            "Resources/Animations/green_hell_fire.ExportJson",
            "Resources/Animations/hero_two_blades.ExportJson",
            "Resources/Animations/leopard_girl.ExportJson",
            "Resources/Animations/fruit_bat.ExportJson",
            "Resources/Animations/malitia_spear.ExportJson",
            "Resources/Animations/faerie_dragon.ExportJson",
            "Resources/Animations/pig_blue.ExportJson",
            "Resources/Animations/bat.ExportJson",
            "Resources/Animations/cabalist_chen.ExportJson",
            "Resources/Animations/catapult.ExportJson",
            "Resources/Animations/hailong.ExportJson",
            "Resources/Animations/fangyan.ExportJson",
            "Resources/Animations/pig_king.ExportJson",
            "Resources/Animations/summon_big_spider_c2.ExportJson",
            "Resources/Animations/skeleton_elite_archer.ExportJson",
            "Resources/Animations/sickle_blade.ExportJson",
            "Resources/Animations/frog.ExportJson",
            "Resources/Animations/boss_bat.ExportJson",
            "Resources/Animations/pig.ExportJson",
            "Resources/Animations/barbarian_elite_02.ExportJson",
            "Resources/Animations/barbarian_elite.ExportJson",
            "Resources/Animations/witch_doctor.ExportJson",
            "Resources/Animations/griffin.ExportJson",
            "Resources/Animations/archer_soldier.ExportJson",
            "Resources/Animations/red_hell_fire.ExportJson",
            "Resources/Animations/stone_monster.ExportJson",
            "Resources/Animations/frog_summon.ExportJson",
        ];

        let unit_infos = anim_json.iter()
            .map(|&anim_file| {
                let path = Path::new(anim_file);
                let file_name = path.file_stem().unwrap().to_str().unwrap().to_string();
                (file_name, UnitInfo {
                    anim_file: anim_file.to_string(),
                    anim_handle: Handle::default(),
                    loaded: false,
                })
            })
            .collect::<HashMap<String, UnitInfo>>();

        Self {
            infos: unit_infos,
        }
    }
}

fn load_sprite(
    asset_server: Res<AssetServer>,
    mut unit_infos: ResMut<UnitInfoRes>,
    mut projectile_infos: ResMut<ProjectileRes>,
) {
    for (_, unit_info) in unit_infos.infos.iter_mut() {
        if !unit_info.loaded {
            unit_info.anim_handle = asset_server.load(unit_info.anim_file.clone());
        }
    }

    for (_, projectile_info) in projectile_infos.infos.iter_mut() {
        if !projectile_info.loaded {
            projectile_info.anim_handle = asset_server.load(projectile_info.anim_file.clone());
        }
    }
}

fn check_anim_load(
    mut events: EventReader<AssetEvent<Cocos2dAnimAsset>>,
    mut unit_infos: ResMut<UnitInfoRes>,
    mut next_state: ResMut<NextState<GameStates>>,
    mut projectile_infos: ResMut<ProjectileRes>,
) {
    for event in events.read() {
        for (_, unit_info) in unit_infos.infos.iter_mut() {
            if event.is_loaded_with_dependencies(&unit_info.anim_handle) {
                unit_info.loaded = true;
            }
        }

        for (_, projectile_info) in projectile_infos.infos.iter_mut() {
            if event.is_loaded_with_dependencies(&projectile_info.anim_handle) {
                projectile_info.loaded = true;
            }
        }
    }

    if unit_infos.infos.iter().all(|(_, unit_info)| unit_info.loaded)
        && projectile_infos.infos.iter().all(|(_, projectile_info)| projectile_info.loaded) {
        next_state.set(GameStates::Playing);
        info!("All anim loaded");
    }
}

fn move_sprite(
    window_size: Res<WindowSize>,
    mut query: Query<(&mut Cocos2dAnimator, &mut Transform, &mut MoveElement)>,
) {
    for (mut animator, mut transform, mut elm) in query.iter_mut() {
        let dir = elm.dir;
        let speed = elm.speed;
        transform.translation += Vec3::new(dir.x * speed, dir.y * speed, 0.0);

        if elm.flippable {
            if dir.x > 0.0 {
                animator.face_dir = AnimationFaceDir::Right;
            } else if dir.x < 0.0 {
                animator.face_dir = AnimationFaceDir::Left;
            }
        }

        if transform.translation.x > window_size.0.x {
            elm.dir.x = -elm.dir.x.abs();
        } else if transform.translation.x < -window_size.0.x {
            elm.dir.x = elm.dir.x.abs();
        }

        if transform.translation.y > window_size.0.y {
            elm.dir.y = -elm.dir.y.abs();
        } else if transform.translation.y < -window_size.0.y {
            elm.dir.y = elm.dir.y.abs();
        }
        transform.translation.z = transform.translation.y;
    }
}

fn projectile_point(
    mut query: Query<(&mut Transform, &MoveElement), With<Projectile>>,
) {
    for (mut transform, elm) in query.iter_mut() {
        let dir = elm.dir;
        transform.rotation = Quat::from_rotation_z(-dir.angle_between(Vec2::X));
    }
}

#[derive(Event)]
pub enum SpawnEvent {
    Unit(i32),
    Projectile(i32),
}

fn spawn_event(
    mut commands: Commands,
    mut events: EventReader<SpawnEvent>,
    unit_infos: Res<UnitInfoRes>,
    projectile_info: Res<ProjectileRes>,
    window_size: Res<WindowSize>,
) {
    let hw = window_size.x;
    let hh = window_size.y;

    for event in events.read() {
        match event {
            SpawnEvent::Unit(num) => {
                let mut units = Vec::with_capacity(*num as usize);
                let mut rng = rand::thread_rng();
                for _ in 0..*num {
                    let unit = new_unit(&unit_infos, &mut rng, hw, hh);

                    units.push(unit);
                }

                commands.spawn_batch(units);
            }
            SpawnEvent::Projectile(num) => {
                let mut projectiles = Vec::with_capacity(*num as usize);
                let mut rng = rand::thread_rng();
                for _ in 0..*num {
                    let projectile = new_projectile(&projectile_info.infos, &mut rng, hw, hh);
                    projectiles.push(projectile);
                }

                commands.spawn_batch(projectiles);
            }
        }
    }
}

fn debug_projectile(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    projectile_info: Res<ProjectileRes>,
    mut projectile_query: Query<(Entity), With<Projectile>>,
    mut index: Local<usize>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for entity in projectile_query.iter_mut() {
            commands.entity(entity).despawn_recursive();
        }

        let (name, projectile) = projectile_info.infos.iter().nth(*index % projectile_info.infos.len()).unwrap();
        info!("Spawn projectile: {}", name);
        let projectile = (
            Cocos2dAnimator {
                duration: None,
                anim_handle: projectile.anim_handle.clone(),
                new_anim: Some("fly".to_string()),
                mode: AnimationMode::Loop,
                event_channel: Some(0),
                face_dir: AnimationFaceDir::Right,
            },
            SpatialBundle {
                transform: Transform {
                    translation: Vec3::new(0.0, 0.0, 0.0),
                    // scale: Vec3::splat(0.5),
                    ..default()
                },
                ..default()
            },
            MoveElement {
                dir: Vec2::new(1.0, 0.0),
                speed: 1.0,
                flippable: false,
            },
            Projectile {},
        );

        commands.spawn(projectile);
        *index += 1;
    }
}