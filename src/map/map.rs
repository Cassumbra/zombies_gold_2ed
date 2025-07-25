use std::{collections::VecDeque, fs::File, io::{BufWriter, Read, Write}, ops::{Range, RangeBounds}, time::Instant};
use bevy::{app::AppExit, ecs::event::ManualEventReader, math::{DVec3, Vec3A}, prelude::*, render::{self, render_resource::ShaderType}, tasks::{ComputeTaskPool, ParallelSliceMut}, time::Stopwatch, utils::{petgraph::data, HashMap, HashSet}, window::WindowCloseRequested};
use fastrand::{Rng, choice};
use flate2::{bufread::{DeflateDecoder, GzDecoder}, write::{DeflateEncoder, GzEncoder, ZlibEncoder}, Compression};
use indexmap::{IndexMap, IndexSet};
use itertools::{iproduct, Itertools};
//use grid_tree::OctreeU32;
use noise::{core::worley::{distance_functions::{self, euclidean, euclidean_squared}, worley_3d, ReturnType}, permutationtable::PermutationTable, Blend, Constant, NoiseFn, Perlin, ScalePoint, Value, Worley};
//use rand::{seq::SliceRandom, thread_rng};
use derive_more::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign, };
use bevy::{ecs::{entity::{EntityMapper, MapEntities}, reflect::ReflectMapEntities}, prelude::*};
use rusqlite::{params, Connection};
use crate::{directions::{DIR_6, DIR_6_NO_DOWN}, grid3::Grid3, point::GridPoint, Item, ItemID, MoveToSpawn, RNGSeed, Slip, CHUNK_SIZE, WORLD_DEPTH, WORLD_HEIGHT, WORLD_SIZE};

use crate::sparse_grid3::SparseGrid3;


const SEA_LEVEL: f64 = -0.0;



//Plugin
#[derive(Default)]
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ChunkMap>()
            .init_resource::<PendingModificationMap>()
            .init_resource::<ChunkLoadingQueue>()
            .init_resource::<ChunkUnloadingQueue>()
            .init_resource::<ChunkSavingQueue>()
            .init_resource::<ChunkStatusMap>()
            .init_resource::<FunnyMapConsts>()
            .add_event::<BlockUpdateEvent>()
            .add_event::<LoadChunkEvent>()
            .add_event::<LoadReasonChangeEvent>()
            .add_event::<UpdateChunkEvent>()
            .add_event::<GenerateTreeEvent>();
    }
}
 

 // Systems
