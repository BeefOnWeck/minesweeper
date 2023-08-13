pub mod components;
pub mod resources;
mod bounds;
mod events;
mod systems;

use bevy::utils::HashMap;
use bevy::app::{App, Plugin, Startup};
use bevy::asset::{AssetServer, Handle};
use bevy::core::Name;
use bevy::ecs::{
    entity::Entity,
    query::With,
    system::{Commands, Query, Res}
};
use bevy::hierarchy::{BuildChildren, ChildBuilder};
use bevy::log;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::Update;
use bevy::render::{
    color::Color,
    prelude::SpatialBundle,
    texture::Image,
    view::Visibility
};
use bevy::sprite::{Anchor, SpriteBundle, Sprite};
use bevy::text::{Font, Text2dBundle, Text, TextSection, TextStyle, TextAlignment};
use bevy::transform::components::{Transform, GlobalTransform};
use bevy::window::{PrimaryWindow, Window};

use components::{Bomb, BombNeighbor, Coordinates, Uncover};
use resources::{
    board::Board,
    BoardOptions,
    BoardPosition,
    tile::Tile,
    tile_map::TileMap,
    TileSize
};
use bounds::Bounds2;

use crate::events::TileTriggerEvent;

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::create_board);
        app.add_systems(Update, systems::input::input_handling);
        app.add_systems(Update, systems::uncover::trigger_event_handler);
        app.add_systems(Update, systems::uncover::uncover_tiles);
        app.add_event::<TileTriggerEvent>();
        log::info!("Loaded Board Plugin");
    }
}

