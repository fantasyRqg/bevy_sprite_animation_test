use std::cmp::max;
use std::f32::consts::FRAC_PI_2;
use std::time::Duration;

use bevy::{
    prelude::*,
};
use bevy::time::TimerMode::Repeating;

use anim::Cocos2dAnimAsset;
use sprite_sheet::{PlistSpriteAssetLoader, PlistSpriteFrameAsset};

use crate::cocos2d_anim::anim::Cocos2dAnimAssetLoader;
use crate::cocos2d_anim::AnimationState::Ended;
use crate::cocos2d_anim::EventType::{Custom, End};

pub mod sprite_sheet;
pub mod anim;

pub struct Cocos2dAnimPlugin;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum Cocos2dAnimSet {
    Update,
    AdjustSprite,
}

impl Plugin for Cocos2dAnimPlugin {
    fn build(&self, app: &mut App) {
        app
            .configure_sets(Update, (Cocos2dAnimSet::Update, Cocos2dAnimSet::AdjustSprite).chain())
            .init_asset::<PlistSpriteFrameAsset>()
            .init_asset_loader::<PlistSpriteAssetLoader>()
            .init_asset::<Cocos2dAnimAsset>()
            .init_asset_loader::<Cocos2dAnimAssetLoader>()
            .add_event::<AnimEvent>()
            .add_systems(Update,
                         (
                             (spawn_anim, ).in_set(Cocos2dAnimSet::Update),
                             (animate_sprite, ).in_set(Cocos2dAnimSet::AdjustSprite),
                         ),
            )
        ;
    }
}


pub enum AnimationMode {
    Once,
    Loop,
    Remove,
}

pub enum AnimationFaceDir {
    Left,
    Right,
}

#[derive(Component)]
pub struct Cocos2dAnimator {
    pub duration: Option<Duration>,
    pub anim_handle: Handle<Cocos2dAnimAsset>,
    pub new_anim: Option<String>,
    pub mode: AnimationMode,
    pub event_channel: Option<i32>,
    pub face_dir: AnimationFaceDir,
}

impl Default for Cocos2dAnimator {
    fn default() -> Self {
        Cocos2dAnimator {
            duration: None,
            anim_handle: Handle::default(),
            new_anim: None,
            mode: AnimationMode::Once,
            event_channel: None,
            face_dir: AnimationFaceDir::Right,
        }
    }
}

impl Cocos2dAnimator {
    pub fn switch_anim(&mut self, name: &str) {
        self.new_anim = Some(name.to_string());
    }
}

#[derive(Debug)]
enum AnimationState {
    Playing,
    Ended,
}

#[derive(Component)]
struct Cocos2dAnimatorInner {
    frame_idx: usize,
    timer: Timer,
    anim_name: String,
    state: AnimationState,
}

#[derive(Component)]
pub struct CocoAnim2dAnimatorLayer {
    name: String,
    idx: usize,
}


#[derive(Debug)]
pub enum EventType {
    Custom(String),
    End,
}

#[derive(Event, Debug)]
pub struct AnimEvent {
    pub entity: Entity,
    pub channel: i32,
    pub evt_type: EventType,
}


fn spawn_anim(
    mut commands: Commands,
    animations: Res<Assets<Cocos2dAnimAsset>>,
    query: Query<(Entity, &Cocos2dAnimator), Added<Cocos2dAnimator>>,
) {
    for (entity, cfg) in &mut query.iter() {
        let animation = animations.get(cfg.anim_handle.clone()).unwrap();
        let anim_name = if let Some(name) = &cfg.new_anim {
            name.clone()
        } else {
            warn!("anim name is empty, nothing to play. Set anim name in Cocos2dAnimator component.");
            continue;
        };
        if anim_name.is_empty() || !animation.animation.contains_key(&anim_name) {
            commands.entity(entity).despawn();
            warn!("In {:?} animation, anim {} not found.",cfg.anim_handle, anim_name);
            continue;
        }

        let animation = &animation.animation[&anim_name];
        let interval = if let Some(duration) = cfg.duration {
            duration.as_secs_f32() / max(animation.frame_size - 1, 1) as f32
        } else {
            animation.interval
        };

        commands.entity(entity).insert(Cocos2dAnimatorInner {
            frame_idx: usize::MAX,
            timer: Timer::from_seconds(interval, Repeating),
            state: AnimationState::Playing,
            anim_name,
        })
            .with_children(|parent| {
                for (name, _) in &animation.layers {
                    parent.spawn((
                        CocoAnim2dAnimatorLayer {
                            name: name.clone(),
                            idx: 0,
                        },
                        SpriteSheetBundle {
                            ..default()
                        }
                    ));
                }
            });
    }
}

