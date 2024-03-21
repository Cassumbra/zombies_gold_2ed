use std::time::Duration;

use bevy::prelude::*;

use movement::*;

use crate::{block_pos_from_global, chunk_pos_from_global, raycast_blocks, Block, BlockID, Chunk, ChunkMap, Inventory, CHUNK_SIZE};
pub mod movement;


pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_event::<MovementAction>()
        .add_event::<MiningEvent>()
        .add_event::<DamageBlockEvent>()
        .add_event::<BuildingEvent>()
        .add_event::<PutBlockEvent>();
    }
}

/// If you're not start. you're end.
#[derive(Clone, Copy, Event)]
pub struct MiningEvent {
    pub entity: Entity,
    pub is_start: bool,
}

/// If you're not start. you're end.
#[derive(Clone, Copy, Event)]
pub struct BuildingEvent {
    pub entity: Entity,
    pub is_start: bool,
}

#[derive(Clone, Copy, Event)]
pub struct DamageBlockEvent {
    pub position: IVec3,
    pub damage: u8,
    pub strength: u8,
    pub entity: Entity,
}

#[derive(Clone, Copy, Event)]
pub struct PutBlockEvent {
    pub position: IVec3,
    pub id: BlockID,
    pub entity: Entity,
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

#[derive(Clone, Component, Deref, DerefMut, Debug)]
pub struct BuildingTimer (pub Timer);
impl Default for BuildingTimer {
    fn default() -> Self {
        let mut timer = Timer::new(Duration::from_millis(250), TimerMode::Repeating);
        timer.pause();
        Self(timer)
    }
}

pub fn mining (
    //mut commands: Commands,
    mut chunk_query: Query<&mut Chunk>,
    mut miner_query: Query<(Entity, &mut MiningTimer, &Children)>,
    // TODO: We should use some "head" component or something later on when we have entities that mine but don't have a camera.
    cam_query: Query<(&GlobalTransform), With<Camera>>,

    mut chunk_map: ResMut<ChunkMap>,
    
    mut evr_mining: EventReader<MiningEvent>,
    mut evw_damage_block: EventWriter<DamageBlockEvent>,

    time: Res<Time>,
) {
    for (entity, mut timer, children) in &mut miner_query {
        timer.tick(time.delta());

        if timer.finished() {

            for child in children.iter() {
                if let Ok(global_transform) = cam_query.get(*child) {
                    let hits = raycast_blocks(global_transform.translation(), global_transform.forward().normalize(), 5.0);
                    for hit in hits {
                        //println!("hit_position: {}, hit_normal: {}", hit.position, hit.normal);

                        let chunk_pos = chunk_pos_from_global(hit.position.as_ivec3());

                        if let Some(chunk_entity) = chunk_map.get(&chunk_pos) {
                            if let Ok(chunk) = chunk_query.get_mut(*chunk_entity) {
                                let block_pos = block_pos_from_global(hit.position.as_ivec3());

                                if chunk[block_pos].id != BlockID::Air {
                                    evw_damage_block.send(DamageBlockEvent { position: hit.position.as_ivec3(), damage: 1, strength: 1, entity });
                                    break;
                                }
                            }
                        }
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
    mut inventory_query: Query<&mut Inventory>,

    mut chunk_map: ResMut<ChunkMap>,

    mut evr_damage_block: EventReader<DamageBlockEvent>,
) {
    for ev in evr_damage_block.read() {
        let chunk_pos = chunk_pos_from_global(ev.position);

        if let Some(chunk_entity) = chunk_map.get(&chunk_pos) {
            if let Ok(mut chunk) = chunk_query.get_mut(*chunk_entity) {
                let block_pos = block_pos_from_global(ev.position);

                //println!("position: {}", ev.position);
                //println!("chunk pos: {}", chunk_pos);
                //println!("block pos: {}", block_pos);
                let attributes = chunk[block_pos].get_attributes();
                
                if ev.strength >= attributes.toughness {
                    chunk[block_pos].damage = chunk[block_pos].damage + ev.damage;

                    if let Some(drop) = attributes.give_on_damage {
                        if let Ok(mut inventory) = inventory_query.get_mut(ev.entity) {
                            if let Err(fault) = inventory.insert_item(drop) {
                                match fault {
                                    crate::ItemInsertFault::NoSpace => todo!("We should drop the item as an entity!"),
                                    crate::ItemInsertFault::InsufficientSpace => todo!("We should drop some of the item as an entity!"),
                                }
                            }
                        }
                    }
                    
                    if chunk[block_pos].damage == attributes.health {
                        chunk[block_pos] = Block::new(attributes.breaks_into);
                        //println!("new block: {:?}", attributes.breaks_into);
                    }
                }
                    
            }
        }
    }
}

pub fn building (
    mut chunk_query: Query<&mut Chunk>,
    // TODO: Add some component to tell us what to actually build.
    mut builder_query: Query<(Entity, &mut BuildingTimer, &Children)>,
    // TODO: We should use some "head" component or something later on when we have entities that build but don't have a camera.
    cam_query: Query<(&GlobalTransform), With<Camera>>,

    mut chunk_map: ResMut<ChunkMap>,
    
    mut evr_building: EventReader<BuildingEvent>,
    mut evw_put_block: EventWriter<PutBlockEvent>,

    time: Res<Time>,
) {
    for (entity, mut timer, children) in &mut builder_query {
        timer.tick(time.delta());

        if timer.finished() {

            for child in children.iter() {
                if let Ok(global_transform) = cam_query.get(*child) {
                    let hits = raycast_blocks(global_transform.translation(), global_transform.forward().normalize(), 5.0);
                    for hit in hits {
                        //println!("hit_position: {}, hit_normal: {}", hit.position, hit.normal);

                        let chunk_pos = chunk_pos_from_global(hit.position.as_ivec3());

                        if let Some(chunk_entity) = chunk_map.get(&chunk_pos) {
                            if let Ok(chunk) = chunk_query.get_mut(*chunk_entity) {
                                let block_pos = block_pos_from_global(hit.position.as_ivec3());

                                if chunk[block_pos].id != BlockID::Air {
                                    evw_put_block.send(PutBlockEvent { position: hit.position.as_ivec3() + hit.normal.as_ivec3(), id: BlockID::StoneBrick, entity } );
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // TODO: Should this be a separate system?
    for ev in evr_building.read() {
        if let Ok((_, mut timer, _)) = builder_query.get_mut(ev.entity) {
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

pub fn place_block (
    mut chunk_query: Query<&mut Chunk>,
    mut inventory_query: Query<&mut Inventory>,

    mut chunk_map: ResMut<ChunkMap>,

    mut evr_put_block: EventReader<PutBlockEvent>,
) {
    'events: for ev in evr_put_block.read() {
        let chunk_pos = chunk_pos_from_global(ev.position);

        if let Some(chunk_entity) = chunk_map.get(&chunk_pos) {
            if let Ok(mut chunk) = chunk_query.get_mut(*chunk_entity) {
                let block_pos = block_pos_from_global(ev.position);
                
                if chunk[block_pos].id == BlockID::Air {
                    if let Ok(mut inventory) = inventory_query.get_mut(ev.entity) {
                        let attributes = ev.id.get_attributes();

                        for cost_opt in attributes.cost_to_build {
                            if let Some(cost) = cost_opt {
                                if cost.amount > inventory.get_item_amount(cost.id) {
                                    // TODO: Make this some kind of proper in game indicator.
                                    println!("Not enough to build!");
                                    continue 'events;
                                }
                            }
                        }

                        for cost_opt in attributes.cost_to_build {
                            if let Some(cost) = cost_opt {
                                let _ = inventory.take_item(cost);
                            }
                        }
                    }

                    chunk[block_pos] = Block::new(ev.id);
                }
                    
            }
        }
    }
}