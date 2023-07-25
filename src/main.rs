use bevy::prelude::*;
use bevy::window::WindowResolution;

use board_plugin::BoardPlugin;
use board_plugin::components::Coordinates;
use board_plugin::resources::BoardOptions;

#[cfg(feature = "debug")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    let mut app = App::new();
    // Window setup
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resolution: WindowResolution::new(700.0, 800.0),
            title: "Mine Sweeper!".to_string(),
            ..default()
        }),
        ..default()
    }));
    #[cfg(feature = "debug")]
    // Debug hierarchy inspector
    app.add_plugins(WorldInspectorPlugin::new());
    app.insert_resource(BoardOptions {
        map_size: (20, 20),
        bomb_count: 40,
        tile_padding: 3.0,
        ..Default::default()
    });
    app.add_plugins(BoardPlugin);
    app.register_type::<Coordinates>();
    // Startup system (cameras)
    app.add_systems(Startup, camera_setup);
    // Run the app
    app.run();
}

fn camera_setup(mut commands: Commands) {
    // 2D orthographic camera
    commands.spawn(Camera2dBundle::default());
}