fn animate_sprite(
    mut commands: Commands,
    time: Res<Time>,
    animations: Res<Assets<Cocos2dAnimAsset>>,
    mut query: Query<(&mut Cocos2dAnimator, &mut Cocos2dAnimatorInner, &Children, Entity)>,
    mut child_query: Query<(&mut TextureAtlasSprite, &mut CocoAnim2dAnimatorLayer, &mut Handle<TextureAtlas>, &mut Transform)>,
    mut events: EventWriter<AnimEvent>,
) {
    for (mut cfg, mut animator, children, entity) in &mut query {
        // info!("animate_sprite: {:?}, interval: {}", animator.state, animator.timer.duration().as_secs_f32());
        if let Some(anim_name) = &cfg.new_anim {
            animator.frame_idx = usize::MAX;
            animator.state = AnimationState::Playing;
            animator.anim_name = anim_name.clone();
            cfg.new_anim = None;
        }

        if matches!(animator.state,Ended) {
            continue;
        }

        animator.timer.tick(time.delta());
        if !animator.timer.just_finished() {
            continue;
        }

        // info!("animate_sprite: {}", animator.frame_idx);

        let animation = animations.get(cfg.anim_handle.clone()).unwrap();
        let animation = &animation.animation[&animator.anim_name];


        if animator.frame_idx == usize::MAX {
            animator.frame_idx = 0;
        } else if animator.frame_idx + 1 >= animation.frame_size {
            match cfg.mode {
                AnimationMode::Once => {
                    animator.state = Ended;
                }
                AnimationMode::Loop => {
                    animator.frame_idx = 0;
                }
                AnimationMode::Remove => {
                    commands.entity(entity)
                        .remove::<Cocos2dAnimator>()
                        .remove::<Cocos2dAnimatorInner>();

                    for child in children.iter() {
                        if child_query.get_mut(*child).is_ok() {
                            commands.entity(*child).despawn_recursive();
                        }
                    }
                }
            }
        } else {
            animator.frame_idx += 1;
        }

        if animator.frame_idx + 1 >= animation.frame_size {
            if let Some(channel) = &cfg.event_channel {
                let evt = AnimEvent {
                    entity: entity.clone(),
                    channel: *channel,
                    evt_type: End,
                };

                events.send(evt);
            }
        }

        // info!("animate_sprite: {}, frame size: {}, name: {}", animator.frame_idx,animation.frame_size,animator.anim_name);

        if matches!(animator.state,Ended) {
            continue;
        }


        for child in children.iter() {
            let (mut sprite,
                mut layer,
                mut atlas,
                mut transform) = match child_query.get_mut(*child) {
                Ok(v) => v,
                Err(_) => {
                    continue;
                }
            };


            let frames = if let Some(frames) = animation.layers.get(&layer.name) {
                frames
            } else {
                warn!("layer {} not found in animation {:?}.", layer.name,cfg.anim_handle);
                continue;
            };


            if animator.frame_idx == 0 {
                layer.idx = 0;
            }

            if layer.idx + 1 < frames.len() {
                let next_frame = &frames[layer.idx + 1];
                if next_frame.fi <= animator.frame_idx {
                    layer.idx += 1;
                }
            }

            let frame = &frames[layer.idx];
            sprite.index = frame.sprite_idx;
            sprite.color = if let Some(color) = frame.color {
                color
            } else {
                Color::WHITE
            };
            *atlas = frame.sprite_atlas.clone();

            sprite.flip_x = match cfg.face_dir {
                AnimationFaceDir::Left => true,
                AnimationFaceDir::Right => false,
            };

            if sprite.flip_x {
                if frame.rotated {
                    transform.rotation = Quat::from_rotation_z(-FRAC_PI_2);
                } else {
                    transform.rotation = Quat::IDENTITY;
                }
                transform.translation = frame.translate;
                transform.translation.x *= -1.0;
                transform.scale = frame.scale.extend(0.0);
            } else {
                if frame.rotated {
                    transform.rotation = Quat::from_rotation_z(FRAC_PI_2) * Quat::IDENTITY;
                } else {
                    transform.rotation = Quat::IDENTITY;
                }

                transform.translation = frame.translate;
                transform.scale = frame.scale.extend(1.0);
            }

            // info!("layer {} set frame: {:?}, transform: {:?}",layer.name, frame,*transform);


            if let Some(evt) = &frame.evt {
                if let Some(channel) = &cfg.event_channel {
                    let evt = AnimEvent {
                        entity: entity.clone(),
                        channel: *channel,
                        evt_type: Custom(evt.clone()),
                    };

                    events.send(evt);
                }
            }
        }
    }
}


