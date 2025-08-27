use std::time::Duration;

use bevy::{prelude::*, time::Stopwatch};

use movement::*;

use crate::{block_pos_from_global, chunk_pos_from_global, hotbar::{Hotbar, SlotAction}, raycast_blocks, update_chunk_events_from_global, Block, BlockID, BlockUpdateEvent, Chunk, ChunkMap, Inventory, UpdateChunkEvent, CHUNK_SIZE};
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
    mut miner_query: Query<(Entity, &mut MiningTimer, &Children)>,
    // TODO: We should use some "head" component or something later on when we have entities that mine but don't have a camera.
    cam_query: Query<(&GlobalTransform), With<Camera>>,

    chunk_map: Res<ChunkMap>,
    
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

                        if let Some(chunk) = chunk_map.get(&chunk_pos) {
                            let block_pos = block_pos_from_global(hit.position.as_ivec3());

                            if chunk.blocks[block_pos].id != BlockID::Air && chunk.blocks[block_pos].id != BlockID::Water {
                                evw_damage_block.send(DamageBlockEvent { position: hit.position.as_ivec3(), damage: 1, strength: 1, entity });
                                break;
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
    mut inventory_query: Query<&mut Inventory>,

    mut chunk_map: ResMut<ChunkMap>,

    mut evr_damage_block: EventReader<DamageBlockEvent>,
    mut evw_update_chunk: EventWriter<UpdateChunkEvent>,
    mut evw_block_update: EventWriter<BlockUpdateEvent>,
) {
    for ev in evr_damage_block.read() {
        let chunk_pos = chunk_pos_from_global(ev.position);

        if let Some(chunk) = chunk_map.get_mut(&chunk_pos) {
            let block_pos = block_pos_from_global(ev.position);
            //println!("position: {}", ev.position);
            //println!("chunk pos: {}", chunk_pos);
            //println!("block pos: {}", block_pos);
            let attributes = chunk.blocks[block_pos].get_attributes();
            
            if ev.strength >= attributes.toughness {
                chunk.blocks[block_pos].damage = chunk.blocks[block_pos].damage + ev.damage;

                // TODO: Should we condense things by just sending these when we handle block updates?
                for event in update_chunk_events_from_global(ev.position) {
                    evw_update_chunk.send(event);
                }

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
                
                if chunk.blocks[block_pos].damage == attributes.health {
                    chunk.blocks[block_pos] = Block::new(attributes.breaks_into);
                    //println!("new block: {:?}", attributes.breaks_into);
                    evw_block_update.send(BlockUpdateEvent { position: ev.position, time_waited: Stopwatch::new() });
                }
            }
        }
    }
}

pub fn building (
    // TODO: Add some component to tell us what to actually build.
    mut builder_query: Query<(Entity, &mut BuildingTimer, &Children, &Hotbar)>,
    // TODO: We should use some "head" component or something later on when we have entities that build but don't have a camera.
    cam_query: Query<(&GlobalTransform), With<Camera>>,

    chunk_map: Res<ChunkMap>,
    
    mut evr_building: EventReader<BuildingEvent>,
    mut evw_put_block: EventWriter<PutBlockEvent>,

    time: Res<Time>,
) {
    for (entity, mut timer, children, hotbar) in &mut builder_query {
        match hotbar.slots[hotbar.position] {
            SlotAction::Block(block_id) => {
                timer.tick(time.delta());

                if timer.finished() {

                    for child in children.iter() {
                        if let Ok(global_transform) = cam_query.get(*child) {
                            let hits = raycast_blocks(global_transform.translation(), global_transform.forward().normalize(), 5.0);
                            for hit in hits {
                                //println!("hit_position: {}, hit_normal: {}", hit.position, hit.normal);

                                let chunk_pos = chunk_pos_from_global(hit.position.as_ivec3());

                                if let Some(chunk) = chunk_map.get(&chunk_pos) {
                                    let block_pos = block_pos_from_global(hit.position.as_ivec3());

                                    if chunk.blocks[block_pos].id != BlockID::Air && chunk.blocks[block_pos].id != BlockID::Water {
                                        evw_put_block.send(PutBlockEvent { position: hit.position.as_ivec3() + hit.normal.as_ivec3(), id: block_id, entity } );
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            },
            _ => continue,
        }

        
    }

    // TODO: Should this be a separate system?
    for ev in evr_building.read() {
        if let Ok((_, mut timer, _, _)) = builder_query.get_mut(ev.entity) {
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
    mut inventory_query: Query<&mut Inventory>,

    mut chunk_map: ResMut<ChunkMap>,

    mut evr_put_block: EventReader<PutBlockEvent>,
    mut evw_update_chunk: EventWriter<UpdateChunkEvent>,
    mut evw_block_update: EventWriter<BlockUpdateEvent>,
) {
    'events: for ev in evr_put_block.read() {
        let chunk_pos = chunk_pos_from_global(ev.position);

        if let Some(chunk) = chunk_map.get_mut(&chunk_pos) {
            let block_pos = block_pos_from_global(ev.position);
                
            if chunk.blocks[block_pos].id == BlockID::Air || chunk.blocks[block_pos].id == BlockID::Water {
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

                chunk.blocks[block_pos] = Block::new(ev.id);
                evw_block_update.send(BlockUpdateEvent { position: ev.position, time_waited: Stopwatch::new() });

                for event in update_chunk_events_from_global(ev.position) {
                    evw_update_chunk.send(event);
                } 
            }
        }
    }
}