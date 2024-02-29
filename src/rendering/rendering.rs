use bevy::prelude::*;
use bevy::render::mesh::{Indices, MeshVertexAttribute, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;
use bevy_xpbd_3d::components::RigidBody;
use bevy_xpbd_3d::plugins::collision::{Collider, ColliderAabb};

use crate::Chunk;

use block_mesh::ndshape::{ConstShape, ConstShape3u32};
use block_mesh::{greedy_quads, visible_block_faces, GreedyQuadsBuffer, MergeVoxel, UnitQuadBuffer, Voxel, VoxelVisibility, RIGHT_HANDED_Y_UP_CONFIG};

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

type ChunkShape = ConstShape3u32<18, 66, 18>;



// systems
pub fn update_chunk_meshes (
    mut commands: Commands,

    query: Query<(Entity, &Chunk), Or<(Added<Chunk>, Changed<Chunk>)>>,

    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, chunk) in query.iter() {
        let mut voxels = [FULL; ChunkShape::SIZE as usize];
        
        for (i, block) in chunk.iter_3d() {
            voxels[ChunkShape::linearize([(i.x + 1) as u32, (i.y + 1) as u32, (i.z + 1) as u32]) as usize] = match block.block_id {
                crate::BlockID::Air => EMPTY,
                _ => FULL
            }
        }

        let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;

        let mut buffer = UnitQuadBuffer::new();
        visible_block_faces(
            &voxels,
            &ChunkShape {},
            [0; 3],
            [17, 65, 17],
            &faces,
            &mut buffer,
        );
        let num_indices = buffer.num_quads() * 6;
        let num_vertices = buffer.num_quads() * 4;
        let mut indices = Vec::with_capacity(num_indices);
        let mut positions = Vec::with_capacity(num_vertices);
        let mut normals = Vec::with_capacity(num_vertices);
        for (group, face) in buffer.groups.into_iter().zip(faces.into_iter()) {
            for quad in group.into_iter() {
                indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
                positions.extend_from_slice(&face.quad_mesh_positions(&quad.into(), 1.0));
                normals.extend_from_slice(&face.quad_mesh_normals());
            }
        }

        // TODO: Should we maybe set this to RENDER_WORLD instead?
        let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD);
        render_mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            VertexAttributeValues::Float32x3(positions),
        );
        render_mesh.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            VertexAttributeValues::Float32x3(normals),
        );
        render_mesh.insert_attribute(
            Mesh::ATTRIBUTE_UV_0,
            VertexAttributeValues::Float32x2(vec![[0.0; 2]; num_vertices]),
        );
        render_mesh.insert_indices(Indices::U32(indices.clone()));

        //println!("{:?}", render_mesh);

        let mesh_handle = meshes.add(render_mesh.clone());

        let mut material = StandardMaterial::from(Color::rgb(0.0, 0.0, 0.0));
        material.perceptual_roughness = 0.9;

        // Some quads were generated.
        //assert!(buffer.num_quads() > 0);
        //println!("quads: {}", buffer.num_quads());

        commands.entity(entity).insert(PbrBundle{
            mesh: mesh_handle,
            material: materials.add(material),

            //visibility: Visibility::Visible,
            //inherited_visibility: InheritedVisibility::VISIBLE,
            
            ..default()
            /*
            transform: todo!(),
            global_transform: todo!(),
            visibility: todo!(),
            inherited_visibility: todo!(),
            view_visibility: todo!(),
             */
        });
    }
}