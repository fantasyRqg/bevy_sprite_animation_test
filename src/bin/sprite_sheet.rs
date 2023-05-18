//! Renders an animated sprite by loading all animation frames from a single image (a sprite sheet)
//! into a texture atlas, and changing the displayed image periodically.

use rand::Rng;
use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    asset::LoadState,
};


fn main() {
    App::new()
        .init_resource::<RpgSpriteHandles>()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .add_state::<AppState>()
        .add_system(load_textures.in_schedule(OnEnter(AppState::Setup)))
        .add_system(check_textures.in_set(OnUpdate(AppState::Setup)))
        .add_system(setup.in_schedule(OnEnter(AppState::Finished)))
        .add_system(animate_sprite)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_startup_system(ui_setup)
        .add_system(text_update_system)
        .add_system(text_color_system)
        .run();
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
enum AppState {
    #[default]
    Setup,
    Finished,
}

impl States for AppState {
    type Iter = std::array::IntoIter<AppState, 2>;

    fn variants() -> Self::Iter {
        [AppState::Setup, AppState::Finished].into_iter()
    }
}

#[derive(Resource, Default)]
struct RpgSpriteHandles {
    handles: Vec<HandleUntyped>,
}

fn load_textures(mut rpg_sprite_handles: ResMut<RpgSpriteHandles>, asset_server: Res<AssetServer>) {
    rpg_sprite_handles.handles = asset_server.load_folder("textures/archer").unwrap();
}

fn check_textures(
    mut next_state: ResMut<NextState<AppState>>,
    rpg_sprite_handles: ResMut<RpgSpriteHandles>,
    asset_server: Res<AssetServer>,
) {
    if let LoadState::Loaded = asset_server
        .get_group_load_state(rpg_sprite_handles.handles.iter().map(|handle| handle.id()))
    {
        next_state.set(AppState::Finished);
    }
}


fn build_archer_tex(asset_server: &Res<AssetServer>, textures: &mut ResMut<Assets<Image>>,
                    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
                    rpg_sprite_handles: &mut ResMut<RpgSpriteHandles>,
) -> Handle<TextureAtlas> {
    let mut texture_atlas_builder = TextureAtlasBuilder::default();
    for handle in &rpg_sprite_handles.handles {
        let handle = handle.typed_weak();
        let Some(texture) = textures.get(&handle) else {
            warn!("{:?} did not resolve to an `Image` asset.", asset_server.get_handle_path(handle));
            continue;
        };

        texture_atlas_builder.add_texture(handle, texture);
    }

    let archer_texture_atlas = texture_atlas_builder.finish(textures).unwrap();


    return texture_atlases.add(archer_texture_atlas);
}


fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = if sprite.index == indices.last {
                indices.first
            } else {
                sprite.index + 1
            };
        }
    }
}

fn gen_animated_sprites(texture_atlas_handle: &Handle<TextureAtlas>, texture_atlas_handle2: &Handle<TextureAtlas>, tah3: &Handle<TextureAtlas>) -> Vec<(SpriteSheetBundle, AnimationIndices, AnimationTimer)> {
    let mut rng = rand::thread_rng();

    let mut sprites = vec![];
    let num: i32 = 100;
    for i in 0..num {
        for j in 0..num {
            let animation_indices = AnimationIndices { first: 1, last: 6 };


            sprites.push((
                SpriteSheetBundle {
                    texture_atlas: match rng.gen_range(0..3) {
                        0 => texture_atlas_handle.clone(),
                        1 => texture_atlas_handle2.clone(),
                        _ => tah3.clone(),
                    },

                    sprite: TextureAtlasSprite {
                        index: j as usize % 6 + 1,
                        ..default()
                    },
                    transform: Transform {
                        translation: Vec3::new((i - num / 2) as f32 * 12.5, (j - num / 2) as f32 * 6.5, rng.gen_range(0.0..2.0)),
                        scale: Vec3::splat(rng.gen_range(0.5..1.5)),
                        ..default()
                    },
                    ..default()
                },
                animation_indices,
                AnimationTimer(Timer::from_seconds(rng.gen_range(0.1..0.3), TimerMode::Repeating)),
            ))
        }
    }

    return sprites;
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut textures: ResMut<Assets<Image>>,
    mut rpg_sprite_handles: ResMut<RpgSpriteHandles>,
) {
    let texture_atlas_handle = tex_handler_from_image(&asset_server, &mut texture_atlases, "textures/gabe/gabe-idle-run.png");
    let texture_atlas_handle2 = tex_handler_from_image(&asset_server, &mut texture_atlases, "textures/mani/mani-idle-run.png");
    let tah3 = build_archer_tex(&asset_server, &mut textures, &mut texture_atlases, &mut rpg_sprite_handles);

    // Use only the subset of sprites in the sheet that make up the run animation
    commands.spawn(Camera2dBundle::default());


    for _ in 0..10 {
        let sprites = gen_animated_sprites(&texture_atlas_handle, &texture_atlas_handle2, &tah3);
        commands.spawn_batch(sprites);
    }


    // let sprites = gen_animated_sprites(&texture_atlas_handle, &texture_atlas_handle2);
    // commands.spawn_batch(sprites);
    //
    // let sprites = gen_animated_sprites(&texture_atlas_handle, &texture_atlas_handle2);
    // commands.spawn_batch(sprites);
}

fn tex_handler_from_image(asset_server: &Res<AssetServer>, texture_atlases: &mut ResMut<Assets<TextureAtlas>>, tex_path: &str) -> Handle<TextureAtlas> {
    let texture_handle = asset_server.load(tex_path);
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 7, 1, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    texture_atlas_handle
}


fn ui_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // UI camera
    // Text with one section
    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            "hello\nbevy!",
            TextStyle {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 100.0,
                color: Color::WHITE,
            },
        ) // Set the alignment of the Text
            .with_text_alignment(TextAlignment::Center)
            // Set the style of the TextBundle itself.
            .with_style(Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    bottom: Val::Px(5.0),
                    right: Val::Px(15.0),
                    ..default()
                },
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
}

fn text_color_system(time: Res<Time>, mut query: Query<&mut Text, With<ColorText>>) {
    for mut text in &mut query {
        let seconds = time.elapsed_seconds();

        // Update the color of the first and only section.
        text.sections[0].style.color = Color::Rgba {
            red: (1.25 * seconds).sin() / 2.0 + 0.5,
            green: (0.75 * seconds).sin() / 2.0 + 0.5,
            blue: (0.50 * seconds).sin() / 2.0 + 0.5,
            alpha: 1.0,
        };
    }
}

fn text_update_system(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                text.sections[1].value = format!("{value:.2}");
            }
        }
    }
}