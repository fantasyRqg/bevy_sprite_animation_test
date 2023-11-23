use bevy::app::{AppExit, PluginsState};
use bevy::ecs::event::ManualEventReader;
use bevy::ecs::system::{SystemParam, SystemState};
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::{MouseButtonInput, MouseMotion, MouseWheel};
use bevy::input::touchpad::{TouchpadMagnify, TouchpadRotate};
use bevy::math::DVec2;
use bevy::prelude::*;
use bevy::tasks::tick_global_task_pools_on_main_thread;
use bevy::window::{ApplicationLifetime, CursorEntered, CursorLeft, CursorMoved, FileDragAndDrop, Ime, RawHandleWrapper, ReceivedCharacter, Window, WindowBackendScaleFactorChanged, WindowCloseRequested, WindowCreated, WindowDestroyed, WindowFocused, WindowMoved, WindowResized, WindowScaleFactorChanged, WindowThemeChanged};
use bevy::winit::converters;
use bevy::winit::converters::{convert_enabled_buttons, convert_window_level, convert_window_theme};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
};
use winit::dpi::LogicalSize;
use winit::event::StartCause;
use winit::event_loop::EventLoopWindowTarget;

pub struct MyWinitPlugin {}

#[derive(Debug)]
struct WinitWindows {
    entity: Entity,
    app_should_run: bool,
    started: bool,
    window: Option<winit::window::Window>,
}

impl Default for WinitWindows {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
            app_should_run: false,
            started: false,
            window: None,
        }
    }
}

impl Plugin for MyWinitPlugin {
    fn build(&self, app: &mut App) {
        app.init_non_send_resource::<WinitWindows>()
            .set_runner(my_runner)
        ;

        let event_loop = EventLoop::new();

        app.insert_non_send_resource(event_loop);
    }
}


#[derive(SystemParam)]
struct WindowAndInputEventWriters<'w> {
    // `winit` `WindowEvent`s
    window_resized: EventWriter<'w, WindowResized>,
    window_close_requested: EventWriter<'w, WindowCloseRequested>,
    window_scale_factor_changed: EventWriter<'w, WindowScaleFactorChanged>,
    window_backend_scale_factor_changed: EventWriter<'w, WindowBackendScaleFactorChanged>,
    window_focused: EventWriter<'w, WindowFocused>,
    window_moved: EventWriter<'w, WindowMoved>,
    window_theme_changed: EventWriter<'w, WindowThemeChanged>,
    window_destroyed: EventWriter<'w, WindowDestroyed>,
    lifetime: EventWriter<'w, ApplicationLifetime>,
    keyboard_input: EventWriter<'w, KeyboardInput>,
    character_input: EventWriter<'w, ReceivedCharacter>,
    mouse_button_input: EventWriter<'w, MouseButtonInput>,
    touchpad_magnify_input: EventWriter<'w, TouchpadMagnify>,
    touchpad_rotate_input: EventWriter<'w, TouchpadRotate>,
    mouse_wheel_input: EventWriter<'w, MouseWheel>,
    touch_input: EventWriter<'w, TouchInput>,
    ime_input: EventWriter<'w, Ime>,
    file_drag_and_drop: EventWriter<'w, FileDragAndDrop>,
    cursor_moved: EventWriter<'w, CursorMoved>,
    cursor_entered: EventWriter<'w, CursorEntered>,
    cursor_left: EventWriter<'w, CursorLeft>,
    // `winit` `DeviceEvent`s
    mouse_motion: EventWriter<'w, MouseMotion>,
}

