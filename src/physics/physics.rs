use bevy::{prelude::*, utils::HashSet};
use itertools::{iproduct, izip};

use crate::{block_pos_from_global, chunk_pos_from_global, BlockID, Chunk, ChunkMap};

const BLOCK_AABB: AabbCollider = AabbCollider{ width: 1.0, height: 1.0, length: 1.0 };

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

    mut query: Query<(Entity, &mut Transform, &mut LinearVelocity, Option<&AabbCollider>,)>,
    mut chunk_query: Query<(&Chunk)>,

    time: Res<Time>,
    chunk_map: Res<ChunkMap>,

) {
    for (entity, mut transform, mut velocity, opt_collider) in &mut query {
        let frame_velocity = **velocity * time.delta_seconds();

        if let Some(collider) = opt_collider {
            let step_count = (frame_velocity * 4.0).abs().max_element().ceil() as i32;
            

            //println!("vrooms: {}", frame_velocity);
            //println!("steps: {}", step_count);

            let mut surface_contacts = SurfaceContacts(HashSet::new());

            for _ in 0..step_count {
                let frame_velocity = **velocity * time.delta_seconds();
                let step_velocity = frame_velocity / step_count as f32;
                transform.translation += step_velocity;

                let mut collisions: Vec<BlockCollision> = iproduct!((transform.translation.x - collider.width.ceil()) as i32..=(transform.translation.x + collider.width.ceil()) as i32, 
                                                                (transform.translation.y - collider.height.ceil()) as i32..=(transform.translation.y + collider.height.ceil()) as i32,
                                                                (transform.translation.z - collider.length.ceil()) as i32..=(transform.translation.z + collider.length.ceil()) as i32).filter_map(|(x, y, z)| {
                    let global_block_position = IVec3::new(x, y, z);
                    let chunk_position = chunk_pos_from_global(global_block_position);
                    let block_position = block_pos_from_global(global_block_position);

                    if let Some(chunk_entity) = chunk_map.get(&chunk_position) {
                        if let Ok(chunk) = chunk_query.get(*chunk_entity) {
                            if chunk[block_position].id != BlockID::Air {
                                let (penetration, normal) = collider.get_penetration_and_normal(transform.translation, BLOCK_AABB, global_block_position.as_vec3());
                                if normal != Vec3::ZERO {
                                    return Some(BlockCollision::new(global_block_position, penetration, normal));
                                }
                            }
                            return None;
                        }
                    }
                    // OOB check
                    let (penetration, normal) = collider.get_penetration_and_normal(transform.translation, BLOCK_AABB, global_block_position.as_vec3());
                    if normal != Vec3::ZERO {
                        return Some(BlockCollision::new(global_block_position, penetration, normal));
                        //println!("OOB at {}", global_block_position);
                    }

                    return None;
                }).collect();

                collisions.sort_unstable_by(|collision_a, collision_b| collision_b.penetration.partial_cmp(&collision_a.penetration).unwrap());

                let mut collisions_new = Vec::<BlockCollision>::new();

                for (i, collision) in collisions.iter().enumerate() {
                    let (penetration, normal) = if i != 0 {collider.get_penetration_and_normal(transform.translation, BLOCK_AABB, collision.position.as_vec3())} else {(collision.penetration, collision.normal)};

                    if normal != Vec3::ZERO {
                        //if normal.y != 1.0  {
                        //    println!("Penetration: {}, Normal: {}", penetration, normal);
                        //}
                        transform.translation += penetration * normal;
                        
                        for i in 0..=2 {
                            let mut surface_contact = SurfaceContact::NegX;

                            if normal[i] < 0.0 && velocity[i] > 0.0 {
                                velocity[i] = 0.0;

                                match i {
                                    0 => surface_contact = SurfaceContact::NegX,
                                    1 => surface_contact = SurfaceContact::NegY,
                                    2 => surface_contact = SurfaceContact::NegZ,
                                    _ => panic!(),
                                }
                            }
                            else if normal[i] > 0.0 && velocity[i] < 0.0 {
                                velocity[i] = 0.0;

                                match i {
                                    0 => surface_contact = SurfaceContact::PosX,
                                    1 => surface_contact = SurfaceContact::PosY,
                                    2 => surface_contact = SurfaceContact::PosZ,
                                    _ => panic!(),
                                }
                            }

                            surface_contacts.insert(surface_contact);

                            //if i != 1 && velocity[i] == 0.0 {
                            //    println!("velocity zeroed");
                            //}
                        }

                        collisions_new.push(BlockCollision::new(collision.position, penetration, normal));
                    }
                }
            }

            commands.get_entity(entity).unwrap().insert(surface_contacts);
            //commands.get_entity(entity).unwrap().insert(BlockCollisions(collisions_new));
            //println!("Collisions: {:?}", collisions_new);

        }
        else {
            transform.translation += frame_velocity;
        }
    }
}

// TODO: We should probably store a bool that tells us if this is an OOB collision or not.
#[derive(Copy, Clone, Debug, Reflect)]
pub struct BlockCollision {
    pub position: IVec3,
    pub penetration: f32,
    pub normal: Vec3,
}
impl BlockCollision {
    pub fn new(position: IVec3, penetration: f32, normal: Vec3) -> BlockCollision {
        BlockCollision {position, penetration, normal}
    }
}

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

#[derive(Component, Default, Copy, Clone, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct Gravity(pub f32);

#[derive(Component, Default, Copy, Clone, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct LinearVelocity(pub Vec3);


#[derive(Component, Default, Clone, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct SurfaceContacts(pub HashSet<SurfaceContact>);

#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Reflect)]
pub enum SurfaceContact{
    PosY,
    NegY,
    PosX,
    NegX,
    PosZ,
    NegZ,
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