pub fn generate_chunks (
    mut commands: Commands,

    seed: Res<RNGSeed>,
    mut chunk_map: ResMut<ChunkMap>,
    mut pending_map: ResMut<PendingModificationMap>,

    mut evr_load_chunk: EventReader<LoadChunkEvent>,
    mut evw_gen_tree: EventWriter<GenerateTreeEvent>,
    mut evw_update_chunk: EventWriter<UpdateChunkEvent>,

    mut loader_query: Query<(&mut ChunkLoader)>,

    partial_save_map: Res<ChunkSavingQueue>,
    mut loading_queue: ResMut<ChunkLoadingQueue>,
    mut chunk_status_map: ResMut<ChunkStatusMap>,
    funny_map_consts: Res<FunnyMapConsts>,
    //time: Res<Time>,

    //mut next_mapgen_state: ResMut<NextState<MapGenState>>,
) {

    let start = Instant::now();

    let gradient = SingleDirectionAxialGradient { values: vec![1.0, 0.0, -0.5], points: vec![-(CHUNK_SIZE) as f64, 0.0, (WORLD_HEIGHT * CHUNK_SIZE) as f64], dimension: 1 };

    let noise_gen = Blend::new(ScalePoint::new(Perlin::new(**seed)).set_scale(0.025), gradient, Constant::new(0.7));

    //let tree_noise = Worley::new(**seed).set_distance_function(euclidean_squared).set_return_type(ReturnType::Distance).set_frequency(0.025 );

    let tree_noise = Blend::new(
        ScalePoint::new(Perlin::new(**seed + 1)).set_scale(0.001),
        WhiteNoise{seed: **seed},
        Constant::new(0.85),
    );

    //let mut chunks_to_load = Vec::new();

    //**loading_queue = VecDeque::from_iter(loading_queue.iter().filter_map(|ev| if !chunk_map.contains_key(&ev.chunk) {Some(*ev)} else {None}));

    for ev in evr_load_chunk.read() {
        // Ignore chunks that are out of generation scope.
        if !((-WORLD_SIZE[0]..=WORLD_SIZE[0]).contains(&ev.chunk.x) && (-WORLD_SIZE[1]..=WORLD_SIZE[1]).contains(&ev.chunk.z) && (-WORLD_DEPTH..=WORLD_HEIGHT).contains(&ev.chunk.y)) {
            continue
        }
        // Check to see if this chunk is already in the queue.
        if *chunk_status_map.get(&ev.chunk).unwrap_or(&ChunkStatus::NonLoaded) == ChunkStatus::Loading {
            continue
        }
        // Check if there's already a chunk here.
        if chunk_map.contains_key(&ev.chunk) {
            continue
        }

        // TODO: We should check loadreasons so that we don't end up loading chunks that are just going to get immediately unloaded.

        chunk_status_map.insert(ev.chunk, ChunkStatus::Loading);
        match ev.load_reason {
            LoadReason::Loader(_) => loading_queue.push_front(*ev),
            LoadReason::Spawning(_) => loading_queue.push_front(*ev),
        }
    }

    // Load only a limited amount of chunks each frame to make things smoother.
    let conn = Connection::open("saves/1.sl3").unwrap();
    conn.execute("pragma SYNCHRONOUS = NORMAL", []);
    let mut chunks_loaded = 0;
    while start.elapsed().as_millis() < 2 {
        let start_load_chunk = Instant::now();
        if let Some(ev) = loading_queue.pop_back() {
            //println!(":3");
            //println!("loading: {:?}", ev.chunk);
            let render_entity = Some(commands.spawn(Transform::from_translation((ev.chunk * CHUNK_SIZE).as_vec3()))
                                        .insert(GlobalTransform::default())
                                        .id());

            let water_render_entity = Some(commands.spawn(Transform::from_translation((ev.chunk * CHUNK_SIZE).as_vec3()))
                                        .insert(GlobalTransform::default())
                                        .id());

            let mut chunk = Chunk { blocks: Grid3::filled(Block::new(BlockID::Air), [CHUNK_SIZE, CHUNK_SIZE, CHUNK_SIZE]),
                                    load_reasons: HashSet::from([ev.load_reason]),
                                    render_entity,
                                    water_render_entity
                                  };
            
            //let start_chunkdata = Instant::now();
            let potential_compressed_chunk: Option<Vec<u8>> =
            if let Some(data) = partial_save_map.get(&ev.chunk) {
                Some(data.clone())
            }
            else if let Ok(data) = conn.query_one("SELECT ChunkData FROM Chunks WHERE PosX=?1 AND PosY=?2 AND PosZ=?3", [ev.chunk.x, ev.chunk.y, ev.chunk.z], |row| row.get(0)){
                Some(data)
            }
            else {
                None
            };
            //println!("time to get chunkdata: {:?}", start_chunkdata.elapsed());

            if let Some(compressed_chunk) = potential_compressed_chunk {
                
                let mut d = GzDecoder::new(&*compressed_chunk);
                let mut chunk_data: [u8; CHUNK_SIZE.pow(3) as usize * 2] = [0; CHUNK_SIZE.pow(3) as usize * 2];
                let result_bytes_read = d.read(&mut chunk_data);
                match result_bytes_read {
                    Ok(bytes_read) => {} //println!("bytes read: {}", bytes_read),
                    Err(err) => {} //println!("update failed: {}", err),
                }
                for (i, data) in chunk_data.chunks(2).enumerate() {
                    chunk.blocks.data[i].id = BlockID::from_u8(data[0]);
                    chunk.blocks.data[i].damage = data[1];
                }
            }
            else {
                
                                  
                let offset = ev.chunk * CHUNK_SIZE;
                let mut set_block_val = |position: IVec3, block_val: &mut Block| {
                    let point = DVec3::from(offset + position);
                
                    let noise_val = noise_gen.get([point.x, point.y, point.z]);
                    if noise_val >= 0.0 {
                        *block_val = Block::new(BlockID::Dirt);
                        // Make our block grass instead of dirt if the block above is air.
                        if noise_gen.get([point.x, point.y + 1.0, point.z]) < 0.0 && point.y > 0.0 {
                            *block_val = Block::new(BlockID::Grass);

                            // Tree!
                            if tree_noise.get([point.x, point.z]) > 0.80 {
                                evw_gen_tree.send(GenerateTreeEvent(IVec3::new(point.x as i32, point.y as i32 + 1, point.z as i32)));
                                // TODO: Maybe we want to do this in the tree generation system?
                                *block_val = Block::new(BlockID::Dirt);
                            }
                        }
                        if (noise_gen.get([point.x, point.y + 5.0, point.z]) > 0.0) || point.y < -5.0 {
                            *block_val = Block::new(BlockID::Stone);
                        }
                    }
                    if point.y < 0.0 && block_val.id == BlockID::Air {
                        *block_val = Block::new(BlockID::Water)
                    }
                };

                let mut all_air = true;
                let mut all_stone = true;
                for (i, position) in funny_map_consts.chunk_perimeter_indices.iter() {
                    set_block_val(*position, &mut chunk.blocks.data[*i]);
                    if chunk.blocks.data[*i].id != BlockID::Air {
                        all_air = false;
                    }
                    if chunk.blocks.data[*i].id != BlockID::Stone {
                        all_stone = false
                    }
                }
                /*
                for (position, block_val) in chunk.blocks.iter_3d_mut() {
                    // TODO: If this works, lets get some compile-time list we can use to iterate over. that would be much faster. (probably)
                    if position.max_element() == CHUNK_SIZE - 1 || position.min_element() == 0 {
                        set_block_val(position, block_val);
                        if block_val.id != BlockID::Air {
                            all_air = false;
                        }
                        if block_val.id != BlockID::Stone {
                            all_stone = false
                        }
                    }
                }
                */
                if !(all_air || all_stone) {
                    for (i, position) in funny_map_consts.chunk_volume_indices.iter() {
                        set_block_val(*position, &mut chunk.blocks.data[*i]);
                    }
                }

                if all_stone {
                    chunk.blocks = Grid3::filled(Block::new(BlockID::Stone), [CHUNK_SIZE, CHUNK_SIZE, CHUNK_SIZE]);
                }

                if let Some(pending_chunk) = pending_map.get_mut(&ev.chunk) {
                    for (position, modification) in pending_chunk.iter_3d_mut() {
                        if !modification.yield_to_terrain || chunk.blocks[position].id == BlockID::Air {
                            chunk.blocks[position] = modification.block;
                        }
                    }
                }
            }



            if let Some(chunk) = chunk_map.get(&ev.chunk) {
                if let Some(render_entity) = chunk.render_entity {
                    commands.entity(render_entity).despawn();
                }
                if let Some(water_render_entity) = chunk.water_render_entity {
                    commands.entity(water_render_entity).despawn();
                }
            }

            chunk_map.insert(ev.chunk, chunk);
            let loader_entity = match ev.load_reason {
                LoadReason::Loader(entity) => entity,
                LoadReason::Spawning(entity) => entity,
            };
            if let Ok(mut loader) = loader_query.get_mut(loader_entity) {
                loader.load_list.push(ev.chunk);
            }

            evw_update_chunk.send(UpdateChunkEvent(ev.chunk));
            for adj in ev.chunk.adj_6() {
                evw_update_chunk.send(UpdateChunkEvent(adj));
            }

            chunk_status_map.insert(ev.chunk, ChunkStatus::Active);
            chunks_loaded += 1;
        }
        else {
            break;
        }
        //println!("time to load chunk: {:?}", start_load_chunk.elapsed());
    }
    //println!("chunks loaded this frame: {}", chunks_loaded);
    //next_mapgen_state.set(MapGenState::TempBand);
}

