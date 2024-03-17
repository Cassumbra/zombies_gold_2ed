use bevy::{prelude::*, utils::HashSet};

use crate::{block_pos_from_global, chunk_pos_from_global, BlockID, Chunk, ChunkMap};

const BLOCK_AABB: AabbCollider = AabbCollider{ width: 1.0, height: 1.0, length: 1.0 };

pub fn do_physics(
    mut commands: Commands,

    mut query: Query<(Entity, &mut Transform, &mut LinearVelocity, Option<&AabbCollider>,)>,
    mut chunk_query: Query<(&Chunk)>,

    time: Res<Time>,
    chunk_map: Res<ChunkMap>,

) {
    for (entity, mut transform, mut velocity, opt_collider) in &mut query {
        transform.translation += **velocity * time.delta_seconds();

        if let Some(collider) = opt_collider {
            for x in (transform.translation.x as i32 - 3)..=(transform.translation.x as i32 + 3) {
                for y in (transform.translation.y as i32 - 3)..=(transform.translation.y as i32 + 3) {
                    for z in (transform.translation.z as i32 - 3)..=(transform.translation.z as i32 + 3) {
                        let global_block_position = IVec3::new(x, y, z);
                        let chunk_position = chunk_pos_from_global(global_block_position);
                        let block_position = block_pos_from_global(global_block_position);
    
                        if let Some(chunk_entity) = chunk_map.get(&chunk_position) {
                            if let Ok(chunk) = chunk_query.get(*chunk_entity) {
                                if chunk[block_position].id != BlockID::Air {
                                    let (penetration, normal) = collider.get_penetration_and_normal(transform.translation, BLOCK_AABB, global_block_position.as_vec3());
                                    if normal != Vec3::ZERO {
                                        println!("Penetration: {}, Normal: {}", penetration, normal);
                                    }
                                }
                                continue;
                            }
                        }
                        // OOB check
                        if collider.get_intersection(transform.translation, BLOCK_AABB, global_block_position.as_vec3()) {
                            println!("OOB at {}", global_block_position);
                        }
                    }
                }
            }
        }
    }
}

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
pub struct LinearVelocity(pub Vec3);

#[derive(Component, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct SurfaceContacts(pub HashSet<SurfaceContact>);

#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Reflect)]
pub enum SurfaceContact{
    Ceiling,
    Ground,
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