use std::{cmp::{max, min}, ops::Index};
use bevy::{prelude::*, utils::{HashMap, HashSet}};
use fastrand::{Rng, choice};
use sark_grids::Grid;
//use grid_tree::OctreeU32;
use noise::{Perlin, NoiseFn, Worley, core::worley::distance_functions::euclidean_squared};
//use rand::{seq::SliceRandom, thread_rng};
use derive_more::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign, };
use bevy::{ecs::{entity::{EntityMapper, MapEntities}, reflect::ReflectMapEntities}, prelude::*};
use crate::{grid3::Grid3, RNGSeed, CHUNK_SIZE};

use crate::sparse_grid3::SparseGrid3;


const SEA_LEVEL: f64 = -0.0;


//Plugin
#[derive(Default)]
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ChunkMap>()
            .add_event::<LoadChunkEvent>();
    }
}
 

 // Systems
pub fn generate_chunks (
    mut commands: Commands,

    seed: Res<RNGSeed>,
    mut chunk_map: ResMut<ChunkMap>,

    mut evr_load_chunk: EventReader<LoadChunkEvent>,

    //mut next_mapgen_state: ResMut<NextState<MapGenState>>,
) {
    //let mut ranges = ranges_from_weights(&water_weights, [-1.0, SEA_LEVEL]);
    //ranges.append(&mut ranges_from_weights(&land_weights, [SEA_LEVEL, 1.0]));

    //let mut worley_noise = Worley::new(**seed);
    //worley_noise = worley_noise.set_distance_function(euclidean_squared);
    //worley_noise = worley_noise.set_return_type(noise::core::worley::ReturnType::Distance);
    let perlin_noise = Perlin::new(**seed);

    let worley_scaling = 10.0;
    let perlin_scaling = 0.025;

    for ev in evr_load_chunk.read() {
        //println!("wiggledog");
        if chunk_map.contains_key(&ev.chunk) {
            continue
        }

        let mut altitude_grid: Grid::<f64> = Grid::<f64>::new([CHUNK_SIZE, CHUNK_SIZE]);
        let mut chunk = Chunk(Grid3::filled(Block::new(BlockID::Air), [CHUNK_SIZE, CHUNK_SIZE, CHUNK_SIZE]));
    
        let offset = ev.chunk * CHUNK_SIZE;

        // Set initial values
        for (position, altitude_val) in altitude_grid.iter_2d_mut() {
            let point_x = (offset.x + position.x) as f64; // / (width as f64);
            let point_y = (offset.z + position.y) as f64; // / (length as f64);
            
            //let worley_x = point_x * worley_scaling;
            //let worley_y = point_y * worley_scaling;
            
            let perlin_x = point_x * perlin_scaling;
            let perlin_y = point_y * perlin_scaling;
            
            //let worley_noise_val = (worley_noise.get([worley_x, worley_y]) + SEA_LEVEL + 1.0 ) / 2.0; // clamp seems to provide uninteresting results. add 1 instead.
            //let perlin_noise_val = create_averaged_noise(point_x, point_y, vec![2.0, 5.0], vec![2.0, 1.0], &perlin_noise);
            //let perlin_noise_val_islands = (perlin_noise.get([perlin_x, perlin_y]) + SEA_LEVEL + 1.0) / 4.0;
    
            //let noise_val = perlin_noise_val + 
            //if perlin_noise_val > SEA_LEVEL 
            //    {worley_noise_val} 
            //else if perlin_noise_val < SEA_LEVEL - 0.25 
            //    {perlin_noise_val_islands}  
            //else 
            //    {0.0};
    
            //*altitude_val = noise_val;
            //println!("x: {}, y: {}, noise: {}", x, y, noise_val);
    
            let surface_height = (((perlin_noise.get([perlin_x, perlin_y]) + 1.0) / 2.0) * CHUNK_SIZE as f64) as i32;
            for h in 0..surface_height {
                if h == surface_height - 1 {
                    *chunk.get_mut([position.x, h, position.y]).unwrap() = Block::new(BlockID::Grass);
                }
                else {
                    *chunk.get_mut([position.x, h, position.y]).unwrap() = Block::new(BlockID::Dirt);
                }
                
            }
        }
        
        let chunk_entity = commands.spawn(chunk)
            .insert(LoadReasonList(HashSet::from([ev.load_reason])))
            .insert(Transform::from_translation((ev.chunk * CHUNK_SIZE).as_vec3()))
            .insert(GlobalTransform::default())
            .id();
        chunk_map.insert(ev.chunk, chunk_entity);
    }
    //next_mapgen_state.set(MapGenState::TempBand);
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

    mut chunk_map: ResMut<ChunkMap>,

    mut commands: Commands,

    mut evw_load_chunk: EventWriter<LoadChunkEvent>,
) {
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
        let y = 0;
        for x in min_corner.x..=max_corner.x {
            //for y in min_corner.y..=max_corner.y {
                for z in min_corner.z..=max_corner.z {
                    if let Some(chunk_entity) = chunk_map.get(&IVec3::new(x, y, z)) {
                        if let Ok(mut load_reason_list) = chunk_query.get_mut(*chunk_entity) {
                            // Try to remove this. Just in case.
                            load_reason_list.remove(&LoadReason::Spawning(entity));

                            load_reason_list.insert(LoadReason::Loader(entity));

                        }
                    }
                    else {
                        evw_load_chunk.send(LoadChunkEvent { chunk: IVec3::new(x, y, z), load_reason: LoadReason::Loader(entity) });
                    }
                }
            //}
        }

    }
}

