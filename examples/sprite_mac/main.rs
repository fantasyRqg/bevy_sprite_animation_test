#![allow(clippy::type_complexity)]

use bevy::{
    prelude::*,
    window::WindowMode,
};

use swj::sprite_tt::SpriteTtPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(ImagePlugin::default_nearest())
        ).add_plugins(SpriteTtPlugin {})
        .run();
}