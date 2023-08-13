use bevy::app::App;
use bevy::prelude::{DefaultPlugins, default, PluginGroup, Startup, Commands, Camera2dBundle};
use bevy::window::{Window, WindowPlugin, WindowResolution};

use board_plugin::BoardPlugin;
use board_plugin::resources::BoardOptions;

fn main() {
    let mut app = App::new();
    // Window setup
    app.add_plugins(DefaultPlugins.set(
        WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(700.0, 800.0),
                title: "Mine Sweeper!".to_string(),
                ..default()
            }),
            ..default()
        }
    ));
    app.insert_resource(
        BoardOptions {
            map_size: (20, 20),
            bomb_count: 40,
            tile_padding: 3.0,
            safe_start: true,
            ..Default::default()
        }
    );
    app.add_plugins(BoardPlugin);
    // Startup system (cameras)
    app.add_systems(Startup, camera_setup);
    // Run the app
    app.run();
}

fn camera_setup(mut commands: Commands) {
    // 2D orthographic camera
    commands.spawn(Camera2dBundle::default());
}