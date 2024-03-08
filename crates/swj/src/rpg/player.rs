use bevy::prelude::*;
use crate::game::GameStates::Loading;


pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(Loading), setup_player)

        ;
    }
}

struct Player;

fn setup_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {

}
