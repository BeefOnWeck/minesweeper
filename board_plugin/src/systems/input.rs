use bevy::input::{
    mouse::{
        MouseButton,
        MouseButtonInput
    }, 
    ButtonState
};
use bevy::log;

use bevy::ecs::{
    event::{EventReader, EventWriter},
    query::With,
    system::{Commands, Query, Res}
};
use bevy::window::{PrimaryWindow, Window};

use crate::Board;
use crate::events::TileTriggerEvent;

pub fn input_handling(
    window_query: Query<&Window, With<PrimaryWindow>>,
    board: Res<Board>,
    mut button_evr: EventReader<MouseButtonInput>,
    mut tile_trigger_ewr: EventWriter<TileTriggerEvent>
) {

    let window = window_query.get_single().unwrap();

    for event in button_evr.iter() {
        if let ButtonState::Pressed = event.state {
            let position = window.cursor_position();
            if let Some(pos) = position {
                log::info!("Mouse button pressed: {:?} at {}", event.button, pos);
                let tile_coordinates = board.mouse_position(window, pos);
                if let Some(coordinates) = tile_coordinates {
                    match event.button {
                        MouseButton::Left => {
                            log::info!("Trying to uncover tile on {}", coordinates);
                            tile_trigger_ewr.send(TileTriggerEvent{coordinates});
                        }
                        MouseButton::Right => {
                            log::info!("Trying to mark tile on {}", coordinates);
                            // TODO: generate an event
                        }
                        _ => (),
                    }
                }
            }
        }
    }
}