#[derive(Clone, Event)]
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

#[derive(Default, Clone, Deref, DerefMut, Component)]
pub struct LoadReasonList(HashSet<LoadReason>);

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
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
            BlockID::Dirt => BlockAttributes { health: 4, tex_coords: TextureCoords::symmetrical(IVec2::new(0, 0)), ..default() },
            BlockID::Grass => BlockAttributes { health: 4, tex_coords: TextureCoords::asymmetric_y(IVec2::new(0, 1), IVec2::new(0, 0), IVec2::new(1, 1)), breaks_into: BlockID::Dirt, ..default() },
            BlockID::Stone => BlockAttributes { health: 6, tex_coords: TextureCoords::symmetrical(IVec2::new(0, 2)), ..default() },
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

// Components

/*
#[derive(Component, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct MapSize {
    pub width: i32,
    pub height: i32,
}
impl Default for MapSize {
    fn default() -> MapSize {
        MapSize {
            width: 80,
            height: 40,
        }
    }
}
 */


//Systems
/*
pub fn entity_map_rooms_passages (
    mut commands: Commands,

    mut res_rooms: ResMut<Rooms>,
    map_size: Res<MapSize>,
) {

    let mut map_objects: Grid<Option<Entity>> = Grid::new([map_size.width, map_size.height]);

    let mut rng = rand::thread_rng();

    const MAX_ROOMS: i32 = 30;
    const MIN_SIZE: i32 = 6;
    const MAX_SIZE: i32 = 10;

    let wall_rect = Rectangle::new(IVec2::new(0, 0), map_size.width as i32 - 1, map_size.height as i32 - 1);
    fill_rect(&mut commands, &mut map_objects, WallBundle::default(), &wall_rect);

    let mut rooms = Vec::<Rectangle>::new();

    for _i in 0..=MAX_ROOMS {
        let w = rng.gen_range(MIN_SIZE..MAX_SIZE);
        let h = rng.gen_range(MIN_SIZE..MAX_SIZE);
        let x = rng.gen_range(1..(map_size.width as i32 - w - 1));
        let y = rng.gen_range(1..(map_size.height as i32 - h - 1));

        

        let room = Rectangle::new(IVec2::new(x, y), w, h);
        //let room_ent = commands.spawn().insert(rect).id();

        let mut ok = true;
        for other_room in rooms.iter() {
            if room.intersect(other_room) { ok = false }
        }
        if ok {
            fill_rect (&mut commands, &mut map_objects, FloorBundle::default(), &room);

            if !rooms.is_empty() {
                let center = room.center();
                let previous_center = rooms[rooms.len()-1].center();
                if rng.gen_range(0..=1) == 1 {
                    fill_row(&mut commands, &mut map_objects, FloorBundle::default(), previous_center.x, center.x, previous_center.y);
                    fill_column(&mut commands, &mut map_objects, FloorBundle::default(), previous_center.y, center.y, center.x);
                } else {
                    fill_column(&mut commands, &mut map_objects, FloorBundle::default(), previous_center.y, center.y, previous_center.x);
                    fill_row(&mut commands, &mut map_objects, FloorBundle::default(), previous_center.x, center.x, center.y);
                }
            }

            rooms.push(room);
            commands.spawn(room);
        }
    }

    res_rooms.0 = rooms;

    commands.insert_resource(NextState(Some(GameState::Playing)));
}
 */

