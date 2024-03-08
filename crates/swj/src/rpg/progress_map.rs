use bevy::prelude::*;
use crate::game::GameStates::PrepareScene;


pub struct ProgressMapPlugin;

impl Plugin for ProgressMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(PrepareScene), setup);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    info!("Setting up progress map...");

    commands.spawn(SpriteBundle {
        ..Default::default()
    });
}