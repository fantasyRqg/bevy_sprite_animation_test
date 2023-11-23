
use rand::Rng;
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy::diagnostic::DiagnosticsStore;
use bevy::ecs::query::BatchingStrategy;
use bevy::math::vec2;


pub struct SpriteTtPlugin {}

impl Plugin for SpriteTtPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RpgSpriteHandles>()
            .add_systems(Update, animate_sprite)
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Startup, setup)
            .add_systems(Startup, ui_setup)
            .add_systems(Update, text_update_system)
            .add_systems(Update, text_color_system)
            .add_systems(Update, touch_system)
            .add_systems(Update, button_system)
        // .add_systems(PostUpdate, culling_system)
        ;
    }
}


#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

// A unit struct to help identify the FPS UI component, since there may be many Text components
#[derive(Component)]
struct FpsText;

// A unit struct to help identify the color-changing Text component
#[derive(Component)]
struct ColorText;


#[derive(Resource, Default)]
struct RpgSpriteHandles {
    gen_handles: Vec<Handle<TextureAtlas>>,
}

fn overlapped_by_others(rect: Rect, visible_rects: &Vec<Rect>) -> bool {
    let left_top = rect.min;
    for vr in visible_rects {
        if !vr.contains(left_top) {
            return false;
        }
    }

    let left_bottom = vec2(rect.min.x, rect.max.y);
    for vr in visible_rects {
        if !vr.contains(left_bottom) {
            return false;
        }
    }

    let right_top = vec2(rect.max.x, rect.min.y);
    for vr in visible_rects {
        if !vr.contains(right_top) {
            return false;
        }
    }

    let right_bottom = rect.max;
    for vr in visible_rects {
        if !vr.contains(right_bottom) {
            return false;
        }
    }

    true
}

fn culling_system(texture_atlases: Res<Assets<TextureAtlas>>,
                  mut query: Query<(&mut Visibility, &Transform, &TextureAtlasSprite, &Handle<TextureAtlas>)>) {
    let mut visible_rects: Vec<Rect> = vec![];

    let mut entities = query.iter_mut().collect::<Vec<_>>();
    entities.sort_by(|(_, t1, _, _), (_, t2, _, _)| {
        t1.translation.z.total_cmp(&t2.translation.z).reverse()
    });

    for (mut visibility, transform, sprite, tex) in entities {
        // *visibility = Visibility::Hidden;

        let size = match sprite.custom_size {
            Some(size) => size,
            None => {
                texture_atlases.get(tex).unwrap().textures[sprite.index].size()
            }
        };

        let world_rect = Rect::new(
            transform.translation.x,
            transform.translation.y,
            transform.translation.x + size.x * transform.scale.x,
            transform.translation.y + size.y * transform.scale.y,
        );

        *visibility = Visibility::Inherited;

        if visible_rects.is_empty() {
            visible_rects.push(world_rect);
            continue;
        }

        if overlapped_by_others(world_rect, &visible_rects) {
            *visibility = Visibility::Hidden;
        } else {
            visible_rects.push(world_rect);
        }
    }

    // if !visible_rects.is_empty() {
    //     // print first 5 items
    //     for i in 0..5 {
    //         let rect = visible_rects[i];
    //         info!("visible_rects[{}]: {:?}", i, rect);
    //     }
    // }
}

fn touch_system(mut commands: Commands, touches: Res<Touches>, rpg_sprite_handles: Res<RpgSpriteHandles>) {
    for touch in touches.iter_just_pressed() {
        info!(
            "just pressed touch with id: {:?}, at: {:?}",
            touch.id(),
            touch.position()
        );

        let sprites = gen_animated_sprites(&rpg_sprite_handles, 1000);
        commands.spawn_batch(sprites);
    }
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    query.par_iter_mut()
        .batching_strategy(BatchingStrategy::fixed(64))
        .for_each(|(indices, mut timer, mut sprite)| {
            timer.tick(time.delta());
            if timer.just_finished() {
                sprite.index = if sprite.index == indices.last {
                    indices.first
                } else {
                    sprite.index + 1
                };
            }
        });
}

