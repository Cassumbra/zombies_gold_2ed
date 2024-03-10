use std::{collections::VecDeque, ops::{Range, RangeBounds}};
use bevy::{math::Vec3A, prelude::*, utils::{HashMap, HashSet}};
use fastrand::{Rng, choice};
use sark_grids::Grid;
//use grid_tree::OctreeU32;
use noise::{core::worley::distance_functions::euclidean_squared, Blend, Constant, NoiseFn, Perlin, ScalePoint, Worley};
//use rand::{seq::SliceRandom, thread_rng};
use derive_more::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign, };
use bevy::{ecs::{entity::{EntityMapper, MapEntities}, reflect::ReflectMapEntities}, prelude::*};
use crate::{grid3::Grid3, RNGSeed, CHUNK_SIZE, WORLD_DEPTH, WORLD_HEIGHT, WORLD_SIZE};

use crate::sparse_grid3::SparseGrid3;


const SEA_LEVEL: f64 = -0.0;


//Plugin
#[derive(Default)]
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ChunkMap>()
            .init_resource::<ChunkLoadingQueue>()
            .add_event::<LoadChunkEvent>();
    }
}
 

 // Systems
pub fn generate_chunks (
    mut commands: Commands,

    seed: Res<RNGSeed>,
    mut chunk_map: ResMut<ChunkMap>,

    mut evr_load_chunk: EventReader<LoadChunkEvent>,

    mut loader_query: Query<(&mut ChunkLoader)>,

    mut loading_queue: ResMut<ChunkLoadingQueue>,

    //mut next_mapgen_state: ResMut<NextState<MapGenState>>,
) {
    let perlin_noise3d = Perlin::new(**seed);
    let scaled_perlin = ScalePoint::new(perlin_noise3d).set_scale(0.025); //.set_all_scales(0.025, 0.025, 0.025, 1.0);
    //let gradient = AxialGradient{ val_1: 1.0, val_2: -1.0, point_1: [0.0, 0.0, 0.0, 0.0], point_2: [0.0, (WORLD_HEIGHT * CHUNK_SIZE) as f64, 0.0, 0.0] };
    let gradient = SingleDirectionAxialGradient { values: vec![1.0, 0.0, -1.0], points: vec![-(WORLD_DEPTH * CHUNK_SIZE) as f64, 0.0, (WORLD_HEIGHT * CHUNK_SIZE) as f64], dimension: 1 };
    let noise_gen = Blend::new(scaled_perlin, gradient, Constant::new(0.7));

    let mut chunks_to_load = Vec::new();

    for ev in evr_load_chunk.read() {
        // Ignore chunks that are out of generation scope.
        if !((-WORLD_SIZE[0]..=WORLD_SIZE[0]).contains(&ev.chunk.x) && (-WORLD_SIZE[1]..=WORLD_SIZE[1]).contains(&ev.chunk.z) && (-WORLD_DEPTH..=WORLD_HEIGHT).contains(&ev.chunk.y)) {
            continue
        }

        // Check if there's already an entity here.
        if let Some(chunk_entity) = chunk_map.get(&ev.chunk) {
            // Check to see if the entity is valid.
            if commands.get_entity(*chunk_entity).is_some() {
                continue
            }
        }

        match ev.load_reason {
            LoadReason::Loader(_) => loading_queue.push_front(*ev),
            LoadReason::Spawning(_) => chunks_to_load.push(*ev),
        }
    }

    // Load only a limited amount of chunks each frame to make things smoother.
    for _ in 0..4 {
        if let Some(ev) = loading_queue.pop_back() {
            chunks_to_load.push(ev);
        }
    }
    

    for ev in chunks_to_load.iter() {
        //println!("loading: {:?}", ev.chunk);

        let mut chunk = Chunk(Grid3::filled(Block::new(BlockID::Air), [CHUNK_SIZE, CHUNK_SIZE, CHUNK_SIZE]));
    
        let offset = ev.chunk * CHUNK_SIZE;

        // Set initial values
        for (position, block_val) in chunk.iter_3d_mut() {
            let point_x = (offset.x + position.x) as f64;
            let point_y = (offset.y + position.y) as f64;
            let point_z = (offset.z + position.z) as f64;
    
            let noise_val = noise_gen.get([point_x, point_y, point_z]);
            if noise_val >= 0.0 {
                *block_val = Block::new(BlockID::Dirt);
                // Make our block grass instead of dirt if the block above is air.
                if noise_gen.get([point_x, point_y + 1.0, point_z]) < 0.0 {
                    *block_val = Block::new(BlockID::Grass);
                }
            }
        }
        
        let chunk_entity = commands.spawn(chunk)
            .insert(LoadReasonList(HashSet::from([ev.load_reason])))
            .insert(Transform::from_translation((ev.chunk * CHUNK_SIZE).as_vec3()))
            .insert(GlobalTransform::default())
            .id();
        chunk_map.insert(ev.chunk, chunk_entity);
        let loader_entity = match ev.load_reason {
            LoadReason::Loader(entity) => entity,
            LoadReason::Spawning(entity) => entity,
        };
        if let Ok(mut loader) = loader_query.get_mut(loader_entity) {
            loader.load_list.push(chunk_entity);
        }
    }

    //next_mapgen_state.set(MapGenState::TempBand);
}

