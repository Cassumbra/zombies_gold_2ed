

use std::any::TypeId;

use bevy::input::{ButtonState, keyboard::KeyboardInput};
use bevy::prelude::*;
use bevy_xpbd_3d::components::LinearVelocity;
use bevy_xpbd_3d::math::{Scalar, Vector2};
use leafwing_input_manager::action_state::ActionState;
use leafwing_input_manager::input_mocking::QueryInput;

use crate::movement::{Grounded, JumpImpulse, MovementAcceleration, MovementAction, MovementType};
use crate::Action;

//use crate::rendering::window::WindowChangeEvent;

// Components
#[derive(Component, Default, Copy, Clone, Reflect)]
#[reflect(Component)]
pub struct Player;

// Systems
/// Player input.
pub fn player_input_game (
    //query: Query<(Entity, &ActionState<Action>, &MovementAcceleration, &JumpImpulse, &mut LinearVelocity, Has<Grounded>,), (With<Player>)>,
    query: Query<(Entity, &ActionState<Action>, &GlobalTransform), (With<Player>)>,
    mut movement_event_writer: EventWriter<MovementAction>,
) {
    // TODO: Perhaps we should send events for movement instead of moving directly?
    if let Ok((player, action_state, transform)) = query.get_single() {
        //println!("{:?}", transform.translation());
        // Modified from bevy_xpbd's examples.
        let forward = action_state.pressed(&Action::MoveForward);
        let backward = action_state.pressed(&Action::MoveBackward);
        let left = action_state.pressed(&Action::MoveLeft);
        let right = action_state.pressed(&Action::MoveRight);

        let vertical = forward as i8 - backward as i8;
        let horizontal = right as i8 - left as i8;
        let direction = Vector2::new(horizontal as Scalar, vertical as Scalar).clamp_length_max(1.0);

        if direction != Vector2::ZERO {
            movement_event_writer.send(MovementAction::new(player, MovementType::Move(direction) ));
        }

        if action_state.pressed(&Action::Jump) {
            movement_event_writer.send(MovementAction::new(player, MovementType::Jump));
        }
    }

    
}

/*
/// Sends [`MovementAction`] events based on keyboard input.
fn keyboard_input(
    mut movement_event_writer: EventWriter<MovementAction>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let up = keyboard_input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]);
    let down = keyboard_input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]);
    let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);

    let horizontal = right as i8 - left as i8;
    let vertical = up as i8 - down as i8;
    let direction = Vector2::new(horizontal as Scalar, vertical as Scalar).clamp_length_max(1.0);

    if direction != Vector2::ZERO {
        movement_event_writer.send(MovementAction::Move(direction));
    }

    if keyboard_input.just_pressed(KeyCode::Space) {
        movement_event_writer.send(MovementAction::Jump);
    }
}

/// Sends [`MovementAction`] events based on gamepad input.
fn gamepad_input(
    mut movement_event_writer: EventWriter<MovementAction>,
    gamepads: Res<Gamepads>,
    axes: Res<Axis<GamepadAxis>>,
    buttons: Res<ButtonInput<GamepadButton>>,
) {
    for gamepad in gamepads.iter() {
        let axis_lx = GamepadAxis {
            gamepad,
            axis_type: GamepadAxisType::LeftStickX,
        };
        let axis_ly = GamepadAxis {
            gamepad,
            axis_type: GamepadAxisType::LeftStickY,
        };

        if let (Some(x), Some(y)) = (axes.get(axis_lx), axes.get(axis_ly)) {
            movement_event_writer.send(MovementAction::Move(
                Vector2::new(x as Scalar, y as Scalar).clamp_length_max(1.0),
            ));
        }

        let jump_button = GamepadButton {
            gamepad,
            button_type: GamepadButtonType::South,
        };

        if buttons.just_pressed(jump_button) {
            movement_event_writer.send(MovementAction::Jump);
        }
    }
}
 */