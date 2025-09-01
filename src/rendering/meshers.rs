use block_mesh::{ilattice::{self, extent::Extent, glam::UVec3}, ndshape::{self, Shape}, OrientedBlockFace, UnitQuadBuffer, UnorientedUnitQuad, Voxel, VoxelVisibility};

// Shamelessly ripped from block_mesh and dumbed down
pub fn assert_in_bounds<T, S>(voxels: &[T], voxels_shape: &S, min: [u32; 3], max: [u32; 3])
where
    S: Shape<3, Coord = u32>,
{
    assert!(
        voxels_shape.size() as usize <= voxels.len(),
        "voxel buffer size {:?} is less than the shape size {:?}; would cause access out of bounds",
        voxels.len(),
        voxels_shape.size()
    );
    let shape = voxels_shape.as_array();
    let local_extent = Extent::from_min_and_shape(UVec3::ZERO, UVec3::from(shape));
    local_extent
        .check_positive_shape()
        .unwrap_or_else(|| panic!("Invalid shape={shape:?}"));
    let query_extent = Extent::from_min_and_max(UVec3::from(min), UVec3::from(max));
    query_extent.check_positive_shape().unwrap_or_else(|| {
        panic!("Invalid extent min={min:?} max={max:?}; has non-positive shape")
    });
    assert!(
        query_extent.is_subset_of(&local_extent),
        "min={min:?} max={max:?} would access out of bounds"
    );
}

/// Used as a dummy for functions that must wrap a voxel
/// but don't want to change the original's properties.
struct IdentityVoxel<'a, T: Voxel>(&'a T);

impl<'a, T: Voxel> Voxel for IdentityVoxel<'a, T> {
    #[inline]
    fn get_visibility(&self) -> VoxelVisibility {
        self.0.get_visibility()
    }
}

impl<'a, T: Voxel> From<&'a T> for IdentityVoxel<'a, T> {
    fn from(voxel: &'a T) -> Self {
        Self(voxel)
    }
}

pub fn naive_mesh_wrapper<T, S>(
    voxels: &[T],
    voxels_shape: &S,
    min: [u32; 3],
    max: [u32; 3],
    faces: &[OrientedBlockFace; 6],
    output: &mut UnitQuadBuffer,
) where
    T: Voxel,
    S: Shape<3, Coord = u32>,
{
    naive_mesh::<_, IdentityVoxel<T>, _>(
        voxels,
        voxels_shape,
        min,
        max,
        faces,
        output,
    )
}

pub fn naive_mesh<'a, T, V, S>(
    voxels: &'a [T],
    voxels_shape: &S,
    min: [u32; 3],
    max: [u32; 3],
    faces: &[OrientedBlockFace; 6],
    output: &mut UnitQuadBuffer,
) where
    V: Voxel + From<&'a T>,
    S: Shape<3, Coord = u32>,
{
    assert_in_bounds(voxels, voxels_shape, min, max);

    let min = UVec3::from(min).as_ivec3();
    let max = UVec3::from(max).as_ivec3();
    let extent = Extent::from_min_and_max(min, max);
    let interior = extent.padded(-1); // Avoid accessing out of bounds with a 3x3x3 kernel.
    let interior =
        Extent::from_min_and_shape(interior.minimum.as_uvec3(), interior.shape.as_uvec3());

    let kernel_strides =
        faces.map(|face| voxels_shape.linearize(face.signed_normal().as_uvec3().to_array()));

    for p in interior.iter3() {
        let p_array = p.to_array();
        let p_index = voxels_shape.linearize(p_array);
        let p_voxel = V::from(unsafe { voxels.get_unchecked(p_index as usize) });

        if let VoxelVisibility::Empty = p_voxel.get_visibility() {
            continue;
        }

        for (face_index, face_stride) in kernel_strides.into_iter().enumerate() {
            let neighbor_index = p_index.wrapping_add(face_stride);
            let neighbor_voxel = V::from(unsafe { voxels.get_unchecked(neighbor_index as usize) });

            output.groups[face_index].push(UnorientedUnitQuad { minimum: p_array });
        }
    }
}