pub fn generate_trees(
    seed: Res<RNGSeed>,
    mut chunk_map: ResMut<ChunkMap>,
    mut pending_map: ResMut<PendingModificationMap>,

    mut evr_gen_tree: EventReader<GenerateTreeEvent>,
    mut evw_update_chunk: EventWriter<UpdateChunkEvent>,
) {
    let mut chunk_updates = Vec::new();

    for ev in evr_gen_tree.read() {
        let mut local_seed = **seed as u64 + 1;
        let mut visited_positions = Vec::<IVec3>::new();
        let mut expansion_points = vec![**ev];
        let mut up_chance = 1.0;
        let mut up_done = false;
        let mut terminate_chance = -0.10;
        let mut branch_chance = 0.0;
        let mut branch_factor = 0.0;
        let mut last_direction = IVec3::new(0, 1, 0);

        while !expansion_points.is_empty() {
            let point = expansion_points[expansion_points.len() - 1];
            visited_positions.push(point);


            let mut block_placements = vec![(point, true)];

            if up_chance < 0.50 {
                let mut adj_points = point.adj_6().map(|p| (p, false)).collect_vec();
                block_placements.append(&mut adj_points);
            }

            for (placement_pos, is_log) in block_placements {
                let chunk_pos = chunk_pos_from_global(placement_pos);
                let block_pos = block_pos_from_global(placement_pos);
                //println!("---");


                if let Some(chunk) = chunk_map.get_mut(&chunk_pos) {

                    if chunk.blocks[block_pos].id == BlockID::Air || chunk.blocks[block_pos].id == BlockID::Leaves {
                        if is_log {
                            chunk.blocks[block_pos] = Block::new(BlockID::Log);
                        }
                        else {
                            chunk.blocks[block_pos] = Block::new(BlockID::Leaves);
                        }
                        continue;
                    }
                } 
                // else
                if !pending_map.contains_key(&chunk_pos) {
                    pending_map.insert(chunk_pos, Grid3::new([CHUNK_SIZE; 3]));
                }

                //println!("block pre modification: {:?}", pending_map[&chunk_pos][block_pos].block.id);
                if is_log {
                    pending_map.get_mut(&chunk_pos).unwrap()[block_pos] = PendingModification{ yield_to_terrain: true, block: Block::new(BlockID::Log) };
                }
                else if pending_map[&chunk_pos][block_pos].block.id == BlockID::Air {
                    pending_map.get_mut(&chunk_pos).unwrap()[block_pos] = PendingModification{ yield_to_terrain: true, block: Block::new(BlockID::Leaves) };
                }
                //println!("block post modification: {:?}", pending_map[&chunk_pos][block_pos].block.id);

                for event in update_chunk_events_from_global(placement_pos) {
                    if !chunk_updates.contains(&event) {
                        chunk_updates.push(event);
                    }
                }
            }
            
            
            
            //println!("local seed: {}", local_seed);
            local_seed = local_seed.wrapping_mul(point.x.abs() as u64 + 1).wrapping_mul(point.y.abs() as u64 + 1).wrapping_mul(point.z.abs() as u64 + 1);
            if local_seed == 0 { local_seed += 1};
            if Rng::with_seed(local_seed.wrapping_add(1)).f32() < up_chance {
                *expansion_points.last_mut().unwrap() = point.up(1);
                last_direction = IVec3::new(0, 1, 0);
            }
            else if Rng::with_seed(local_seed.wrapping_add(2)).f32() < branch_chance {
                //println!("branching!");
                expansion_points.push(point);
                terminate_chance = -0.10;
                branch_chance = 0.0;
                branch_factor += 0.02;
            }
            else if Rng::with_seed(local_seed.wrapping_add(3)).f32() < terminate_chance {
                expansion_points.pop();
                //last_direction = *Rng::with_seed(local_seed.wrapping_add(4)).choice(DIR_6_NO_DOWN).unwrap();
            }
            else {
                if Rng::with_seed(local_seed.wrapping_add(4)).f32() > 0.90 && !visited_positions.contains(&(point + last_direction)) {
                    *expansion_points.last_mut().unwrap() = point + last_direction;
                }
                else {
                    let choices = point.adj_6_no_down().filter(|p| !visited_positions.contains(p)).collect_vec();
                    if let Some(choice) = Rng::with_seed(local_seed.wrapping_add(4)).choice(choices) {
                        *expansion_points.last_mut().unwrap() = choice;
                        last_direction = point - choice;
                    }
                    else {
                        expansion_points.pop();
                    }
                }
            }
            //let direction = Rng::with_seed((**seed) as u64).choice(DIR_6);
            //for adj in point.adj_6() {

            //}
            up_chance -= 0.075;
            if up_chance < 0.50 && !up_done {
                up_done = true;
                up_chance = 0.0;
                for _ in 0..5 {
                    expansion_points.push(point);
                }
            }

            if up_done {
                branch_chance += 0.20 - branch_factor;
                terminate_chance += 0.10;
            }


            //println!("branch chance: {}", branch_chance);
            
        }

        for event in chunk_updates.iter() {
            evw_update_chunk.send(*event);
        }
    }
}