fn gen_animated_sprites(rpg_sprite_handles: &Res<RpgSpriteHandles>, num: i32) -> Vec<(SpriteSheetBundle, AnimationIndices, AnimationTimer)> {
    let mut rng = rand::thread_rng();

    let mut sprites = vec![];
    for _i in 0..num {
        let tex_handle = rpg_sprite_handles.gen_handles[2].clone();
        let end_sprite_idx = 4 * 13 - 1;
        // let end_sprite_idx = 6;
        sprites.push((
            SpriteSheetBundle {
                // texture_atlas: match rng.gen_range(0..3) {
                //     0 => texture_atlas_handle.clone(),
                //     1 => texture_atlas_handle2.clone(),
                //     _ => tah3.clone(),
                // },
                texture_atlas: tex_handle,

                sprite: TextureAtlasSprite {
                    index: rng.gen_range(1..end_sprite_idx),
                    custom_size: Some(Vec2::new(176.0, 148.0) / 4.0),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(rng.gen_range(-50.0..50.0) * 12.5, rng.gen_range(-50.0..50.0) * 6.5, rng.gen_range(0.0..2.0)),
                    scale: Vec3::splat(0.5),
                    ..default()
                },
                ..default()
            },
            AnimationIndices { first: 1, last: end_sprite_idx },
            AnimationTimer(Timer::from_seconds(rng.gen_range(0.1..0.3), TimerMode::Repeating)),
            // AnimationTimer(Timer::from_seconds(1. / 30., TimerMode::Repeating)),
        ))
    }

    return sprites;
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut rpg_sprite_handles: ResMut<RpgSpriteHandles>,
) {
    let texture_atlas_handle = tex_handler_from_image(&asset_server, &mut texture_atlases, "textures/gabe/gabe-idle-run.png");
    let texture_atlas_handle2 = tex_handler_from_image(&asset_server, &mut texture_atlases, "textures/mani/mani-idle-run.png");

    let texture_handle = asset_server.load("textures/archer/archer.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(176.0, 148.0), 4, 13, None, None);
    let tah3 = texture_atlases.add(texture_atlas);


    rpg_sprite_handles.gen_handles.push(texture_atlas_handle);
    rpg_sprite_handles.gen_handles.push(texture_atlas_handle2);
    rpg_sprite_handles.gen_handles.push(tah3);
    // Use only the subset of sprites in the sheet that make up the run animation
    commands.spawn(Camera2dBundle::default());


    let _rpg_res = &Res::from(rpg_sprite_handles);
    // for _ in 0..1 {
    //     let sprites = gen_animated_sprites(rpg_res, 10000);
    //     commands.spawn_batch(sprites);
    // }


    // let sprites = gen_animated_sprites(&texture_atlas_handle, &texture_atlas_handle2);
    // commands.spawn_batch(sprites);
    //
    // let sprites = gen_animated_sprites(&texture_atlas_handle, &texture_atlas_handle2);
    // commands.spawn_batch(sprites);
}

fn tex_handler_from_image(asset_server: &Res<AssetServer>, texture_atlases: &mut ResMut<Assets<TextureAtlas>>, tex_path: &str) -> Handle<TextureAtlas> {
    let texture_handle = asset_server.load(tex_path.to_string());
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 7, 1, None, None);
    texture_atlases.add(texture_atlas)
}


fn ui_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // UI camera
    // Text with one section
    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_sections([
            TextSection::new(
                "Sprites: ",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 60.0,
                    color: Color::WHITE,
                },
            ),
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            TextSection::new(
                "0",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 100.0,
                    color: Color::WHITE,
                },
            )
        ]) // Set the alignment of the Text
            .with_text_alignment(TextAlignment::Center)
            // Set the style of the TextBundle itself.
            .with_style(Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(5.0),
                right: Val::Px(15.0),

                ..default()
            }),
        ColorText,
    ));
    // Text with multiple sections
    commands.spawn((
        // Create a TextBundle that has a Text with a list of sections.
        TextBundle::from_sections([
            TextSection::new(
                "FPS: ",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 60.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::from_style(TextStyle {
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                font_size: 60.0,
                color: Color::GOLD,
            }),
        ]),
        FpsText,
    ));

    #[cfg(target_os = "macos")]
    commands.spawn((
        ButtonBundle {
            style: Style {
                width: Val::Px(150.0),
                height: Val::Px(65.0),
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                bottom: Val::Px(10.0),
                ..Default::default()
            },
            ..Default::default()
        },
    ));
}

fn button_system(
    mut commands: Commands,
    rpg_sprite_handles: Res<RpgSpriteHandles>,
    interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<Button>),
    >,
) {
    for interaction in &interaction_query {
        match *interaction {
            Interaction::Pressed => {
                let sprites = gen_animated_sprites(&rpg_sprite_handles, 1000);
                commands.spawn_batch(sprites);
            }
            _ => {}
        }
    }
}

fn text_color_system(_time: Res<Time>, mut query: Query<&mut Text, With<ColorText>>, sprites: Query<&TextureAtlasSprite>) {
    for mut text in &mut query {


        // Update the color of the first and only section.
        text.sections[1].value = sprites.iter().count().to_string();
    }
}

fn text_update_system(diagnostics: Res<DiagnosticsStore>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                text.sections[1].value = format!("{value:.2}");
            }
        }
    }
}

