use std::time::Duration;

use bevy::prelude::*;

use bevy_xpbd_3d::plugins::spatial_query::{self, SpatialQuery, SpatialQueryFilter};
use movement::*;

use crate::{Block, Chunk, ChunkMap, CHUNK_SIZE};
pub mod movement;


pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_event::<MovementAction>()
        .add_event::<MiningEvent>()
        .add_event::<DamageBlockEvent>();
    }
}

/// If you're not start. you're end.
#[derive(Clone, Copy, Event)]
pub struct MiningEvent {
    pub entity: Entity,
    pub is_start: bool,
}

#[derive(Clone, Copy, Event)]
pub struct DamageBlockEvent {
    pub position: IVec3,
    pub damage: u8,
    pub strength: u8,
}



#[derive(Clone, Component, Deref, DerefMut, Debug)]
pub struct MiningTimer (pub Timer);
impl Default for MiningTimer {
    fn default() -> Self {
        let mut timer = Timer::new(Duration::from_millis(750), TimerMode::Repeating);
        timer.pause();
        Self(timer)
    }
}


pub fn mining (
    //mut commands: Commands,

    mut miner_query: Query<(Entity, &mut MiningTimer, &Children)>,
    // TODO: We should use some "head" component or something later on when we have entities that mine but don't have a camera.
    cam_query: Query<(&GlobalTransform), With<Camera>>,
    spatial_query: SpatialQuery,
    
    mut evr_mining: EventReader<MiningEvent>,
    mut evw_damage_block: EventWriter<DamageBlockEvent>,

    time: Res<Time>,
) {
    for (entity, mut timer, children) in &mut miner_query {
        timer.tick(time.delta());

        if timer.finished() {

            for child in children.iter() {
                if let Ok(global_transform) = cam_query.get(*child) {
                    for hit in spatial_query.ray_hits(
                        global_transform.translation(),
                        // TODO: I don't think we have to normalize this, actually? IDK.
                        Direction3d::new_unchecked(global_transform.forward().normalize()),
                        5.0,                         
                        1,                             
                        true,                          
                        SpatialQueryFilter::default().with_excluded_entities(vec![entity]), 
                    ) {
                        let hit_point = global_transform.translation() + global_transform.forward() * hit.time_of_impact;
                        //println!("forward: {}", global_transform.forward());
                        //println!("normalized forward: {}", global_transform.forward().normalize());
                        let hit_coords = (hit_point - hit.normal / 2.0).round().as_ivec3();
                        println!("hit: {:?}, hit_point: {:?}", hit, hit_point);
                        println!("hit_coords: {:?}", hit_coords);

                        evw_damage_block.send(DamageBlockEvent { position: hit_coords, damage: 1, strength: 1 });
                    }
                }
            }
        }
    }

    // TODO: Should this be a separate system?
    for ev in evr_mining.read() {
        if let Ok((_, mut timer, _)) = miner_query.get_mut(ev.entity) {
            if ev.is_start {
                timer.unpause();
            }
            else {
                timer.pause();
                timer.reset();
            }
        }
    }
}

pub fn damage_block (
    mut chunk_query: Query<&mut Chunk>,

    mut chunk_map: ResMut<ChunkMap>,

    mut evr_damage_block: EventReader<DamageBlockEvent>,
) {
    for ev in evr_damage_block.read() {
        let mut modified_pos = ev.position;
        
        // TODO: This, and the block of if statements below don't feel very elegant. Perhaps we could get a more mathy solution somehow? Would be nice.
        // TODO: We should probably make this process of getting chunk coords and local block coords into some function somewhere. Definitely want to do that if we end up needing to do this again. (Likely)
        // This block of if statements and the one below handle, of course, getting the positions of negative blocks.
        if ev.position.x < 0 {modified_pos.x = ev.position.x - 15};
        if ev.position.y < 0 {modified_pos.y = ev.position.y - 15};
        if ev.position.z < 0 {modified_pos.z = ev.position.z - 15};

        let chunk_pos = modified_pos / CHUNK_SIZE;

        if let Some(chunk_entity) = chunk_map.get(&chunk_pos) {
            if let Ok(mut chunk) = chunk_query.get_mut(*chunk_entity) {
                
                let mut block_pos = ev.position % CHUNK_SIZE;

                if block_pos.x < 0 {block_pos.x += CHUNK_SIZE};
                if block_pos.y < 0 {block_pos.y += CHUNK_SIZE};
                if block_pos.z < 0 {block_pos.z += CHUNK_SIZE};
                
                println!("position: {}", ev.position);
                println!("chunk pos: {}", chunk_pos);
                println!("block pos: {}", block_pos);
                let attributes = chunk[block_pos].get_attributes();
                
                if ev.strength >= attributes.toughness {
                    chunk[block_pos].damage = chunk[block_pos].damage + ev.damage;
                    
                    if chunk[block_pos].damage == attributes.health {
                        chunk[block_pos] = Block::new(attributes.breaks_into);
                        println!("new block: {:?}", attributes.breaks_into);
                    }
                }
                    
            }
        }
    }
}