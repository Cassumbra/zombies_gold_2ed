use bevy::{prelude::*, utils::HashSet};
use itertools::{iproduct, izip};

use crate::{block_pos_from_global, chunk_pos_from_global, BlockID, Chunk, ChunkMap, HasAir, Solidity};

pub const BLOCK_AABB: AabbCollider = AabbCollider{ width: 1.0, height: 1.0, length: 1.0 };

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_event::<FallEvent>();
    }
}

//Events
#[derive(Clone, Copy, Event)]
pub struct FallEvent {
    pub distance: f32,
    pub entity: Entity,
}
impl FallEvent {
    pub fn new(distance: f32, entity: Entity) -> Self {
        FallEvent {distance, entity}
    }
}


pub fn apply_gravity (
    //mut commands: Commands,

    mut query: Query<(&mut LinearVelocity, &Gravity)>,
    //mut chunk_query: Query<(&Chunk)>,

    time: Res<Time>,
    //chunk_map: Res<ChunkMap>,
) {
    for (mut velocity, gravity) in &mut query {
        velocity.y -= gravity.0 * time.delta_seconds();
    }
}

pub fn do_physics (
    mut commands: Commands,

    mut query: Query<(Entity, &mut Transform, &mut LinearVelocity, &mut DistanceBeforeCollision, Option<&AabbCollider>, Option<&mut HasAir>)>,

    time: Res<Time>,
    chunk_map: Res<ChunkMap>,
    mut evr_fall: EventWriter<FallEvent>,
) {
    let dist_bf_collision_calc = |dist_bf_collision: Vec3, distance: Vec3| -> Vec3 {
        let mut dbfc = dist_bf_collision;

        for i in 0..=2 {
            if dbfc[i] == 0.0 {
                dbfc[i] += distance[i];
            }
            else if dbfc[i].signum() != distance[i].signum() {
                dbfc[i] = 0.0;
            }
            else {// dbfc[i] == 0.0 {
               dbfc[i] += distance[i]; 
            }
        }
        //println!("heheheha: {}", dbfc);
        dbfc
    };

    for (entity, mut transform, mut velocity, mut dist_bf_collision, opt_collider, mut opt_has_air) in &mut query {
        let frame_velocity = **velocity * time.delta_seconds();

        if let Some(collider) = opt_collider {
            let step_count = (frame_velocity * 4.0).abs().max_element().ceil() as i32;
            
            let mut in_water = false;
            let mut mouth_submerged = false;

            //println!("vrooms: {}", frame_velocity);
            //println!("steps: {}", step_count);

            let mut surface_contacts = SurfaceContacts(HashSet::new());
            // Air slip
            //let mut applied_slip = Vec3::new(0.975, 0.99, 0.975);
            let mut applied_slip = Vec3::new(0.05, 1.0, 0.05);

            for _ in 0..step_count {
                let frame_velocity = **velocity * time.delta_seconds();
                let step_velocity = frame_velocity / step_count as f32;
                transform.translation += step_velocity;
                **dist_bf_collision = dist_bf_collision_calc(**dist_bf_collision, step_velocity);

                let mut collisions: Vec<BlockCollision> = iproduct!((transform.translation.x - collider.width.ceil()) as i32..=(transform.translation.x + collider.width.ceil()) as i32, 
                                                                (transform.translation.y - collider.height.ceil()) as i32..=(transform.translation.y + collider.height.ceil()) as i32,
                                                                (transform.translation.z - collider.length.ceil()) as i32..=(transform.translation.z + collider.length.ceil()) as i32).filter_map(|(x, y, z)| {
                    let global_block_position = IVec3::new(x, y, z);
                    let chunk_position = chunk_pos_from_global(global_block_position);
                    let block_position = block_pos_from_global(global_block_position);

                    if let Some(chunk) = chunk_map.get(&chunk_position) {
                        match chunk.blocks[block_position].get_attributes().solidity {
                            Solidity::Solid => {
                                let (penetration, normal) = collider.get_penetration_and_normal(transform.translation, BLOCK_AABB, global_block_position.as_vec3());
                                if normal != Vec3::ZERO {
                                    return Some(BlockCollision::new(global_block_position, penetration, normal, Some(chunk.blocks[block_position].id)));
                                }
                            },
                            Solidity::NonSolid => {},
                            Solidity::Water => {
                                **dist_bf_collision = Vec3::ZERO;
                                let mut top_box = *collider;
                                top_box.height /= 2.0;
                                let mut top_box_pos = transform.translation;
                                top_box_pos.y += top_box.height / 2.0;

                                let mut mouth_box = top_box;
                                mouth_box.height /= 4.0;
                                mouth_box.width /= 4.0;
                                mouth_box.length /= 4.0;
                                let mut mouth_box_pos = top_box_pos;
                                mouth_box_pos.y += top_box.height / 8.0;
                                if !in_water {
                                    in_water = top_box.get_intersection(top_box_pos, BLOCK_AABB, global_block_position.as_vec3());
                                }
                                if !mouth_submerged {
                                    mouth_submerged = mouth_box.get_intersection(mouth_box_pos, BLOCK_AABB, global_block_position.as_vec3());
                                }
                            },
                        }
                        return None;
                    }
                    // OOB check
                    let (penetration, normal) = collider.get_penetration_and_normal(transform.translation, BLOCK_AABB, global_block_position.as_vec3());
                    if normal != Vec3::ZERO {
                        return Some(BlockCollision::new(global_block_position, penetration, normal, None));
                        //println!("OOB at {}", global_block_position);
                    }

                    return None;
                }).collect();

                collisions.sort_unstable_by(|collision_a, collision_b| collision_b.penetration.partial_cmp(&collision_a.penetration).unwrap());

                let mut collisions_new = Vec::<BlockCollision>::new();
                if in_water {applied_slip = Vec3::new(0.005, 0.0025, 0.005)};

                for (i, collision) in collisions.iter().enumerate() {
                    let (penetration, normal) = if i != 0 {collider.get_penetration_and_normal(transform.translation, BLOCK_AABB, collision.position.as_vec3())} else {(collision.penetration, collision.normal)};

                    if normal != Vec3::ZERO {
                        //if normal.y != 1.0  {
                        //    println!("Penetration: {}, Normal: {}", penetration, normal);
                        //}
                        transform.translation += penetration * normal;
                        **dist_bf_collision += penetration * normal;
                        
                        for i in 0..=2 {
                            let mut surface_contact = SurfaceContact::NegX;

                            if normal[i] < 0.0 && velocity[i] > 0.0 {
                                velocity[i] = 0.0;
                                

                                match i {
                                    0 => {
                                            dist_bf_collision.x = 0.0;
                                            surface_contact = SurfaceContact::NegX;
                                         }
                                    1 => {
                                            dist_bf_collision.y = 0.0;
                                            surface_contact = SurfaceContact::NegY;
                                         }
                                    2 => {
                                            dist_bf_collision.z = 0.0;
                                            surface_contact = SurfaceContact::NegZ;
                                         }
                                    _ => panic!(),
                                }
                            }
                            else if normal[i] > 0.0 && velocity[i] < 0.0 {
                                velocity[i] = 0.0;

                                match i {
                                    0 => {
                                            dist_bf_collision.x = 0.0;
                                            surface_contact = SurfaceContact::PosX;
                                         }
                                    1 => {
                                            if dist_bf_collision.y < -0.05 {
                                                evr_fall.send(FallEvent::new(dist_bf_collision.y, entity));
                                            }
                                            
                                            dist_bf_collision.y = 0.0;
                                            surface_contact = SurfaceContact::PosY;
                                         }
                                    2 => {
                                            dist_bf_collision.z = 0.0;
                                            surface_contact = SurfaceContact::PosZ;
                                         }
                                    _ => panic!(),
                                }
                            }

                            if surface_contact == SurfaceContact::PosY || surface_contact == SurfaceContact::NegY {
                                if let Some(id) = collision.id {
                                    applied_slip.x = id.get_attributes().slip.x;
                                    applied_slip.z = id.get_attributes().slip.z;
                                }
                                else {
                                    applied_slip.x = 0.0;
                                    applied_slip.z = 0.0;
                                }
                            }
                            else {
                                if let Some(id) = collision.id {
                                    applied_slip.y = id.get_attributes().slip.y;
                                }
                                else {
                                    applied_slip.y = 0.0;
                                }
                            }

                            

                            surface_contacts.insert(surface_contact);

                            //if i != 1 && velocity[i] == 0.0 {
                            //    println!("velocity zeroed");
                            //}
                        }

                        collisions_new.push(BlockCollision::new(collision.position, penetration, normal, collision.id));
                    }
                }
            }

            if let Some(ref mut has_air) = opt_has_air {
                ***has_air = !mouth_submerged;
            }

            let _ = if in_water {surface_contacts.insert(SurfaceContact::Water)} else {false};

            commands.get_entity(entity).unwrap().insert(surface_contacts).insert(AppliedSlip(applied_slip));
            //commands.get_entity(entity).unwrap().insert(BlockCollisions(collisions_new));
            //println!("Collisions: {:?}", collisions_new);

        }
        else {
            transform.translation += frame_velocity;
            **dist_bf_collision = dist_bf_collision_calc(**dist_bf_collision, frame_velocity);
        }
    }
}