/*
fn simple_entity_map(
    mut commands: Commands,

    map_size: Res<MapSize>,
    collidables: Res<Collidables>,
) {
    let mut rng = rand::thread_rng();

    draw_line_cardinal(&mut commands, IVec2::new(0, 0), IVec2::new(0, map_size.height as i32 - 1));
    draw_line_cardinal(&mut commands, IVec2::new(map_size.width as i32 - 1, 0), IVec2::new(map_size.width as i32 - 1, map_size.height as i32 - 1));

    draw_line_cardinal(&mut commands, IVec2::new(0, map_size.height as i32 - 1), IVec2::new(map_size.width as i32 - 1, map_size.height as i32 - 1));
    draw_line_cardinal(&mut commands, IVec2::new(0, 0), IVec2::new(map_size.width as i32 - 1, 0));

    for _i in 0..100 {
        let x = rng.gen_range(0..map_size.width);
        let y = rng.gen_range(0..map_size.height);

        commands.spawn(WallBundle{
            position: Position (IVec2::new(x as i32, y as i32)),
            ..Default::default()
        });
    }
}
 */

/*
fn draw_line_cardinal( commands: &mut Commands, pos1: IVec2, pos2: IVec2 ) {
    if pos1.x == pos2.x {
        for i in pos1.y ..= pos2.y {
            commands.spawn(WallBundle{
                position: Position (IVec2::new(pos1.x, i)),
                ..Default::default()
            });
        }
    }
    else if pos1.y == pos2.y {
        for i in pos1.x ..= pos2.x {
            commands.spawn(WallBundle{
                position: Position (IVec2::new(i, pos1.y)),
                ..Default::default()
            });
        }
    }
    else {
        eprintln!("ERROR: Not a cardinal direction!");
    }
}

// Functions
fn fill_rect ( commands: &mut Commands, map_objects: &mut Grid<Option<Entity>>, bundle: impl Bundle + Copy, rect: &Rectangle) {
    
    
    for pos in map_objects.clone().rect_iter([rect.pos1.x, rect.pos1.y]..=[rect.pos2.x, rect.pos2.y]) {
        if pos.1.is_some() {
            let old_entity = map_objects[[pos.0.x as u32, pos.0.y as u32]].unwrap();
            commands.entity(old_entity).despawn();
        }
        let entity = commands.spawn(bundle)
            .insert(Position(IVec2::new(pos.0.x, pos.0.y)))
            .id();
        
        map_objects[[pos.0.x as u32, pos.0.y as u32]] = Some(entity);
    }
}

fn fill_row(commands: &mut Commands, map_objects: &mut Grid<Option<Entity>>, bundle: impl Bundle + Copy, x1: i32, x2: i32, y: i32) {
    for x in min(x1, x2)..=max(x1, x2) {
        if map_objects[[x as u32, y as u32]].is_some() {
            let old_entity = map_objects[[x as u32, y as u32]].unwrap();
            commands.entity(old_entity).despawn();
        }
        let entity = commands.spawn(bundle)
            .insert(Position(IVec2::new(x, y)))
            .id();
        
        map_objects[[x as u32, y as u32]] = Some(entity);
    }
}

fn fill_column(commands: &mut Commands, map_objects: &mut Grid<Option<Entity>>, bundle: impl Bundle + Copy, y1: i32, y2: i32, x: i32) {
    for y in min(y1, y2)..=max(y1, y2) {
        if map_objects[[x as u32, y as u32]].is_some() {
            let old_entity = map_objects[[x as u32, y as u32]].unwrap();
            commands.entity(old_entity).despawn();
        }
        let entity = commands.spawn(bundle)
            .insert(Position(IVec2::new(x, y)))
            .id();
        
        map_objects[[x as u32, y as u32]] = Some(entity);
    }
}
 */


