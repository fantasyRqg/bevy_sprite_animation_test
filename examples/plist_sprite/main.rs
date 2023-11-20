#![allow(clippy::type_complexity)]

use bevy::{
    prelude::*,
};
use bevy::diagnostic::{Diagnostic, DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use swj::game::GamePlugin;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(GamePlugin)
        .run();
}


fn fps_debug(diagnostic: Res<DiagnosticsStore>) {
    if let Some(fps) = diagnostic.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(value) = fps.smoothed() {
            println!("FPS: {}", value);
        }
    }
}

