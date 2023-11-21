#![allow(clippy::type_complexity)]

use bevy::{
    prelude::*,
    window::WindowMode,
};
use swj::game::GamePlugin;

#[bevy_main]
pub fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins.set(
            WindowPlugin {
                primary_window: Some(Window {
                    resizable: false,
                    mode: WindowMode::BorderlessFullscreen,
                    ..default()
                }),
                ..default()
            }
        ).set(ImagePlugin::default_nearest())
    )
        .add_plugins(GamePlugin);

    #[cfg(target_os = "android")]
    app.insert_resource(Msaa::Off);

    app.run();
}

