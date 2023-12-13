#![allow(clippy::type_complexity)]

use bevy::{
    prelude::*,
};

use swj::game::GamePlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(ImagePlugin::default_nearest())
        )
        .add_plugins(GamePlugin)

        .run();
}

