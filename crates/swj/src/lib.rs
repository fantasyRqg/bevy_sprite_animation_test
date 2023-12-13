pub mod game;
mod pf_controller;
mod sprite_debug;
pub mod cocos2d_anim;
pub mod map;
pub mod unit;
pub mod effect;
mod resource;
mod clash;

pub(crate) enum AnimChannel {
    Unit = 0,
    Shot,
}

impl From<AnimChannel> for i32 {
    fn from(value: AnimChannel) -> Self {
        value as i32
    }
}

impl From<i32> for AnimChannel {
    fn from(value: i32) -> Self {
        match value {
            0 => AnimChannel::Unit,
            1 => AnimChannel::Shot,
            _ => panic!("unknown anim channel {}", value),
        }
    }
}