pub fn unload_chunks (
    mut commands: Commands,

    mut chunk_map: ResMut<ChunkMap>,

    mut evr_load_reason: EventReader<LoadReasonChangeEvent>,
    close_requested_evr: EventReader<WindowCloseRequested>,

    mut chunk_status_map: ResMut<ChunkStatusMap>,
    mut unloading_queue: ResMut<ChunkUnloadingQueue>,
    mut save_queue: ResMut<ChunkSavingQueue>,

) {
    let start = Instant::now();
    for (ev) in evr_load_reason.read() {
        if let Some(chunk) = chunk_map.get_mut(&**ev) {
            if chunk.load_reasons.is_empty() && *chunk_status_map.get(&**ev).unwrap_or(&ChunkStatus::NonLoaded) != ChunkStatus::Unloading {
                unloading_queue.push_front(**ev);
                chunk_status_map.insert(**ev, ChunkStatus::Unloading);
            }
        }
    }

    if !close_requested_evr.is_empty() {
        for (pos, _) in chunk_map.iter() {
            unloading_queue.push_front(*pos)
        }
    }
    
    let mut i = 0;
    let compressed_chunks: Vec<(Vec<u8>, IVec3)> = ComputeTaskPool::get().scope(|scope| {
        while i != 4 || !close_requested_evr.is_empty() {
            i += 1;
            if let Some(pos) = unloading_queue.pop_back() {
                if !chunk_map.contains_key(&pos) {
                    continue
                }
                if let Some(chunk) = chunk_map.get(&pos) {
                    // WARN: This effectively suppresses a B0003 warning by not attempting to despawn in situations that might cause that issue.
                    //       The warning may be indicative of a larger issue, so if we notice something strange happening with chunk loading/unloading, look here.
                    if close_requested_evr.is_empty() {
                        if let Some(render_entity) = chunk.render_entity {
                            commands.entity(render_entity).despawn();
                        }
                        if let Some(water_render_entity) = chunk.water_render_entity {
                            commands.entity(water_render_entity).despawn();
                        }
                    }
                    

                    scope.spawn(async move {
                        //let start_encode = Instant::now();
                        let mut e = GzEncoder::new(Vec::new(), Compression::fast());
                        for block in chunk.blocks.iter() {
                            e.write(&[block.id as u8, block.damage]).unwrap();
                        }
                        //println!("time to encode: {:?}", start_encode.elapsed());

                        (match e.finish() {
                            Ok(data) => data,
                            Err(_) => todo!(),
                        }, pos)
                    });
                    

                    
                    //println!("yeet... {:?}", pos);
                }
            }
            else {
                break;
            }
        }
        
    });

    for (chunk, pos) in compressed_chunks {
        save_queue.insert(pos, chunk);
        //println!("{}", chunk_map.remove(&pos).is_some());
        chunk_map.remove(&pos);
        chunk_status_map.insert(pos, ChunkStatus::PartialSave);
    }
    /*
    while start.elapsed().as_millis() < 1 || !close_requested_evr.is_empty() {
        if let Some(pos) = unloading_queue.pop_back() {
            if !chunk_map.contains_key(&pos) {
                continue
            }

            if let Some(chunk) = chunk_map.get_mut(&pos) {
                if let Some(render_entity) = chunk.render_entity {
                    commands.entity(render_entity).despawn();
                }
                if let Some(water_render_entity) = chunk.water_render_entity {
                    commands.entity(water_render_entity).despawn();
                }

                let start_encode = Instant::now();
                let mut e = GzEncoder::new(Vec::new(), Compression::fast());
                for block in chunk_map.get(&pos).unwrap().blocks.iter() {
                    e.write(&[block.id as u8, block.damage]).unwrap();
                }
                println!("time to encode: {:?}", start_encode.elapsed());

                save_queue.insert(pos, e.finish().unwrap());

                chunk_map.remove(&pos);
                chunk_status_map.insert(pos, ChunkStatus::PartialSave);
                //println!("yeet... {:?}", pos);
            }
        }
        else {
            break;
        }
    } */
}

