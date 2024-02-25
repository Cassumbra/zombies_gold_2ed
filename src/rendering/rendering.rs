use bevy::prelude::*;

use crate::Chunk;

use block_mesh::ndshape::{ConstShape, ConstShape3u32};
use block_mesh::{greedy_quads, GreedyQuadsBuffer, MergeVoxel, Voxel, VoxelVisibility, RIGHT_HANDED_Y_UP_CONFIG};

#[derive(Clone, Copy, Eq, PartialEq)]
struct BoolVoxel(bool);

const EMPTY: BoolVoxel = BoolVoxel(false);
const FULL: BoolVoxel = BoolVoxel(true);

impl Voxel for BoolVoxel {
    fn get_visibility(&self) -> VoxelVisibility {
        if *self == EMPTY {
            VoxelVisibility::Empty
        } else {
            VoxelVisibility::Opaque
        }
    }
}

impl MergeVoxel for BoolVoxel {
    type MergeValue = Self;

    fn merge_value(&self) -> Self::MergeValue {
        *self
    }
}

type ChunkShape = ConstShape3u32<16, 64, 16>;



// systems
pub fn update_chunk_meshes (
    query: Query<(Entity, &Chunk), Or<(Added<Chunk>, Changed<Chunk>)>>
) {
    for (entity, chunk) in query.iter() {
        let mut voxels = [EMPTY; ChunkShape::SIZE as usize];
        
        for (i, block) in chunk.iter().enumerate() {
            voxels[i] = match block.block_id {
                crate::BlockID::Air => EMPTY,
                _ => FULL
            }
        }

        let mut buffer = GreedyQuadsBuffer::new(voxels.len());
        greedy_quads(
            &voxels,
            &ChunkShape {},
            [0; 3],
            [15, 63, 15],
            &RIGHT_HANDED_Y_UP_CONFIG.faces,
            &mut buffer
        );

        // Some quads were generated.
        assert!(buffer.quads.num_quads() > 0);
        println!("quads: {}", buffer.quads.num_quads());
    }
}