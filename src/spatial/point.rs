// This is yoinked from sark_grids and modified for 3D purposes.
// TODO: Add credits of some kind? IDK how that works... sark_grids is MIT license

//! Traits for more easily dealing with the various types to represent 2d points/sizes
use bevy::math::*;


use crate::directions::{DIR_26, DIR_6, DIR_6_NO_DOWN};
 

/// A trait for types representing an integer point on a 2d grid.
#[allow(clippy::len_without_is_empty)]
pub trait GridPoint: Clone + Copy {
    fn x(&self) -> i32;
    fn y(&self) -> i32;
    fn z(&self) -> i32;

    fn width(&self) -> i32 {
        self.x()
    }

    fn height(&self) -> i32 {
        self.y()
    }

    fn length(&self) -> i32 {
        self.z()
    }

    fn len(&self) -> usize {
        (self.x() * self.y() * self.z()) as usize
    }

    fn as_ivec3(&self) -> IVec3 {
        IVec3::new(self.x(), self.y(), self.z())
    }

    fn as_uvec3(&self) -> UVec3 {
        self.as_ivec3().as_uvec3()
    }
    fn as_vec3(&self) -> Vec3 {
        self.as_ivec3().as_vec3()
    }

    fn as_i_array(&self) -> [i32; 3] {
        self.as_ivec3().to_array()
    }

    fn as_u_array(&self) -> [u32; 3] {
        self.as_ivec3().as_uvec3().to_array()
    }

    /// Get the grid point's corresponding 1d index.
    #[inline]
    fn as_index(&self, grid_width: usize, grid_height: usize) -> usize {
        self.x() as usize + self.y() as usize * grid_width + self.z() as usize * grid_width * grid_height
    }

    /// Returns the grid point the given number of spaces above this one.
    fn up(&self, amount: i32) -> IVec3 {
        IVec3::new(self.x(), self.y() + amount, self.z())
    }

    /// Returns the grid point the given number of spaces below this one.
    fn down(&self, amount: i32) -> IVec3 {
        IVec3::new(self.x(), self.y() - amount, self.z())
    }

    /// Returns the grid point the given number of spaces to the north of
    /// this one.
    fn north(&self, amount: i32) -> IVec3 {
        IVec3::new(self.x(), self.y(), self.z() + amount)
    }

    /// Returns the grid point the given number of spaces to the south of
    /// this one.
    fn south(&self, amount: i32) -> IVec3 {
        IVec3::new(self.x(), self.y(), self.z() - amount)
    }

    /// Returns the grid point the given number of spaces to the east of
    /// this one.
    fn east(&self, amount: i32) -> IVec3 {
        IVec3::new(self.x() + amount, self.y(), self.z())
    }

    /// Returns the grid point the given number of spaces to the west of
    /// this one.
    fn west(&self, amount: i32) -> IVec3 {
        IVec3::new(self.x() - amount, self.y(), self.z())
    }

    /// Returns the grid point offset by the given amount.
    fn offset(&self, xyz: impl GridPoint) -> IVec3 {
        self.as_ivec3() + xyz.as_ivec3()
    }

    /// The [taxicab distance](https://en.wikipedia.org/wiki/Taxicab_geometry)
    /// between two grid points.
    #[inline]
    fn taxi_dist(self, other: impl GridPoint) -> usize {
        let d = (self.as_ivec3() - other.as_ivec3()).abs();
        (d.x + d.y + d.z) as usize
    }

    /// Linearly interpolate between points a and b by the amount t.
    #[inline]
    fn lerp(self, other: impl GridPoint, t: f32) -> IVec3 {
        self.as_vec3().lerp(other.as_vec3(), t).as_ivec3()
    }

    /*
    /// Returns an iterator over the 26 points adjacent to this one.
    #[inline]
    fn adj_26(&self) -> AdjIterator {
        AdjIterator {
            i: 0,
            p: self.as_ivec3(),
            arr: DIR_26,
        }
    }
     */

    /// Returns an iterator over the 6 points adjacent to this one.
    #[inline]
    fn adj_6(&self) -> AdjIterator {
        AdjIterator {
            i: 0,
            p: self.as_ivec3(),
            arr: DIR_6,
        }
    }

    #[inline]
    fn adj_6_no_down(&self) -> AdjIterator {
        AdjIterator {
            i: 0,
            p: self.as_ivec3(),
            arr: DIR_6_NO_DOWN,
        }
    }
}

pub struct AdjIterator<'a> {
    i: usize,
    p: IVec3,
    arr: &'a [IVec3],
}

impl<'a> Iterator for AdjIterator<'a> {
    type Item = IVec3;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.arr.len() {
            return None;
        };