fn my_runner(mut app: App) {
    if app.plugins_state() == PluginsState::Ready {
        // If we're already ready, we finish up now and advance one frame.
        // This prevents black frames during the launch transition on iOS.
        app.finish();
        app.cleanup();
        app.update();
    }

    let mut event_loop = app
        .world
        .remove_non_send_resource::<EventLoop<()>>()
        .unwrap();


    let mut create_window_system_state: SystemState<(
        Commands,
        Query<(Entity, &mut Window), Added<Window>>,
        EventWriter<WindowCreated>,
        NonSendMut<WinitWindows>,
    )> = SystemState::from_world(&mut app.world);


    let mut event_writer_system_state: SystemState<(
        WindowAndInputEventWriters,
        Query<(&mut Window)>,
        NonSendMut<WinitWindows>,
    )> = SystemState::new(&mut app.world);


    let mut app_exit_event_reader = ManualEventReader::<AppExit>::default();

    event_loop.run_return(|event, event_loop, control_flow| {
        if app.plugins_state() != PluginsState::Cleaned {
            if app.plugins_state() != PluginsState::Ready {
                tick_global_task_pools_on_main_thread();
            } else {
                app.finish();
                app.cleanup();
            }
        }


        match event {
            Event::NewEvents(start_cause) => match start_cause {
                StartCause::Init => {
                    let (commands,
                        mut win_query,
                        win_evt_writer,
                        winit_windows,
                    ) = create_window_system_state.get_mut(&mut app.world);

                    create_windows(
                        &event_loop,
                        commands,
                        win_query.iter_mut(),
                        win_evt_writer,
                        winit_windows,
                    );

                    create_window_system_state.apply(&mut app.world);
                }
                _ => {}
            }
            Event::WindowEvent { event, window_id, .. } => {
                let (mut event_writers,
                    mut windows,
                    mut winit_windows,
                ) = event_writer_system_state.get_mut(&mut app.world);


                let window_entity = winit_windows.entity;
                if window_entity == Entity::PLACEHOLDER {
                    return;
                }

                let mut window = windows.get_mut(window_entity).unwrap();
                match event {
                    WindowEvent::Focused(focused) => {
                        window.focused = focused;
                        winit_windows.app_should_run = focused;
                        event_writers.window_focused.send(WindowFocused {
                            window: window_entity,
                            focused,
                        });
                    }
                    WindowEvent::Destroyed => {
                        event_writers.window_destroyed.send(WindowDestroyed {
                            window: window_entity,
                        });
                    }
                    WindowEvent::CloseRequested => {
                        event_writers.window_close_requested.send(WindowCloseRequested {
                            window: window_entity,
                        });
                    }
                    WindowEvent::KeyboardInput { ref input, .. } => {
                        event_writers
                            .keyboard_input
                            .send(converters::convert_keyboard_input(input, window_entity));
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        let physical_position = DVec2::new(position.x, position.y);
                        window.set_physical_cursor_position(Some(physical_position));
                        event_writers.cursor_moved.send(CursorMoved {
                            window: window_entity,
                            position: (physical_position / window.resolution.scale_factor())
                                .as_vec2(),
                        });
                    }
                    WindowEvent::CursorEntered { .. } => {
                        event_writers.cursor_entered.send(CursorEntered {
                            window: window_entity,
                        });
                    }
                    WindowEvent::CursorLeft { .. } => {
                        window.set_physical_cursor_position(None);
                        event_writers.cursor_left.send(CursorLeft {
                            window: window_entity,
                        });
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        event_writers.mouse_button_input.send(MouseButtonInput {
                            button: converters::convert_mouse_button(button),
                            state: converters::convert_element_state(state),
                            window: window_entity,
                        });
                    }
                    _ => ()
                }
            }
            Event::Resumed => {
                let (mut event_writers,
                    _,
                    winit_windows
                ) = event_writer_system_state.get_mut(&mut app.world);
                match winit_windows.started {
                    false => {
                        event_writers.lifetime.send(ApplicationLifetime::Started);
                    }
                    _ => {
                        event_writers.lifetime.send(ApplicationLifetime::Resumed);
                    }
                }
            }
            Event::MainEventsCleared => {
                // control_flow.set_exit();
                let winit_windows = app.world.get_non_send_resource::<WinitWindows>().unwrap();
                if app.plugins_state() == PluginsState::Cleaned && winit_windows.app_should_run {
                    app.update();
                }

                if let Some(app_exit_events) = app.world.get_resource::<Events<AppExit>>() {
                    if app_exit_event_reader.read(app_exit_events).last().is_some() {
                        control_flow.set_exit();
                    }
                }
            }
            _ => (),
        }
    });
}


fn create_windows<'a>(
    event_loop: &EventLoopWindowTarget<()>,
    mut commands: Commands,
    mut created_windows: impl Iterator<Item=(Entity, Mut<'a, Window>)>,
    mut event_writer: EventWriter<WindowCreated>,
    mut winit_windows: NonSendMut<WinitWindows>,
) {
    let window_builder = WindowBuilder::new();
    let (win_entity, mut window) = created_windows.next().unwrap();

    let logical_size = LogicalSize::new(window.width(), window.height());

    let mut window_builder = if let Some(sf) = window.resolution.scale_factor_override() {
        window_builder.with_inner_size(logical_size.to_physical::<f64>(sf))
    } else {
        window_builder.with_inner_size(logical_size)
    };


    window_builder = window_builder
        .with_window_level(convert_window_level(window.window_level))
        .with_theme(window.window_theme.map(convert_window_theme))
        .with_resizable(window.resizable)
        .with_enabled_buttons(convert_enabled_buttons(window.enabled_buttons))
        .with_decorations(window.decorations)
        .with_transparent(window.transparent)
        .with_visible(window.visible);

    let constraints = window.resize_constraints.check_constraints();
    let min_inner_size = LogicalSize {
        width: constraints.min_width,
        height: constraints.min_height,
    };
    let max_inner_size = LogicalSize {
        width: constraints.max_width,
        height: constraints.max_height,
    };

    let window_builder =
        if constraints.max_width.is_finite() && constraints.max_height.is_finite() {
            window_builder
                .with_min_inner_size(min_inner_size)
                .with_max_inner_size(max_inner_size)
        } else {
            window_builder.with_min_inner_size(min_inner_size)
        };

    let window_builder = window_builder.with_title(window.title.as_str());

    let winit_window = window_builder
        .build(&event_loop)
        .unwrap();

    window.resolution
        .set_scale_factor(winit_window.scale_factor());

    commands
        .entity(win_entity)
        .insert(RawHandleWrapper {
            window_handle: winit_window.raw_window_handle(),
            display_handle: winit_window.raw_display_handle(),
        });

    winit_windows.entity = win_entity;
    winit_windows.window = Some(winit_window);

    event_writer.send(WindowCreated { window: win_entity });
}