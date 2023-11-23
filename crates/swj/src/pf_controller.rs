use std::collections::HashMap;
use std::path::Path;
use bevy::prelude::*;
use rand::prelude::ThreadRng;
use rand::Rng;
use crate::cocos2d_anim::{CocosAnimSet, PlistSpriteFrameAsset};
use crate::cocos2d_anim::PlistAnimation;
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
                             (
                                 move_sprite,
                                 projectile_point,
                                 spawn_event,
                             ).in_set(CocosAnimSet::Update),
                             check_plist_load,
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
    plist_sprite_frame_assets: Res<Assets<PlistSpriteFrameAsset>>,
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
        let unit = new_unit(&unit_infos, &plist_sprite_frame_assets, &mut rng, hw, hh);

        units.push(unit);
    }

    let mut projectiles = Vec::with_capacity(1000);
    for _ in 0..1000 {
        let projectile = new_projectile(&projectile_info.info, &plist_sprite_frame_assets, &mut rng, hw, hh);
        projectiles.push(projectile);
    }

    commands.spawn_batch(units);
    commands.spawn_batch(projectiles);
}

fn new_projectile(projectile_info: &ProjectileInfo, plist_sprite_frame_assets: &Res<Assets<PlistSpriteFrameAsset>>, rng: &mut ThreadRng, hw: f32, hh: f32) -> (SpriteSheetBundle, PlistAnimation, MoveElement, Projectile) {
    let ps = plist_sprite_frame_assets.get(projectile_info.plist_handle.id()).unwrap();
    let dir = Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)).normalize();
    let projectile = (
        SpriteSheetBundle {
            texture_atlas: ps.atlas.clone(),
            sprite: TextureAtlasSprite::new(rng.gen_range(0..ps.frames.len())),
            ..default()
        },
        PlistAnimation {
            timer: Timer::from_seconds(1.0 / 30.0, TimerMode::Repeating),
            plist_frame: projectile_info.plist_handle.clone(),
            transform: Transform {
                translation: Vec3::new(
                    rng.gen_range(-hw..hw),
                    rng.gen_range(-hh..hh),
                    0.0, ),
                rotation: Quat::from_rotation_z(-dir.angle_between(Vec2::X)),
                scale: Vec3::splat(0.5),
                ..default()
            },
            animated: false,
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

fn new_unit(unit_infos: &Res<UnitInfoRes>, plist_sprite_frame_assets: &Res<Assets<PlistSpriteFrameAsset>>, rng: &mut ThreadRng, hw: f32, hh: f32) -> (SpriteSheetBundle, PlistAnimation, MoveElement, Unit) {
    let unit_info = unit_infos.infos.iter().nth(rng.gen_range(0..unit_infos.infos.len())).unwrap();
    let ps = plist_sprite_frame_assets.get(unit_info.1.plist_handle.id()).unwrap();
    let unit = (
        SpriteSheetBundle {
            texture_atlas: ps.atlas.clone(),
            sprite: TextureAtlasSprite::new(0),

            ..default()
        },
        PlistAnimation {
            timer: Timer::from_seconds(1.0 / 30.0, TimerMode::Repeating),
            plist_frame: unit_info.1.plist_handle.clone(),
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
    plist_file: String,
    plist_handle: Handle<PlistSpriteFrameAsset>,
    loaded: bool,
}

#[derive(Resource)]
struct UnitInfoRes {
    infos: HashMap<String, UnitInfo>,
}

struct ProjectileInfo {
    plist_file: String,
    plist_handle: Handle<PlistSpriteFrameAsset>,
    loaded: bool,
}

#[derive(Resource)]
struct ProjectileRes {
    info: ProjectileInfo,
}

impl Default for ProjectileRes {
    fn default() -> Self {
        Self {
            info: ProjectileInfo {
                plist_file: "textures/raw/ammunations.plist".to_string(),
                plist_handle: Handle::default(),
                loaded: false,
            },
        }
    }
}

impl Default for UnitInfoRes {
    fn default() -> Self {
        let mut unit_infos = HashMap::new();

        fn from_plist_file(plist_file: &str) -> (String, UnitInfo) {
            let path = Path::new(plist_file);
            let file_name = path.file_stem().unwrap().to_str().unwrap().to_string();


            (file_name, UnitInfo {
                plist_file: plist_file.to_string(),
                plist_handle: Handle::default(),
                loaded: false,
            })
        }

        let plist_files = vec![
            "textures/raw/archer_soldier.plist",
            "textures/raw/barbarian_archer.plist",
            "textures/raw/barbarian_elite.plist",
            "textures/raw/barbarian_infantry.plist",
            "textures/raw/cabalist.plist",
            "textures/raw/faerie_dragon.plist",
            "textures/raw/fangyan.plist",
            "textures/raw/frog_summon.plist",
        ];

        plist_files.iter().for_each(|&plist_file| {
            let (file_name, unit_info) = from_plist_file(plist_file);
            unit_infos.insert(file_name, unit_info);
        });

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
            unit_info.plist_handle = asset_server.load(unit_info.plist_file.clone());
        }
    }

    projectile_infos.info.plist_handle = asset_server.load(projectile_infos.info.plist_file.clone());
}

fn check_plist_load(
    mut events: EventReader<AssetEvent<PlistSpriteFrameAsset>>,
    mut unit_infos: ResMut<UnitInfoRes>,
    mut next_state: ResMut<NextState<GameStates>>,
    mut projectile_infos: ResMut<ProjectileRes>,
) {
    for event in events.read() {
        for (_, unit_info) in unit_infos.infos.iter_mut() {
            if event.is_loaded_with_dependencies(&unit_info.plist_handle) {
                unit_info.loaded = true;
            }
        }

        if event.is_loaded_with_dependencies(&projectile_infos.info.plist_handle) {
            projectile_infos.info.loaded = true;
        }
    }

    if unit_infos.infos.iter().all(|(_, unit_info)| unit_info.loaded) && projectile_infos.info.loaded {
        next_state.set(GameStates::Playing);
    }
}

fn move_sprite(
    window_size: Res<WindowSize>,
    mut query: Query<(&mut PlistAnimation, &mut TextureAtlasSprite, &mut MoveElement)>,
) {
    for (mut pa, mut sprite, mut elm) in query.iter_mut() {
        let dir = elm.dir;
        let speed = elm.speed;
        let transform = &mut pa.transform;
        transform.translation += Vec3::new(dir.x * speed, dir.y * speed, 0.0);

        if elm.flippable {
            if dir.x > 0.0 {
                sprite.flip_x = false;
            } else if dir.x < 0.0 {
                sprite.flip_x = true;
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
    mut query: Query<(&mut PlistAnimation, &mut TextureAtlasSprite, &mut MoveElement), With<Projectile>>,
) {
    for (mut pa, _sprite, elm) in query.iter_mut() {
        let dir = elm.dir;

        pa.transform.rotation = Quat::from_rotation_z(-dir.angle_between(Vec2::X));
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
    plist_sprite_frame_assets: Res<Assets<PlistSpriteFrameAsset>>,
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
                    let unit = new_unit(&unit_infos, &plist_sprite_frame_assets, &mut rng, hw, hh);

                    units.push(unit);
                }

                commands.spawn_batch(units);
            }
            SpawnEvent::Projectile(num) => {
                let mut projectiles = Vec::with_capacity(*num as usize);
                let mut rng = rand::thread_rng();
                for _ in 0..*num {
                    let projectile = new_projectile(&projectile_info.info, &plist_sprite_frame_assets, &mut rng, hw, hh);
                    projectiles.push(projectile);
                }

                commands.spawn_batch(projectiles);
            }
        }
    }
}