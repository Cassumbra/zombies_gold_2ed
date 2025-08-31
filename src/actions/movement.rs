// All this shit below is yoinked straight from bevy_xpbd's dynamic_character_3d example.
// Some modifications will be made as needed.

use bevy::{ecs::query::Has, prelude::*, transform};

use crate::{AabbCollider, LinearVelocity, Player, SurfaceContact, SurfaceContacts, PLAYER_HEIGHT, PLAYER_WIDTH};



/// An event sent for a movement input action.
#[derive(Event)]
pub struct MovementAction {
    entity: Entity,
    movement: MovementType,
}
impl MovementAction {
    pub fn new(entity: Entity, movement: MovementType) -> MovementAction {
        MovementAction { entity, movement }
    }
}

pub enum MovementType {
    Move(Vec2),
    Jump,
}

#[derive(Component, Default, Copy, Clone, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct Crouched(pub bool);

/// A marker component indicating that an entity is using a character controller.
#[derive(Component)]
pub struct CharacterController;

/// A marker component indicating that an entity is on the ground.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;
/// The acceleration used for character movement.
#[derive(Component)]
pub struct MovementAcceleration(f32);

/// The damping factor used for slowing down movement.
#[derive(Component)]
pub struct MovementDampingFactor(f32);

/// The strength of a jump.
#[derive(Component)]
pub struct JumpImpulse(f32);

/// A bundle that contains the components needed for a basic
/// kinematic character controller.
#[derive(Bundle)]
pub struct CharacterControllerBundle {
    character_controller: CharacterController,
    collider: AabbCollider,
    //ground_caster: ShapeCaster,
    //locked_axes: LockedAxes,
    movement: MovementBundle,
}

/// A bundle that contains components for character movement.
#[derive(Bundle)]
pub struct MovementBundle {
    acceleration: MovementAcceleration,
    jump_impulse: JumpImpulse,
}

impl MovementBundle {
    pub const fn new(
        acceleration: f32,
        jump_impulse: f32,
    ) -> Self {
        Self {
            acceleration: MovementAcceleration(acceleration),
            jump_impulse: JumpImpulse(jump_impulse),
        }
    }
}

impl Default for MovementBundle {
    fn default() -> Self {
        Self::new(30.0, 7.0)
    }
}

impl CharacterControllerBundle {
    pub fn new(collider: AabbCollider) -> Self {
        //caster_shape.set_scale(Vec3::ONE * 0.99, 10);

        Self {
            character_controller: CharacterController,
            collider,
            /*
            ground_caster: ShapeCaster::new(
                caster_shape,
                Vector::ZERO,
                Quaternion::default(),
                Direction3d::NEG_Y,
            )
            .with_max_time_of_impact(0.2),
             */
            movement: MovementBundle::default(),
        }
    }

    pub fn with_movement(
        mut self,
        acceleration: f32,
        jump_impulse: f32,
    ) -> Self {
        self.movement = MovementBundle::new(acceleration, jump_impulse);
        self
    }
}



/// Updates the [`Grounded`] status for character controllers.
/*
pub fn update_grounded(
    mut commands: Commands,
    mut query: Query<
        (Entity, &ShapeHits, &Rotation, Option<&MaxSlopeAngle>),
        With<CharacterController>,
    >,
) {
    //println!("sbeeef");
    for (entity, hits, rotation, max_slope_angle) in &mut query {
        // The character is grounded if the shape caster has a hit with a normal
        // that isn't too steep.
        let is_grounded = hits.iter().any(|hit| {
            if let Some(angle) = max_slope_angle {
                rotation.rotate(-hit.normal2).angle_between(Vector::Y).abs() <= angle.0
            } else {
                true
            }
        });

        if is_grounded {
            commands.entity(entity).insert(Grounded);
        } else {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}
 */

/// Responds to [`MovementAction`] events and moves character controllers accordingly.
pub fn movement(
    time: Res<Time>,
    mut movement_event_reader: EventReader<MovementAction>,
    mut controllers: Query<(&MovementAcceleration, &JumpImpulse, &mut LinearVelocity, &SurfaceContacts)>,
) {
    let delta_time = time.delta_seconds();

    for event in movement_event_reader.read() {
        
        if let Ok((movement_acceleration, jump_impulse, mut linear_velocity, surface_contacts)) = controllers.get_mut(event.entity) {
            match event.movement {
                MovementType::Move(direction) => {
                    linear_velocity.x += direction.x * movement_acceleration.0 * delta_time;
                    linear_velocity.z += direction.y * movement_acceleration.0 * delta_time;
                }
                MovementType::Jump => {
                    if surface_contacts.contains(&SurfaceContact::PosY) || surface_contacts.contains(&SurfaceContact::Water) || surface_contacts.contains(&SurfaceContact::Climable) {
                        linear_velocity.y = jump_impulse.0;
                    }
                }
            }
        }
        
    }
}