pub fn apply_friction(
    mut query: Query<(&AppliedSlip, &mut LinearVelocity)>,
    time: Res<Time>,
) {
    for (applied_slip, mut linear_velocity) in &mut query {
        **linear_velocity *= applied_slip.powf(time.delta_seconds());
    }
}


// https://gamedev.stackexchange.com/a/49423
// Imagine writing an answer in JAVASCRIPT
// Credit to inspi for letting me look at their code to help make this more sane and also actually work
pub fn raycast_blocks (mut origin: Vec3, direction: Vec3, mut range: f32) -> Vec<RayCastHit> {
    if direction == Vec3::ZERO {
        panic!("Raycast in zero direction!");
    }

    origin = origin + 0.5;

    //println!("origin: {}, direction: {}", origin, direction);

    let step = direction.signum();
    let next = origin.floor() + step.max(Vec3::ZERO);
    let mut t_max = (next - origin) / direction;
    let t_delta = 1.0 / direction.abs();

    let mut hits = Vec::<RayCastHit>::new();
    let mut hit = RayCastHit::new(origin.floor(), Vec3::ZERO);

    // Rescale from units of 1 cube-edge to units of 'direction' so we can
    // compare with 't'.
    range /= direction.length();

    for _ in 0..500 {
        //println!("t_max: {}", t_max);

        hits.push(hit);

        if t_max.x < t_max.y {
            if t_max.x < t_max.z {
                if t_max.x > range {break};

                hit.position.x += step.x;
                t_max.x += t_delta.x;

                hit.normal = Vec3::new(-step.x, 0.0, 0.0);
            }
            else {
                if t_max.z > range {break};

                hit.position.z += step.z;
                t_max.z += t_delta.z;

                hit.normal = Vec3::new(0.0, 0.0, -step.z);
            }
        }
        else {
            if t_max.y < t_max.z {
                if t_max.y > range {break};

                hit.position.y += step.y;
                t_max.y += t_delta.y;

                hit.normal = Vec3::new(0.0, -step.y, 0.0);
            }
            else {
                if t_max.z > range {break};

                hit.position.z += step.z;
                t_max.z += t_delta.z;

                hit.normal = Vec3::new(0.0, 0.0, -step.z);
            }
        }
    }

    return hits;
}