pub fn unload_chunks (
    mut commands: Commands,

    //mut chunk_map: ResMut<ChunkMap>,

    chunk_query: Query<(Entity, &LoadReasonList), Changed<LoadReasonList>>,
) {
    for (chunk_entity, load_reason_list) in &chunk_query {
        //println!("load reasons: {:?}", **load_reason_list);
        if load_reason_list.is_empty() {
            commands.entity(chunk_entity).despawn();
            // TODO: The fact that we don't remove the entity from the chunk_map means we have to test to see if the entity in the map is actually valid in a lot of different places.
            //       This kinda sucks, and we should probably fix it!
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
    mut query: Query<(Entity, &ChunkPosition, &mut ChunkLoader), (Changed<ChunkPosition>)>,
    mut chunk_query: Query<(&mut LoadReasonList)>,

    chunk_map: Res<ChunkMap>,

    //mut commands: Commands,

    mut evw_load_chunk: EventWriter<LoadChunkEvent>,
) {
    // TODO: Not sure if "buffer" is the right word. Also, maybe this should be an attribute of the ChunkLoader type? Or maybe a const?
    let buffer_range = 1;

    for (entity, position, mut loader) in &mut query {
        // Reset whatever it is we're currently loading.
        // TODO: We should be selectively removing things, maybe. and then we can use range + n for the area where we wont load/generate chunks, but we'll still keep chunks already loaded/generated loaded.
        for chunk_entity in loader.load_list.iter() {
            if let Ok(mut load_reason_list) = chunk_query.get_mut(*chunk_entity) {
                load_reason_list.remove(&LoadReason::Loader(entity));
            }
        }
        loader.load_list = vec![];

        // Load everything in our range.
        let min_corner = **position - loader.range;
        let max_corner = **position + loader.range;
        let min_corner_buffered = min_corner - buffer_range;
        let max_corner_buffered = max_corner + buffer_range;
        //let y = 0;
        for x in min_corner_buffered.x..=max_corner_buffered.x {
            for y in min_corner_buffered.y..=max_corner_buffered.y {
                for z in min_corner_buffered.z..=max_corner_buffered.z {
                    let mut load_success = false;

                    if let Some(chunk_entity) = chunk_map.get(&IVec3::new(x, y, z)) {
                        if let Ok(mut load_reason_list) = chunk_query.get_mut(*chunk_entity) {
                            // Try to remove this. Just in case.
                            load_reason_list.remove(&LoadReason::Spawning(entity));

                            load_reason_list.insert(LoadReason::Loader(entity));

                            loader.load_list.push(*chunk_entity);

                            load_success = true;
                        }
                    }
                    // We keep everything in the range and the buffer range loaded, but we only *start* loading if chunks are in the load range proper.
                    if !load_success && (min_corner.x..=max_corner.x).contains(&x) && (min_corner.y..=max_corner.y).contains(&y) && (min_corner.z..=max_corner.z).contains(&z) {
                        evw_load_chunk.send(LoadChunkEvent { chunk: IVec3::new(x, y, z), load_reason: LoadReason::Loader(entity) });
                    }
                }
            }
        }

    }
}

#[derive(Default, Clone, Deref, DerefMut, Resource)]
pub struct ChunkLoadingQueue(VecDeque<LoadChunkEvent>);

#[derive(Clone, Copy, Event)]
pub struct LoadChunkEvent {
    pub chunk: IVec3,
    pub load_reason: LoadReason,
}

#[derive(Default, Clone, Deref, DerefMut, Resource)]
pub struct ChunkMap(HashMap<IVec3, Entity>);


/// Denotes that an entity loads chunks around itself.
#[derive(Default, Clone, Component)]
pub struct ChunkLoader{
    pub range: i32,
    pub load_list: Vec<Entity>,
}

/// Required for chunkloading entities. May have other purposes later.
#[derive(Default, Clone, Deref, DerefMut, Component)]
pub struct ChunkPosition(IVec3);

#[derive(Default, Clone, Deref, DerefMut, Component, Debug)]
pub struct LoadReasonList(HashSet<LoadReason>);

#[derive(Copy, Clone, Eq, Hash, PartialEq, Debug)]
pub enum LoadReason {
    Loader(Entity),
    Spawning(Entity), // TODO: Refactor to "move"? or "teleport"? not sure if we should
}

#[derive(Default, Clone, Deref, DerefMut, Component)]
pub struct Chunk(Grid3<Block>);


// TODO: Optimization: If we're using too much space, we can try and use u8s instead of enums. :)
#[derive(Default, Clone, Copy)]
pub struct Block {
    pub block_id: BlockID,
    pub damage: u8,
    pub data: [BlockData; 1],
}
impl Block {
    pub fn new(block_id: BlockID) -> Block {
        // TODO: Make the BlockData thing be tailored for the block we're making.
        Block {block_id, damage: 0, data: [BlockData::None]}
    }

    pub fn get_attributes(self) -> BlockAttributes {
        self.block_id.get_attributes()
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BlockID {
    #[default] Air,
    Dirt,
    Grass,
    Stone,
    Log,
}
impl BlockID {
    fn get_attributes(self) -> BlockAttributes {
        match self {
            BlockID::Air => BlockAttributes { health: 0, ..default()  },
            BlockID::Dirt => BlockAttributes { health: 3, tex_coords: TextureCoords::symmetrical(IVec2::new(0, 0)), ..default() },
            BlockID::Grass => BlockAttributes { health: 1, tex_coords: TextureCoords::asymmetric_y(IVec2::new(0, 1), IVec2::new(0, 0), IVec2::new(1, 1)), breaks_into: BlockID::Dirt, ..default() },
            BlockID::Stone => BlockAttributes { health: 5, tex_coords: TextureCoords::symmetrical(IVec2::new(0, 2)), ..default() },
            // Logs will have special behavior for how they get mined, most likely. (Treefelling)
            BlockID::Log => BlockAttributes { health: 2, ..default() },
            
        }
    }

    fn get_default_data(self) -> [BlockData; 1] {
        todo!()
    }
}

#[derive(Default, Clone, Copy)]
pub enum BlockData {
    #[default] None,
    // For trees and stuff.
    DamagedAdjacent(u8),
}

//TODO: Optimization: If we want to get *really* silly with optimization, we can combine everything here into a single unsigned, and start splitting bytes into nibbles
#[derive(Default, Clone, Copy)]
pub struct BlockAttributes {
    pub health: u8,
    pub toughness: u8,
    pub tex_coords: TextureCoords,
    pub breaks_into: BlockID,
}
/*
impl BlockAttributes {
    pub fn new(health: u8, toughness: u8, tex_coords: IVec2) -> BlockAttributes {
        BlockAttributes { health, toughness, tex_coords }
    }
}
 */

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
}

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

/// This struct is overkill, but pretty cool.
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