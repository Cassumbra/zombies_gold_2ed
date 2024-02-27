use std::{cmp::{max, min}, ops::Index};
use bevy::prelude::*;
use fastrand::{Rng, choice};
use sark_grids::Grid;
//use grid_tree::OctreeU32;
use noise::{Perlin, NoiseFn, Worley, core::worley::distance_functions::euclidean_squared};
//use rand::{seq::SliceRandom, thread_rng};
use derive_more::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign, };
use bevy::{ecs::{entity::{EntityMapper, MapEntities}, reflect::ReflectMapEntities}, prelude::*};

use super::*;

const SEA_LEVEL: f64 = -0.0;


//Plugin
#[derive(Default)]
pub struct MapPlugin;
/*
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app
    }
}
 */

 // Systems
pub fn generate_small_map (
    mut commands: Commands,

    seed: Res<RNGSeed>,

    //mut next_mapgen_state: ResMut<NextState<MapGenState>>,
) {
    let width = 16;
    let length = 16;
    let height = 64;

    //let bundles = [TerrainType::DeepWater, TerrainType::ShallowWater,
    //               TerrainType::Plains, TerrainType::Hills, TerrainType::Mountains, TerrainType::Hills];

    //let mut ranges = ranges_from_weights(&water_weights, [-1.0, SEA_LEVEL]);
    //ranges.append(&mut ranges_from_weights(&land_weights, [SEA_LEVEL, 1.0]));

    //let mut worley_noise = Worley::new(**seed);
    //worley_noise = worley_noise.set_distance_function(euclidean_squared);
    //worley_noise = worley_noise.set_return_type(noise::core::worley::ReturnType::Distance);
    let perlin_noise = Perlin::new(**seed);

    let worley_scaling = 10.0;
    let perlin_scaling = 0.1;

    let mut altitude_grid: Grid::<f64> = Grid::<f64>::new([width, length]);
    let mut chunk = Chunk(Grid3::filled(Block::new(BlockID::Air), [width, height, length]));

    // Set initial values
    for (position, altitude_val) in altitude_grid.iter_2d_mut() {
        let point_x = (position.x as f64); // / (width as f64);
        let point_y = (position.y as f64); // / (length as f64);
        
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

        let surface_height = (((perlin_noise.get([perlin_x, perlin_y]) + 1.0) / 2.0) * (height as f64 / 4.0)) as i32;
        for h in 0..surface_height {
            *chunk.get_mut([position.x, h, position.y]).unwrap() = Block::new(BlockID::Dirt);
        }
    }

    // TODO: Don't clone this.
    commands.spawn(chunk.clone());

    for y in (0..height).rev() {
        for x in 0..width {
            let block_id = chunk.get([x, y, 0]).unwrap().block_id;
            if block_id == BlockID::Air {
                print!(" ");
            }
            else {
                print!("#");
            }
            //println!("{:?}", chunk.get([x, y, 0]).unwrap().block_id);
        }
        println!("");
    }

    
    //next_mapgen_state.set(MapGenState::TempBand);
}


// Data
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
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BlockID {
    #[default] Air,
    Dirt,
    Stone,
    Log,
}
impl BlockID {
    fn get_attributes(self) -> BlockAttributes {
        match self {
            BlockID::Air => BlockAttributes { health: 0, toughness: 0 },
            BlockID::Dirt => BlockAttributes { health: 4, toughness: 0 },
            BlockID::Stone => BlockAttributes { health: 6, toughness: 0 },
            // Logs will have special behavior for how they get mined, most likely. (Treefelling)
            BlockID::Log => BlockAttributes { health: 2, toughness: 0 },
            
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
pub struct BlockAttributes {
    health: u8,
    toughness: u8,
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