#[derive(Copy, Clone, Debug, Reflect)]
pub struct RayCastHit {
    pub position: Vec3,
    pub normal: Vec3,
}
impl RayCastHit {
    pub fn new(position: Vec3, normal: Vec3) -> RayCastHit {
        RayCastHit {position, normal}
    }
}

// TODO: We should probably store a bool that tells us if this is an OOB collision or not.
#[derive(Copy, Clone, Debug, Reflect)]
pub struct BlockCollision {
    pub position: IVec3,
    pub penetration: f32,
    pub normal: Vec3,
    pub id: Option<BlockID>,
}
impl BlockCollision {
    pub fn new(position: IVec3, penetration: f32, normal: Vec3, id: Option<BlockID>) -> BlockCollision {
        BlockCollision {position, penetration, normal, id}
    }
}

/// From 0 to 1. Anything else will result in ?strange? behavior.
#[derive(Component, Clone, Copy, Debug, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct Slip(Vec3);
impl Default for Slip {
    fn default() -> Self {
        Self(Vec3::new(0.015, 1.0, 0.015))
    }
}

/// The amount of slip applied to this object that it will experience during movement.
#[derive(Component, Clone, Debug, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct AppliedSlip(Vec3);


#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct BlockCollisions(Vec<BlockCollision>);

