// This is yoinked from sark_grids and modified for 3D purposes.
// TODO: Add credits of some kind? IDK how that works... sark_grids is MIT license

//! Utilities for dealing with directions on a 2d grid.
use bevy::math::IVec3;

use crate::point::GridPoint;

pub const UP: IVec3 = IVec3::from_array([0, 1, 0]);
pub const DOWN: IVec3 = IVec3::from_array([0, -1, 0]);

pub const NORTH: IVec3 = IVec3::from_array([0, 0, 1]);
pub const SOUTH: IVec3 = IVec3::from_array([0, 0, -1]);
pub const EAST: IVec3 = IVec3::from_array([1, 0, 0]);
pub const WEST: IVec3 = IVec3::from_array([-1, 0, 0]);

pub const UP_NORTH: IVec3 = IVec3::from_array([0, 1, 1]);
pub const UP_SOUTH: IVec3 = IVec3::from_array([0, 1, -1]);
pub const UP_EAST: IVec3 = IVec3::from_array([1, 1, 0]);
pub const UP_WEST: IVec3 = IVec3::from_array([-1, 1, 0]);

pub const DOWN_NORTH: IVec3 = IVec3::from_array([0, -1, 1]);
pub const DOWN_SOUTH: IVec3 = IVec3::from_array([0, -1, -1]);
pub const DOWN_EAST: IVec3 = IVec3::from_array([1, -1, 0]);
pub const DOWN_WEST: IVec3 = IVec3::from_array([-1, -1, 0]);

pub const NORTH_EAST: IVec3 = IVec3::from_array([1, 0, 1]);
pub const NORTH_WEST: IVec3 = IVec3::from_array([-1, 0, 1]);
pub const SOUTH_EAST: IVec3 = IVec3::from_array([1, 0, -1]);
pub const SOUTH_WEST: IVec3 = IVec3::from_array([-1, 0, -1]);

pub const UP_NORTH_EAST: IVec3 = IVec3::from_array([1, 1, 1]);
pub const UP_NORTH_WEST: IVec3 = IVec3::from_array([-1, 1, 1]);
pub const UP_SOUTH_EAST: IVec3 = IVec3::from_array([1, 1, -1]);
pub const UP_SOUTH_WEST: IVec3 = IVec3::from_array([-1, 1, -1]);

pub const DOWN_NORTH_EAST: IVec3 = IVec3::from_array([1, -1, 1]);
pub const DOWN_NORTH_WEST: IVec3 = IVec3::from_array([-1, -1, 1]);
pub const DOWN_SOUTH_EAST: IVec3 = IVec3::from_array([1, -1, -1]);
pub const DOWN_SOUTH_WEST: IVec3 = IVec3::from_array([-1, -1, -1]);

/// Array of six orthogonal grid directions.
pub const DIR_6: &[IVec3] = &[UP, DOWN, NORTH, SOUTH, EAST, WEST];
pub const DIR_6_NO_DOWN: &[IVec3] = &[UP, NORTH, SOUTH, EAST, WEST];

/// Array of twenty six adjacent grid directions.
pub const DIR_26: &[IVec3] = &[
    UP, DOWN, NORTH, SOUTH, EAST, NORTH_EAST, NORTH_WEST, SOUTH_EAST, SOUTH_WEST,
];

/// Four orthogonal directions on a 2d grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir6 {
    Up,
    Down,
    North,
    South,
    East,
    West,
}

impl From<Dir6> for IVec3 {
    fn from(d: Dir6) -> Self {
        match d {
            Dir6::Up => UP,
            Dir6::Down => DOWN,
            Dir6::North => NORTH,
            Dir6::South => SOUTH,
            Dir6::East => EAST,
            Dir6::West => WEST,
        }
    }
}

impl Dir6 {
    /// Retrieve the direction from the given point, or none if it's (0,0,0).
    pub fn from_point(p: impl GridPoint) -> Option<Dir6> {
        match p.as_ivec3().signum().to_array() {
            [0, 1, 0] => Some(Dir6::Up),
            [0, -1, 0] => Some(Dir6::Down),
            [1, 0, 0] => Some(Dir6::East),
            [-1, 0, 0] => Some(Dir6::West),
            [0, 0, 1] => Some(Dir6::North),
            [0, 0, -1] => Some(Dir6::South),
            _ => None,
        }
    }

    /*

    /// Retrieve a direction from it's corresponding index.
    pub fn from_index(i: usize) -> Option<Dir4> {
        match i {
            0 => Some(Dir4::Up),
            1 => Some(Dir4::Down),
            2 => Some(Dir4::Left),
            3 => Some(Dir4::Right),
            _ => None,
        }
    }

    /// Convert a direction to it's corresponding index.
    pub fn to_index(&self) -> usize {
        match self {
            Dir4::Up => 0,
            Dir4::Down => 1,
            Dir4::Left => 2,
            Dir4::Right => 3,
        }
    }

     */
}

/*

/// 8 directions on a 2d grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir8 {
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

impl Dir8 {
    /// Retrieve the direction from the given point, or none if it's (0,0).
    pub fn from_point(p: impl GridPoint) -> Option<Dir8> {
        match p.as_ivec2().signum().to_array() {
            [0, 1] => Some(Dir8::Up),
            [0, -1] => Some(Dir8::Down),
            [-1, 0] => Some(Dir8::Left),
            [1, 0] => Some(Dir8::Right),
            [-1, 1] => Some(Dir8::UpLeft),
            [1, 1] => Some(Dir8::UpRight),
            [-1, -1] => Some(Dir8::DownLeft),
            [1, -1] => Some(Dir8::DownRight),
            _ => None,
        }
    }

    /// Retrieve a direction from an index.
    pub fn from_index(i: usize) -> Option<Dir8> {
        match i {
            0 => Some(Dir8::Up),
            1 => Some(Dir8::Down),
            2 => Some(Dir8::Left),
            3 => Some(Dir8::Right),
            4 => Some(Dir8::UpLeft),
            5 => Some(Dir8::UpRight),
            6 => Some(Dir8::DownLeft),
            7 => Some(Dir8::DownRight),
            _ => None,
        }
    }

    /// Convert a direction to it's corresponding index.
    pub fn to_index(&self) -> usize {
        match self {
            Dir8::Up => 0,
            Dir8::Down => 1,
            Dir8::Left => 2,
            Dir8::Right => 3,
            Dir8::UpLeft => 4,
            Dir8::UpRight => 5,
            Dir8::DownLeft => 6,
            Dir8::DownRight => 7,
        }
    }
}

impl From<Dir8> for IVec2 {
    fn from(d: Dir8) -> Self {
        match d {
            Dir8::Up => UP,
            Dir8::Down => DOWN,
            Dir8::Left => LEFT,
            Dir8::Right => RIGHT,
            Dir8::UpLeft => UP_LEFT,
            Dir8::UpRight => UP_RIGHT,
            Dir8::DownLeft => DOWN_LEFT,
            Dir8::DownRight => DOWN_RIGHT,
        }
    }
}

 */