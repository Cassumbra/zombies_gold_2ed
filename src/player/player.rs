

use std::any::TypeId;
use std::f32::consts::PI;

use bevy::input::{ButtonState, keyboard::KeyboardInput};
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy::{math, prelude::*};
use bevy_xpbd_3d::components::LinearVelocity;
use bevy_xpbd_3d::math::{Scalar, Vector2};
use leafwing_input_manager::action_state::ActionState;
use leafwing_input_manager::input_mocking::QueryInput;

use crate::movement::{Grounded, JumpImpulse, MovementAcceleration, MovementAction, MovementType};
use crate::point::Point3d;
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
    mut query: Query<(Entity, &ActionState<Action>, &mut Transform, &Children), (With<Player>)>,
    mut cam_query: Query<(&mut Transform), (Without<Player>)>,
    mut movement_event_writer: EventWriter<MovementAction>,

    mut primary_window: Query<&mut Window, With<PrimaryWindow>>
) {
    // TODO: Perhaps we should send events for movement instead of moving directly?
    if let Ok((player, action_state, mut transform, children)) = query.get_single_mut() {
        //println!("{:?}", transform.translation());
        // Modified from bevy_xpbd's examples + bevy_flycam
        let forward = Vec3::from(transform.forward());
        let right = Vec3::from(transform.right());

        let mut direction = Vec3::ZERO;

        if action_state.pressed(&Action::MoveForward) {
            direction += forward;
        }
        if action_state.pressed(&Action::MoveBackward) {
            direction -= forward;
        }
        if action_state.pressed(&Action::MoveRight) {
            direction += right;
        }
        if action_state.pressed(&Action::MoveLeft) {
            direction -= right;
        }

        
        if let Ok(mut window) = primary_window.get_single_mut() {
            if action_state.just_pressed(&Action::MenuBack) {
                window.cursor.grab_mode = CursorGrabMode::None;
                window.cursor.visible = true;
            }

            if action_state.just_pressed(&Action::Primary) {
                window.cursor.grab_mode = CursorGrabMode::Confined;
                window.cursor.visible = false;
            }

            if window.cursor.grab_mode == CursorGrabMode::Confined {
                if let Some(look) = action_state.axis_pair(&Action::Look) {
                    // TODO: The sensitivity shouldn't be a magic number.
                    transform.rotate_y(look.x() *  -0.001);
                    // TODO: Maybe we should have some camera component or something for this? Or some better way to link things?
                    for child in children.iter() {
                        if let Ok(mut child_transform) = cam_query.get_mut(*child) {
                            let mut rotation_x = child_transform.rotation.to_euler(EulerRot::XYZ).0 + look.y() * -0.001;
                            rotation_x = rotation_x.clamp(-PI/2.0, PI/2.0);
                            child_transform.rotation = Quat::from_axis_angle(Vec3::X, rotation_x);
                            
                            //child_transform.rotate_x(look.y() * -0.001);
                            //child_transform.rotation.x = child_transform.rotation.x.clamp(-0.9, 0.9);
                        }
                    }
                }
            }
            
        }
        

        //direction = direction.normalize_or_zero();

        if direction != Vec3::ZERO {
            movement_event_writer.send(MovementAction::new(player, MovementType::Move(Vec2::new(direction.x, direction.z)) ));
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