pub fn save_chunks (
    mut save_queue: ResMut<ChunkSavingQueue>,
    close_requested_evr: EventReader<WindowCloseRequested>,
    mut exit_evw: EventWriter<AppExit>
) {

    if !close_requested_evr.is_empty() { //save_queue.len() > 10 || 
        let start = Instant::now();
        let conn = Connection::open("saves/1.sl3").unwrap();
        conn.execute("begin", []);
        conn.execute("pragma SYNCHRONOUS = NORMAL", []);
        let mut stmt = conn.prepare("INSERT INTO Chunks (PosX, PosY, PosZ, ChunkData) VALUES (?1, ?2, ?3, ?4)
                                    ON CONFLICT(PosX, PosY, PosZ) DO UPDATE SET ChunkData=excluded.ChunkData;").unwrap();

        for (pos, chunk_data) in save_queue.iter() {
            match stmt.execute(params![pos.x, pos.y, pos.z, chunk_data]) {
                Ok(updated) => {}, //println!("{} rows were updated", updated),
                Err(err) => {}, //println!("update failed: {}", err),
            }
        } 

        conn.execute("end", []);
        save_queue.clear();
        let duration = start.elapsed();
        println!("Time elapsed for upserts: {:?}", duration);
        if !close_requested_evr.is_empty() {
            exit_evw.send(AppExit);
        }
    }
}

pub fn process_block_updates (
    time: Res<Time>,
    mut chunk_map: ResMut<ChunkMap>,
    mut block_update_events: ResMut<Events<BlockUpdateEvent>>,

    mut evw_update_chunk: EventWriter<UpdateChunkEvent>,
    mut mevr_block_update: Local<ManualEventReader<BlockUpdateEvent>>,

    
) {
    let mut thirsty_blocks = Vec::<IVec3>::new();
    let mut requeue_queue = Vec::<BlockUpdateEvent>::new();

    for ev in mevr_block_update.read(&block_update_events) {
        let chunk_pos = chunk_pos_from_global(ev.position);

        if (ev.time_waited.elapsed() + time.delta()).as_millis() < 200 {
            let mut requeue_ev = ev.clone();
            requeue_ev.time_waited.tick(time.delta());
            requeue_queue.push(requeue_ev);
            continue
        }

        // Can't borrow both mutably here and immutably later. Guess we gotta make a vec to process through after we look at everything?
        if let Some(chunk) = chunk_map.get(&chunk_pos) {
            let block_pos = block_pos_from_global(ev.position);

            if chunk.blocks[block_pos].id == BlockID::Air {
                for adj in ev.position.adj_6() {
                    if ev.position.y - adj.y == 1 {
                        continue
                    }
                    let chunk_pos_adj = chunk_pos_from_global(adj);
                    if let Some(chunk_adj) = chunk_map.get(&chunk_pos_adj) {
                        let block_pos_adj = block_pos_from_global(adj);
                        if chunk_adj.blocks[block_pos_adj].id == BlockID::Water {
                            thirsty_blocks.push(ev.position);
                            break
                        }
                    }
                }
            }
        }
    }

    for ev in requeue_queue {
        block_update_events.send(ev);
    }

    for position in thirsty_blocks {
        let chunk_pos = chunk_pos_from_global(position);

        if let Some(chunk) = chunk_map.get_mut(&chunk_pos) {
            let block_pos = block_pos_from_global(position);
            chunk.blocks[block_pos] = Block::new(BlockID::Water);

            for adj in position.adj_6() {
                block_update_events.send(BlockUpdateEvent { position: adj, time_waited: Stopwatch::new() });
            }

            for event in update_chunk_events_from_global(position) {
                evw_update_chunk.send(event);
            } 
        }
    }

}

pub fn update_chunk_positions (
    mut query: Query<(&GlobalTransform, &mut ChunkPosition), (Changed<GlobalTransform>)>,
) {
    for (transform, mut position) in &mut query {
        let new_position = (transform.translation() / CHUNK_SIZE as f32).as_ivec3();
        // Avoid updating Changed except for when we're actually changing the value.
        if **position != new_position {
            //println!("chunk position: {}", new_position);

            **position = new_position;
        }
    }
}

pub fn update_chunk_loaders (
    mut query: Query<(Entity, &ChunkPosition, &mut ChunkLoader), (Changed<ChunkPosition>, Without<MoveToSpawn>)>,

    mut chunk_map: ResMut<ChunkMap>,
    mut chunk_status_map: ResMut<ChunkStatusMap>,

    mut commands: Commands,

    mut evw_load_chunk: EventWriter<LoadChunkEvent>,
    mut evw_load_reason: EventWriter<LoadReasonChangeEvent>,
) {
    // TODO: Not sure if "buffer" is the right word. Also, maybe this should be an attribute of the ChunkLoader type? Or maybe a const?
    let buffer_range = 1;

    for (entity, position, mut loader) in &mut query {
        // Reset whatever it is we're currently loading.
        // TODO: We should be selectively removing things, maybe. and then we can use range + n for the area where we wont load/generate chunks, but we'll still keep chunks already loaded/generated loaded.
        for chunk_pos in loader.load_list.iter() {
            if let Some(chunk) = chunk_map.get_mut(chunk_pos) {
                chunk.load_reasons.remove(&LoadReason::Loader(entity));
                evw_load_reason.send(LoadReasonChangeEvent(*chunk_pos));
            }
        }
        loader.load_list = vec![];

        // Load everything in our range.
        let min_corner = **position - loader.range;
        let max_corner = **position + loader.range;
        let min_corner_buffered = min_corner - buffer_range;
        let max_corner_buffered = max_corner + buffer_range;
        //let y = 0;

        let mut load_range = iproduct!(min_corner_buffered.x..=position.x,
                                         min_corner_buffered.y..=max_corner_buffered.y,
                                         min_corner_buffered.z..=max_corner_buffered.z).map(|(x, y, z)| IVec3::new(x, y, z)).collect_vec();

        load_range.reverse();

        let mut load_range_end = iproduct!((position.x + 1)..=max_corner_buffered.x,
                                       min_corner_buffered.y..=max_corner_buffered.y,
                                       min_corner_buffered.z..=max_corner_buffered.z).map(|(x, y, z)| IVec3::new(x, y, z)).collect_vec();

        load_range.append(&mut load_range_end);

        for p in load_range.iter() {
                                    
            let mut load_success = false;

                if let Some(chunk) = chunk_map.get_mut(p) {
                    // Try to remove this. Just in case.
                    chunk.load_reasons.remove(&LoadReason::Spawning(entity));

                    chunk.load_reasons.insert(LoadReason::Loader(entity));

                    loader.load_list.push(*p);

                    load_success = true;
                    //chunk_status_map.insert(*p, ChunkStatus::Loaded);
                }
            // We keep everything in the range and the buffer range loaded, but we only *start* loading if chunks are in the load range proper.
            if !load_success && (min_corner.x..=max_corner.x).contains(&p.x) && (min_corner.y..=max_corner.y).contains(&p.y) && (min_corner.z..=max_corner.z).contains(&p.z) {
                evw_load_chunk.send(LoadChunkEvent { chunk: *p, load_reason: LoadReason::Loader(entity) });
                //chunk_status_map.insert(*p, ChunkStatus::Loading);
            }
        }
    }
}

#[derive(Clone, Resource)]
pub struct FunnyMapConsts {
    chunk_perimeter_indices: Vec<(usize, IVec3)>, //[(usize, IVec3); (CHUNK_SIZE.pow(3) - (CHUNK_SIZE-2).pow(3)) as usize],
    chunk_volume_indices: Vec<(usize, IVec3)>, //[(usize, IVec3); (CHUNK_SIZE-2).pow(3) as usize],
}
impl Default for FunnyMapConsts {
    fn default() -> Self {
        Self { chunk_perimeter_indices: Grid3::<u8>::new([CHUNK_SIZE, CHUNK_SIZE, CHUNK_SIZE]).iter_3d().enumerate().filter_map(|(i, (pos, _))| {
                if pos.max_element() == CHUNK_SIZE - 1 || pos.min_element() == 0 {
                    Some((i, pos))
                }
                else {
                    None
                }
            }).collect::<Vec<(usize, IVec3)>>(),
            chunk_volume_indices: Grid3::<u8>::new([CHUNK_SIZE, CHUNK_SIZE, CHUNK_SIZE]).iter_3d().enumerate().filter_map(|(i, (pos, _))| {
                if pos.max_element() != CHUNK_SIZE - 1 || pos.min_element() != 0 {
                    Some((i, pos))
                }
                else {
                    None
                }
            }).collect::<Vec<(usize, IVec3)>>(),
         }
    }
}

#[derive(Default, Clone, Deref, DerefMut, Resource)]
pub struct ChunkLoadingQueue(VecDeque<LoadChunkEvent>);

#[derive(Default, Clone, Deref, DerefMut, Resource)]
pub struct ChunkUnloadingQueue(VecDeque<IVec3>);

#[derive(Default, Clone, Deref, DerefMut, Resource)]
pub struct ChunkSavingQueue(IndexMap<IVec3, Vec<u8>>);

#[derive(Clone, Event)]
pub struct BlockUpdateEvent {
    pub position: IVec3,
    pub time_waited: Stopwatch
}

#[derive(Clone, Copy, Event)]
pub struct LoadChunkEvent {
    pub chunk: IVec3,
    pub load_reason: LoadReason,
}

#[derive(Clone, Copy, Event, Deref, DerefMut, PartialEq, Eq)]
pub struct LoadReasonChangeEvent(IVec3);

#[derive(Default, Clone, Deref, DerefMut, Resource)]
pub struct ChunkStatusMap(HashMap<IVec3, ChunkStatus>);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum ChunkStatus {
    NonLoaded,
    Loading,
    // Change from Loaded to Active. Active meaning that the chunk is in memory and able to be interacted with and Loaded meaning that the chunk has load reasons
    // We already have a system for managing load reasons. So we don't need to track that. Also, we don't need to track if a chunk is active either, but it seems incomplete to not include it? Maybe?
    Active, //Loaded,
    Unloading,
    PartialSave,
}

//#[derive(Clone, Copy, Event)]
//pub struct PendingModificationEvent {
//    pub chunk: IVec3,
//}

#[derive(Clone, Copy, Event, Deref, DerefMut, PartialEq, Eq)]
pub struct UpdateChunkEvent(IVec3);


#[derive(Clone, Copy, Event, Deref, DerefMut)]
pub struct GenerateTreeEvent(IVec3);
#[derive(Default, Clone, Deref, DerefMut, Resource)]
pub struct ChunkMap(HashMap<IVec3, Chunk>);

#[derive(Clone)]
pub struct PendingModification {
    /// True if terrain generation takes priority over our modification.
    yield_to_terrain: bool,
    block: Block,
}
impl Default for PendingModification {
    fn default() -> Self {
        Self { yield_to_terrain: true, block: Default::default() }
    }
}

#[derive(Default, Clone, Deref, DerefMut, Resource)]
pub struct PendingModificationMap(HashMap<IVec3, Grid3<PendingModification>>);

/// Denotes that an entity loads chunks around itself.
#[derive(Default, Clone, Component)]
pub struct ChunkLoader{
    pub range: i32,
    pub load_list: Vec<IVec3>,
}

/// Required for chunkloading entities. May have other purposes later.
#[derive(Default, Clone, Deref, DerefMut, Component)]
pub struct ChunkPosition(IVec3);

//#[derive(Default, Clone, Deref, DerefMut, Component, Debug)]
//pub struct LoadReasonList(HashSet<LoadReason>);

#[derive(Copy, Clone, Eq, Hash, PartialEq, Debug)]
pub enum LoadReason {
    Loader(Entity),
    Spawning(Entity), // TODO: Refactor to "move"? or "teleport"? not sure if we should
}

// CRITICAL!
// TODO: If we separate load reasons out from here into its own hashmap, then we can ignore entries in the load queue when they have no loadreasons (so that we dont end up loading chunks that are just going to get immediately unloaded)
#[derive(Clone, Debug)]
pub struct Chunk {
    pub blocks: Grid3<Block>,
    pub load_reasons: HashSet<LoadReason>,
    pub render_entity: Option<Entity>,
    pub water_render_entity: Option<Entity>,
}


// TODO: Optimization: If we're using too much space, we can try and use u8s instead of enums. :)
#[derive(Default, Clone, Copy, Debug)]
pub struct Block {
    pub id: BlockID,
    pub damage: u8,
    //pub data: [BlockData; 1],
}
impl Block {
    pub fn new(id: BlockID) -> Block {
        // TODO: Make the BlockData thing be tailored for the block we're making.
        Block {id, damage: 0, }//data: [BlockData::None]}
    }

    pub fn get_attributes(self) -> BlockAttributes {
        self.id.get_attributes()
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Reflect)]
pub enum BlockID {
    #[default] Air,
    Dirt,
    Grass,
    Stone,
    StoneBrick,
    Log,
    Leaves,
    Water,
}
impl BlockID {
    pub fn get_attributes(self) -> BlockAttributes {
        match self {
            BlockID::Air => BlockAttributes { health: 0, solidity: Solidity::NonSolid, ..default()  },
            BlockID::Dirt => BlockAttributes { health: 3, tex_coords: TextureCoords::symmetrical(IVec2::new(0, 0)), ..default() },
            BlockID::Grass => BlockAttributes { health: 1, tex_coords: TextureCoords::asymmetric_y(IVec2::new(0, 1), IVec2::new(0, 0), IVec2::new(1, 1)), breaks_into: BlockID::Dirt, ..default() },
            BlockID::Stone => BlockAttributes { health: 5, tex_coords: TextureCoords::symmetrical(IVec2::new(0, 2)), give_on_damage: Some(Item{ id: ItemID::Stone, amount: 16, }), ..default() },
            BlockID::StoneBrick => BlockAttributes { health: 5, tex_coords: TextureCoords::symmetrical(IVec2::new(0, 3)), give_on_damage: Some(Item{ id: ItemID::Stone, amount: 2 }), cost_to_build: [Some(Item::new(ItemID::Stone, 16)), None, None],  ..default() },
            // Logs will have special behavior for how they get mined, most likely. (Treefelling)
            BlockID::Log => BlockAttributes { health: 2, tex_coords: TextureCoords::symmetrical(IVec2::new(0, 4)), give_on_damage: Some(Item{id: ItemID::Wood, amount: 32}), ..default() },
            BlockID::Leaves => BlockAttributes { health: 1, tex_coords: TextureCoords::symmetrical(IVec2::new(0, 5)), solidity: Solidity::NonSolid, ..default() },
            BlockID::Water => BlockAttributes {health: 0, tex_coords: TextureCoords::unique_top(IVec2::new(0, 7), IVec2::new(1, 7)), solidity: Solidity::Water, ..default()}
            
        }
    }

    fn get_default_data(self) -> [BlockData; 1] {
        todo!()
    }

    pub fn from_u8(num: u8) -> Self {
        match num {
            0 => BlockID::Air,
            1 => BlockID::Dirt,
            2 => BlockID::Grass,
            3 => BlockID::Stone,
            4 => BlockID::StoneBrick,
            5 => BlockID::Log,
            6 => BlockID::Leaves,
            7 => BlockID::Water,
            _ => todo!("Requested unassigned blockID!"),
        }
    }
}

#[derive(Default, Clone, Copy)]
pub enum BlockData {
    #[default] None,
    // For trees and stuff.
    DamagedAdjacent(u8),
}

#[derive(Default, Clone, Copy)]
pub struct BlockAttributes {
    pub health: u8,
    pub toughness: u8,
    pub tex_coords: TextureCoords,
    pub breaks_into: BlockID,
    pub give_on_damage: Option<Item>,
    pub cost_to_build: [Option<Item>; 3],
    pub solidity: Solidity,
    pub slip: Slip,
}
/*
impl BlockAttributes {
    pub fn new(health: u8, toughness: u8, tex_coords: IVec2) -> BlockAttributes {
        BlockAttributes { health, toughness, tex_coords }
    }
}
 */

 #[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
 pub enum Solidity {
    #[default] Solid,
    NonSolid,
    Water,
 }

// We might as well make this a struct instead of an enum, since it'll be the same size either way, and this will let us clarify what is what better.
#[derive(Default, Clone, Copy)]
pub struct TextureCoords {
    pub top: IVec2,
    pub bottom: IVec2,
    pub north: IVec2,
    pub south: IVec2,
    pub east: IVec2,
    pub west: IVec2,
}
impl TextureCoords {
    pub fn symmetrical(coord: IVec2) -> TextureCoords {
        TextureCoords { top: coord, bottom: coord, north: coord, south: coord, east: coord, west: coord }
    }
    pub fn asymmetric_y(top: IVec2, bottom: IVec2, sides: IVec2) -> TextureCoords {
        TextureCoords { top, bottom, north: sides, south: sides, east: sides, west: sides }
    }
    pub fn unique_top(top: IVec2, sides: IVec2) -> TextureCoords {
        TextureCoords { top, bottom: sides, north: sides, south: sides, east: sides, west: sides }
    }
}

#[derive(Default, Clone)]
pub struct SingleDirectionAxialGradient {
    pub values: Vec<f64>,
    pub points: Vec<f64>,
    pub dimension: usize,
}
impl SingleDirectionAxialGradient {

}
impl<const N: usize> NoiseFn<f64, N> for SingleDirectionAxialGradient {
    fn get(&self, point: [f64; N]) -> f64 {
        if point[self.dimension] < self.points[0] {
            return self.values[0];
        }

        for (i, _) in self.values.iter().enumerate() {
            if i + 1 == self.values.len() {
                return *self.values.last().unwrap();
            }

            if (self.points[i]..=self.points[i+1]).contains(&point[self.dimension]) {
                let a = (self.values[i+1] - self.values[i]) / (self.points[i+1] - self.points[i]);
                let b = self.values[i] - (a * self.points[i]);
                return a * point[self.dimension] + b;
            }
        }

        return 0.0;
    }
}

#[derive(Default, Clone)]
pub struct WhiteNoise {
    pub seed: u32,
}
impl<const N: usize> NoiseFn<f64, N> for WhiteNoise {
    fn get(&self, point: [f64; N]) -> f64 {
        let mut point = point.to_vec();
        // TODO: Is cantor pairing overkill for this? IDK. I'm also not sure if I'm properly preserving uniqueness.
        let mut cantor_pairing = point.pop().unwrap().to_bits();
        for n in point {
            let n = n.to_bits();
            cantor_pairing = (cantor_pairing.wrapping_add(n).wrapping_mul(cantor_pairing.wrapping_add(n).wrapping_add(1)) / 2).wrapping_add(n);
        }
        return Rng::with_seed((self.seed as u64).wrapping_mul(cantor_pairing)).f64() * 2.0 - 1.0;
    }
}

//Helpers
pub fn chunk_pos_from_global (global_position: IVec3) -> IVec3 {
    let mut modified_position = global_position;
        
    // TODO: This doesn't feel very elegant. Perhaps we could get a more mathy solution somehow? Would be nice.
    if global_position.x < 0 {modified_position.x = global_position.x - 15};
    if global_position.y < 0 {modified_position.y = global_position.y - 15};
    if global_position.z < 0 {modified_position.z = global_position.z - 15};

    modified_position / CHUNK_SIZE
}

pub fn block_pos_from_global (global_position: IVec3) -> IVec3 {
    let mut block_pos = global_position % CHUNK_SIZE;

    if block_pos.x < 0 {block_pos.x += CHUNK_SIZE};
    if block_pos.y < 0 {block_pos.y += CHUNK_SIZE};
    if block_pos.z < 0 {block_pos.z += CHUNK_SIZE};

    block_pos
}

pub fn update_chunk_events_from_global (global_position: IVec3) -> Vec<UpdateChunkEvent> {
    let chunk_position = chunk_pos_from_global(global_position);
    let block_position = block_pos_from_global(global_position);

    let mut events = vec![UpdateChunkEvent(chunk_position)];

    for i in 0..=2 {
        if block_position[i] == 0 {
            let mut adjacent_chunk_position = chunk_position;
            adjacent_chunk_position[i] -= 1;
            events.push(UpdateChunkEvent(adjacent_chunk_position))
        }
        else if block_position[i] == CHUNK_SIZE - 1 {
            let mut adjacent_chunk_position = chunk_position;
            adjacent_chunk_position[i] += 1;
            events.push(UpdateChunkEvent(adjacent_chunk_position))
        }
    }

    events
}











// Unused.

/// This struct is overkill, but pretty cool.
#[derive(Default, Clone, Copy)]
pub struct AxialGradient {
    pub val_1: f64,
    pub val_2: f64,
    pub point_1: [f64; 4],
    pub point_2: [f64; 4],
}
impl AxialGradient {
    // TODO: We could potentially precalculate full from points_range.
    fn get_from_distances(&self, full: f64, partial: f64) -> f64 {
        // Normalizing with:
        // a = (max'-min')/(max-min)
        // b = min' - (a * min)
        // newvalue = a * value + b
        // from: https://stats.stackexchange.com/questions/70801/how-to-normalize-data-to-0-1-range#comment137312_70808
        // Making min be 0 simplifies our equations.
        let a = (self.val_2 - self.val_1)/(full);
        let b = self.val_1;
        a * partial + b
    }

    fn get_from_sidelengths(&self, full: f64, to_min: f64, to_max: f64) -> f64 {
        // Heron's formula
        let s = (full + to_min + to_max) / 2.0;
        let area = (s * (s - full) * (s - to_min) * (s - to_max)).sqrt();

        let height = 2.0 * (area / full);

        let partial = (to_min.powi(2) - height.powi(2)).sqrt();

        self.get_from_distances(full, partial)
    }
}

/// 1-dimensional gradient
impl NoiseFn<f64, 1> for AxialGradient {
    fn get(&self, point: [f64; 1]) -> f64 {
        self.get_from_distances(self.point_2[0] - self.point_1[0], point[0] - self.point_1[0])
    }
}


/// 2-dimensional gradient
impl NoiseFn<f64, 2> for AxialGradient {
    fn get(&self, point: [f64; 2]) -> f64 {
        let full = ((self.point_2[0] - self.point_1[0]).powi(2) + (self.point_2[1] - self.point_1[1]).powi(2)).sqrt();
        let to_min = ((point[0] - self.point_1[0]).powi(2) + (point[1] - self.point_1[1]).powi(2)).sqrt();
        let to_max = ((point[0] - self.point_2[0]).powi(2) + (point[1] - self.point_2[1]).powi(2)).sqrt();
        self.get_from_sidelengths(full, to_min, to_max)
    }
}

/// 3-dimensional gradient
impl NoiseFn<f64, 3> for AxialGradient {
    fn get(&self, point: [f64; 3]) -> f64 {
        let full = ((self.point_2[0] - self.point_1[0]).powi(2) + (self.point_2[1] - self.point_1[1]).powi(2) + (self.point_2[2] - self.point_1[2]).powi(2)).sqrt();
        let to_min = ((point[0] - self.point_1[0]).powi(2) + (point[1] - self.point_1[1]).powi(2) + (point[2] - self.point_1[2]).powi(2)).sqrt();
        let to_max = ((point[0] - self.point_2[0]).powi(2) + (point[1] - self.point_2[1]).powi(2) + (point[2] - self.point_2[2]).powi(2)).sqrt();
        self.get_from_sidelengths(full, to_min, to_max)
    }
}

/// 4-dimensional gradient
impl NoiseFn<f64, 4> for AxialGradient {
    fn get(&self, point: [f64; 4]) -> f64 {
        let full = ((self.point_2[0] - self.point_1[0]).powi(2) + (self.point_2[1] - self.point_1[1]).powi(2) + (self.point_2[2] - self.point_1[2]).powi(2) + (self.point_2[3] - self.point_1[3]).powi(2)).sqrt();
        let to_min = ((point[0] - self.point_1[0]).powi(2) + (point[1] - self.point_1[1]).powi(2) + (point[2] - self.point_1[2]).powi(2) + (point[3] - self.point_1[3]).powi(2)).sqrt();
        let to_max = ((point[0] - self.point_2[0]).powi(2) + (point[1] - self.point_2[1]).powi(2) + (point[2] - self.point_2[2]).powi(2) + (point[3] - self.point_2[3]).powi(2)).sqrt();
        self.get_from_sidelengths(full, to_min, to_max)
    }
}