impl BoardPlugin {
    /// System to generate the complete board
    pub fn create_board(
        mut commands: Commands,
        board_options: Option<Res<BoardOptions>>,
        window_query: Query<&Window, With<PrimaryWindow>>,
        asset_server: Res<AssetServer>
    ) {

        let font = asset_server.load("fonts/pixeled.ttf");
        let bomb_image = asset_server.load("sprites/bomb.png");

        let options = match board_options {
            None => BoardOptions::default(),
            Some(o) => o.clone()
        };
        let mut tile_map = TileMap::empty(
            options.map_size.0, 
            options.map_size.1
        );
        tile_map.set_bombs(options.bomb_count);
        #[cfg(feature = "debug")]
        log::info!("{}", tile_map.console_output());

        // We define the size of our tiles in world space
        let window = window_query.get_single().unwrap();
        let tile_size = match options.tile_size {
            TileSize::Fixed(v) => v,
            TileSize::Adaptive { min, max } => Self::adaptive_tile_size(
                window.clone(),
                (min, max),
                (tile_map.width(), tile_map.height()),
            ),
        };
        // We deduce the size of the complete board
        let board_size = Vec2::new(
            tile_map.width() as f32 * tile_size,
            tile_map.height() as f32 * tile_size,
        );
        log::info!("board size: {}", board_size);
        // We define the board anchor position (bottom left)
        let board_position = match options.position {
            BoardPosition::Centered { offset } => {
                Vec3::new(-(board_size.x / 2.), -(board_size.y / 2.), 0.) + offset
            }
            BoardPosition::Custom(p) => p,
        };

        let mut covered_tiles = HashMap::with_capacity(
            ( tile_map.width() * tile_map.height() ).into()
        );

        let mut safe_start = None;

        commands.spawn(
            SpatialBundle {
                visibility: Visibility::Visible,
                transform: Transform::from_translation(board_position.into()),
                ..Default::default()
            }
        ).insert(Name::new("Board"))
        .insert(Transform::from_translation(board_position))
        .insert(GlobalTransform::default())
        .with_children(|parent| {
            // We spawn the board background sprite at the center of the board, since the sprite pivot is centered
            parent.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::WHITE,
                        custom_size: Some(board_size),
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(board_size.x / 2., board_size.y / 2., 0.),
                    ..Default::default()
            }).insert(Name::new("Background"));

            Self::spawn_tiles(
                parent,
                &tile_map,
                tile_size,
                options.tile_padding,
                Color::DARK_GRAY,
                &mut covered_tiles,
                Color::GRAY,
                bomb_image,
                font,
                &mut safe_start
            );
        });
        
        if options.safe_start {
            if let Some(entity) = safe_start {
                commands.entity(entity).insert(Uncover {});
            }
        }
        
        // We add the main resource of the game, the board
        commands.insert_resource(Board {
            tile_map,
            bounds: Bounds2 {
                position: Vec2::new(board_position.x, board_position.y),
                size: board_size,
            },
            tile_size,
            covered_tiles
        })

    }

    /// Computes a tile size that matches the window according to the tile map size
    fn adaptive_tile_size (
        window: Window,
        (min, max): (f32, f32),
        (width, height): (u16, u16)
    ) -> f32 {
        let max_width = window.width() / width as f32;
        let max_height = window.height() / height as f32;
        max_width.min(max_height).clamp(min, max)
    }

    fn spawn_tiles(
        parent: &mut ChildBuilder,
        tile_map: &TileMap,
        size: f32,
        padding: f32,
        covered_tile_color: Color,
        covered_tiles: &mut HashMap<Coordinates, Entity>,
        color: Color,
        bomb_image: Handle<Image>,
        font: Handle<Font>,
        safe_start_entity: &mut Option<Entity>
    ) {
        // Tiles
        for (y, line) in tile_map.iter().enumerate() {
            for (x, tile) in line.iter().enumerate() {
                let coordinates = Coordinates {
                    x: x as u16,
                    y: y as u16,
                };
                let mut cmd = parent.spawn(
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::GRAY,
                            custom_size: Some(Vec2::splat(
                                size - padding,
                            )),
                            ..Default::default()
                        },
                        transform: Transform::from_xyz(
                            (x as f32 * size) + (size / 2.),
                            (y as f32 * size) + (size / 2.),
                            1.,
                        ),
                        ..Default::default()
                    }
                );
                cmd.insert(Name::new(format!("Tile ({}, {})", x, y)));
                cmd.insert(coordinates);
                cmd.with_children(|parent| {
                    let entity = parent
                        .spawn(
                            SpriteBundle {
                                sprite: Sprite {
                                    custom_size: Some(Vec2::splat(size - padding)),
                                    color: covered_tile_color,
                                    ..Default::default()
                                },
                                transform: Transform::from_xyz(0.0, 0.0, 2.0),
                                ..Default::default()
                            })
                        .insert(Name::new("Tile Cover"))
                        .id();
                    covered_tiles.insert(coordinates, entity);
                    if safe_start_entity.is_none() && *tile == Tile::Empty {
                        *safe_start_entity = Some(entity);
                    }
                });
                match tile {
                    // If the tile is a bomb we add the matching component and a sprite child
                    Tile::Bomb => {
                        cmd
                            .insert(Bomb {})
                            .with_children(|parent| {
                                parent.spawn(SpriteBundle {
                                    sprite: Sprite {
                                        custom_size: Some(Vec2::splat(size - padding)),
                                        ..Default::default()
                                    },
                                    transform: Transform::from_xyz(0., 0., 1.),
                                    texture: bomb_image.clone(),
                                    ..Default::default()
                                });
                            });
                    }
                    // If the tile is a bomb neighbour we add the matching component and a text child
                    Tile::BombNeighbor(v) => {
                        cmd
                            .insert(BombNeighbor { count: *v })
                            .with_children(|parent| {
                                parent.spawn(Self::bomb_count_text_bundle(
                                    *v,
                                    font.clone(),
                                    size - padding,
                                ));
                            });
                    }
                    Tile::Empty => (),
                }
            }
        }
    }

    /// Generates the bomb counter text 2D Bundle for a given value
    fn bomb_count_text_bundle(count: u8, font: Handle<Font>, size: f32) -> Text2dBundle {
        // We retrieve the text and the correct color
        let (text, color) = (
            count.to_string(),
            match count {
                1 => Color::WHITE,
                2 => Color::GREEN,
                3 => Color::YELLOW,
                4 => Color::ORANGE,
                _ => Color::PURPLE,
            },
        );
        // We generate a text bundle
        Text2dBundle {
            text: Text {
                sections: vec![TextSection {
                    value: text,
                    style: TextStyle {
                        color,
                        font,
                        font_size: size,
                    },
                }],
                alignment: TextAlignment::Center,
                ..Default::default()
            },
            text_anchor: Anchor::Custom(Vec2::new(0.0, 0.1)),
            transform: Transform::from_xyz(0., 0., 1.),
            ..Default::default()
        }
    }
}