#[derive(Component, Default, Copy, Clone, Reflect)]
#[reflect(Component)]
pub struct AabbCollider {
    pub width: f32,
    pub height: f32,
    pub length: f32,
}
impl AabbCollider {
    pub fn new(width: f32, height: f32, length: f32) -> AabbCollider {
        AabbCollider { width, height, length }
    }

    pub fn get_point_intersection(&self, position: Vec3, point: Vec3) -> bool {
        let self_min = position - Vec3::new(self.width, self.height, self.length) / 2.0;
        let self_max = position + Vec3::new(self.width, self.height, self.length) / 2.0;

        self_min.x <= point.x &&
        self_max.x >= point.x &&
        self_min.y <= point.y &&
        self_max.y >= point.y &&
        self_min.z <= point.z &&
        self_max.z >= point.z
    }

    pub fn get_intersection(&self, position: Vec3, other_aabb: AabbCollider, other_position: Vec3) -> bool {
        let self_min = position - Vec3::new(self.width, self.height, self.length) / 2.0;
        let self_max = position + Vec3::new(self.width, self.height, self.length) / 2.0;

        let other_min = other_position - Vec3::new(other_aabb.width, other_aabb.height, other_aabb.length) / 2.0;
        let other_max = other_position + Vec3::new(other_aabb.width, other_aabb.height, other_aabb.length) / 2.0;

        self_min.x <= other_max.x &&
        self_max.x >= other_min.x &&
        self_min.y <= other_max.y &&
        self_max.y >= other_min.y &&
        self_min.z <= other_max.z &&
        self_max.z >= other_min.z
    }

    pub fn get_penetration_and_normal(&self, position: Vec3, other_aabb: AabbCollider, other_position: Vec3) -> (f32, Vec3) {
        let self_min = position - Vec3::new(self.width, self.height, self.length) / 2.0;
        let self_max = position + Vec3::new(self.width, self.height, self.length) / 2.0;

        let other_min = other_position - Vec3::new(other_aabb.width, other_aabb.height, other_aabb.length) / 2.0;
        let other_max = other_position + Vec3::new(other_aabb.width, other_aabb.height, other_aabb.length) / 2.0;

        let colliding = self_min.x <= other_max.x &&
                        self_max.x >= other_min.x &&
                        self_min.y <= other_max.y &&
                        self_max.y >= other_min.y &&
                        self_min.z <= other_max.z &&
                        self_max.z >= other_min.z;

        if !colliding {
            return (0.0, Vec3::ZERO);
        }

        // No idea how to properly credit this but: https://research.ncl.ac.uk/game/mastersdegree/gametechnologies/physicstutorials/4collisiondetection/Physics%20-%20Collision%20Detection.pdf

        // I swapped the negatives and positives here because i feel like the normal for the to face should be pos y? I might be wrong though. no idea.
        let faces = vec![Vec3::X, Vec3::NEG_X,
                         Vec3::Y, Vec3::NEG_Y,
                         Vec3::Z, Vec3::NEG_Z];

        let distances = vec![other_max.x - self_min.x, self_max.x - other_min.x,
                             other_max.y - self_min.y, self_max.y - other_min.y,
                             other_max.z - self_min.z, self_max.z - other_min.z];

        let mut penetration = f32::MAX;
        let mut normal = Vec3::ZERO;

        for (i, face) in faces.iter().enumerate() {
            if distances[i] < penetration {
                penetration = distances[i];
                normal = *face;
            }
        }

        return(penetration, normal);
    }
}

#[derive(Component, Default, Copy, Clone, Deref, DerefMut)]
pub struct Gravity(pub f32);

#[derive(Component, Default, Copy, Clone, Deref, DerefMut)]
pub struct LinearVelocity(pub Vec3);

#[derive(Component, Default, Copy, Clone, Deref, DerefMut)]
pub struct DistanceBeforeCollision(pub Vec3);

#[derive(Component, Default, Clone, Deref, DerefMut)]
pub struct SurfaceContacts(pub HashSet<SurfaceContact>);

#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum SurfaceContact{
    PosY,
    NegY,
    PosX,
    NegX,
    PosZ,
    NegZ,
    // TODO: Should this be part of this enum? or should it be its own component?
    Water,
}

 /*
#[derive(Component, Default, Copy, Clone, Reflect)]
#[reflect(Component)]
pub struct SurfaceContacts {
    pub ceiling: bool,
    pub ground: bool,
    pub pos_x: bool,
    pub neg_x: bool,
    pub pos_z: bool,
    pub neg_z: bool,
}
 */