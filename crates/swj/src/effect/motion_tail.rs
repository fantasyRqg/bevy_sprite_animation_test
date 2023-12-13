use bevy::prelude::*;


struct MotionTailPlugin;

impl Plugin for MotionTailPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Component)]
pub struct MotionTail {
    pub segment_len: usize,
    pub width: f32,
    pub texture: Handle<Image>,
}

#[derive(Component)]
struct MotionTailInner {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}