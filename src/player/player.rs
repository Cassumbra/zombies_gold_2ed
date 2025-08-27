

use std::any::TypeId;
use std::f32::consts::PI;

use bevy::input::{ButtonState, keyboard::KeyboardInput};
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy::{math, prelude::*};
use leafwing_input_manager::action_state::ActionState;
use leafwing_input_manager::input_mocking::QueryInput;

use crate::hotbar::Hotbar;
use crate::movement::{MovementAction, MovementType};
use crate::point::Point3d;
use crate::{Action, BuildingEvent, MiningEvent, PLAYER_HEIGHT};

//use crate::rendering::window::WindowChangeEvent;

const SENSITIVITY: f32 = -0.0004;

// Components
#[derive(Component, Default, Copy, Clone, Reflect)]
#[reflect(Component)]
pub struct Player;

// Systems
/// Player input.
pub fn player_input_game (
    //query: Query<(Entity, &ActionState<Action>, &MovementAcceleration, &JumpImpulse, &mut LinearVelocity, Has<Grounded>,), (With<Player>)>,
    mut query: Query<(Entity, &ActionState<Action>, &mut Transform, &Children, Option<&mut Hotbar>), (With<Player>)>,
    mut cam_query: Query<(&mut Transform), (Without<Player>)>,
    
    mut evw_movement: EventWriter<MovementAction>,
    mut evw_mining: EventWriter<MiningEvent>,
    mut evw_building: EventWriter<BuildingEvent>,

    mut primary_window: Query<&mut Window, With<PrimaryWindow>>
) {
    // TODO: Perhaps we should send events for movement instead of moving directly?
    if let Ok((player, action_state, mut transform, children, opt_hotbar)) = query.get_single_mut() {
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

        if direction != Vec3::ZERO {
            evw_movement.send(MovementAction::new(player, MovementType::Move(Vec2::new(direction.x, direction.z)) ));
        }

        if action_state.pressed(&Action::Jump) {
            evw_movement.send(MovementAction::new(player, MovementType::Jump));
        }

        
        if let Ok(mut window) = primary_window.get_single_mut() {
            if action_state.just_pressed(&Action::MenuBack) {
                window.cursor.grab_mode = CursorGrabMode::None;
                window.cursor.visible = true;
            }

            if action_state.just_pressed(&Action::Primary) {
                window.cursor.grab_mode = CursorGrabMode::Confined;
                window.cursor.visible = false;
                evw_mining.send(MiningEvent { entity: player, is_start: true });
            }

            if action_state.just_released(&Action::Primary) {
                evw_mining.send(MiningEvent { entity: player, is_start: false });
            }


            if action_state.just_pressed(&Action::Secondary) {
                evw_building.send(BuildingEvent { entity: player, is_start: true });
            }

            if action_state.just_released(&Action::Secondary) {
                evw_building.send(BuildingEvent { entity: player, is_start: false });
            }


            if window.cursor.grab_mode == CursorGrabMode::Confined {
                if let Some(look) = action_state.axis_pair(&Action::Look) {
                    transform.rotate_y(look.x() * SENSITIVITY);
                    // TODO: Maybe we should have some camera component or something for this? Or some better way to link things?
                    for child in children.iter() {
                        if let Ok(mut child_transform) = cam_query.get_mut(*child) {
                            let mut rotation_x = child_transform.rotation.to_euler(EulerRot::XYZ).0 + look.y() * SENSITIVITY;
                            // Rotating the character is OK, since we don't base any collision info based on rotations.
                            rotation_x = rotation_x.clamp(-PI/2.0, PI/2.0);
                            child_transform.rotation = Quat::from_axis_angle(Vec3::X, rotation_x);
                            
                            //child_transform.rotate_x(look.y() * -0.001);
                            //child_transform.rotation.x = child_transform.rotation.x.clamp(-0.9, 0.9);
                        }
                    }
                }
            }
            
        }
        
        if let Some(mut hotbar) = opt_hotbar {
            if action_state.just_pressed(&Action::Slot1) && hotbar.slots.len() > 0 {
                hotbar.position = 0;
            }
            if action_state.just_pressed(&Action::Slot2) && hotbar.slots.len() > 1 {
                hotbar.position = 1;
            }
            if action_state.just_pressed(&Action::Slot3) && hotbar.slots.len() > 2 {
                hotbar.position = 2;
            }
            if action_state.just_pressed(&Action::Slot4) && hotbar.slots.len() > 3 {
                hotbar.position = 3;
            }
            if action_state.just_pressed(&Action::Slot5) && hotbar.slots.len() > 4 {
                hotbar.position = 4;
            }
            if action_state.just_pressed(&Action::Slot6) && hotbar.slots.len() > 5 {
                hotbar.position = 5;
            }
            if action_state.just_pressed(&Action::Slot7) && hotbar.slots.len() > 6 {
                hotbar.position = 6;
            }
            if action_state.just_pressed(&Action::Slot8) && hotbar.slots.len() > 7 {
                hotbar.position = 7;
            }
            if action_state.just_pressed(&Action::Slot9) && hotbar.slots.len() > 8 {
                hotbar.position = 8;
            }
            if action_state.just_pressed(&Action::Slot0) && hotbar.slots.len() > 9 {
                hotbar.position = 9;
            }
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