        let p = self.p + self.arr[self.i];
        self.i += 1;

        Some(p)
    }
}

macro_rules! impl_grid_point {
    ($type:ty) => {
        impl GridPoint for $type {
            fn x(&self) -> i32 {
                self[0] as i32
            }

            fn y(&self) -> i32 {
                self[1] as i32
            }

            fn z(&self) -> i32 {
                self[2] as i32
            }
        }
    };
}

impl_grid_point!(IVec3);
impl_grid_point!(UVec3);
impl_grid_point!([u32; 3]);
impl_grid_point!([i32; 3]);
impl_grid_point!([usize; 3]);

/// A trait for types representing a 3d size.
#[allow(clippy::len_without_is_empty)]
pub trait Size3d: Clone + Copy {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn length(&self) -> usize;

    #[inline]
    fn as_uvec3(&self) -> UVec3 {
        UVec3::new(self.width() as u32, self.height() as u32, self.length() as u32)
    }

    #[inline]
    fn len(&self) -> usize {
        self.width() * self.height() * self.length()
    }

    #[inline]
    fn as_vec3(&self) -> Vec3 {
        self.as_uvec3().as_vec3()
    }

    #[inline]
    fn as_ivec3(&self) -> IVec3 {
        self.as_uvec3().as_ivec3()
    }
    #[inline]
    fn as_array(&self) -> [usize; 3] {
        [self.width(), self.height(), self.length()]
    }
    #[inline]
    fn as_usize_array(&self) -> [usize; 3] {
        let p = self.as_uvec3();
        [p.x as usize, p.y as usize, p.z as usize]
    }
}

macro_rules! impl_size3d {
    ($type:ty) => {
        impl Size3d for $type {
            fn width(&self) -> usize {
                self[0] as usize
            }

            fn height(&self) -> usize {
                self[1] as usize
            }

            fn length(&self) -> usize {
                self[2] as usize
            }
        }
    };
}

impl_size3d!(IVec3);
impl_size3d!(UVec3);
impl_size3d!([u32; 3]);
impl_size3d!([i32; 3]);
impl_size3d!([usize; 3]);

/// A trait for types representing an arbitrary 2d point.
pub trait Point3d {
    fn x(&self) -> f32;
    fn y(&self) -> f32;
    fn z(&self) -> f32;

    fn as_ivec3(&self) -> IVec3 {
        self.as_vec3().as_ivec3()
    }
    fn as_uvec3(&self) -> UVec3 {
        self.as_vec3().as_uvec3()
    }
    fn as_vec3(&self) -> Vec3 {
        Vec3::new(self.x(), self.y(), self.z())
    }
    fn as_array(&self) -> [f32; 3] {
        self.as_vec3().to_array()
    }
    fn as_usize_array(&self) -> [usize; 3] {
        let p = self.as_uvec3();
        [p.x as usize, p.y as usize, p.z as usize]
    }
}

macro_rules! impl_point3d {
    ($type:ty) => {
        impl Point3d for $type {
            fn x(&self) -> f32 {
                self[0] as f32
            }

            fn y(&self) -> f32 {
                self[1] as f32
            }

            fn z(&self) -> f32 {
                self[2] as f32
            }
        }
    };
}

impl_point3d!(Vec3);
impl_point3d!(IVec3);
impl_point3d!(UVec3);
impl_point3d!([u32; 3]);
impl_point3d!([i32; 3]);
impl_point3d!([f32; 3]);
impl_point3d!([usize; 3]);

/*
#[cfg(test)]
mod tests {
    use bevy::math::IVec3;

    use crate::point::GridPoint;

    #[test]
    fn taxi() {
        let a = [10, 10];
        let b = [20, 20];

        let dist = GridPoint::taxi_dist(a, b);
        assert_eq!(dist, 20);
    }

    #[test]
    fn adj() {
        let points: Vec<IVec2> = [10, 10].adj_4().collect();
        assert!(points.contains(&IVec2::new(10, 9)));
        assert!(points.contains(&IVec2::new(9, 10)));
        assert!(points.contains(&IVec2::new(11, 10)));
        assert!(points.contains(&IVec2::new(10, 11)));

        let points: Vec<IVec2> = [10, 10].adj_8().collect();
        assert!(points.contains(&IVec2::new(10, 9)));
        assert!(points.contains(&IVec2::new(9, 10)));
        assert!(points.contains(&IVec2::new(11, 10)));
        assert!(points.contains(&IVec2::new(10, 11)));
        assert!(points.contains(&IVec2::new(11, 11)));
        assert!(points.contains(&IVec2::new(9, 9)));
        assert!(points.contains(&IVec2::new(11, 9)));
        assert!(points.contains(&IVec2::new(9, 11)));
    }
}
 */