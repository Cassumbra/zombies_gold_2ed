use std::mem::size_of;

use bevy::prelude::*;
use bevy::render::mesh::{Indices, MeshVertexAttribute, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::view::NoFrustumCulling;
use bevy_xpbd_3d::components::RigidBody;
use bevy_xpbd_3d::plugins::collision::{Collider, ColliderAabb};

use crate::{Atlas, Chunk, TextureAtlas, CHUNK_SIZE};

use block_mesh::ndshape::{ConstShape, ConstShape3u32};
use block_mesh::{greedy_quads, visible_block_faces, GreedyQuadsBuffer, MergeVoxel, UnitQuadBuffer, UnorientedQuad, Voxel, VoxelVisibility, RIGHT_HANDED_Y_UP_CONFIG};

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

type ChunkShape = ConstShape3u32<18, 18, 18>;



// systems
pub fn update_chunk_meshes (
    mut commands: Commands,

    // TODO: If we end up having chunk changes that don't result in rendering changes, this could end up being wasted performance.
    //       Using some events might be a good idea...
    query: Query<(Entity, &Chunk), Or<(Added<Chunk>, Changed<Chunk>)>>,

    atlas: Res<Atlas>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, chunk) in query.iter() {
        //if commands.get_entity(entity).is_none() {
        //    continue
        //}

        // TODO: Default to FULL instead and optimize by providing a buffer where possible
        let mut voxels = [EMPTY; ChunkShape::SIZE as usize];
        
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
            [CHUNK_SIZE as u32 + 1, CHUNK_SIZE as u32 + 1, CHUNK_SIZE as u32 + 1],
            &faces,
            &mut buffer,
        );

        //println!("unitquadbuffer: {:?}", buffer.groups.clone());

        let num_indices = buffer.num_quads() * 6;
        let num_vertices = buffer.num_quads() * 4;
        let mut indices = Vec::with_capacity(num_indices);
        let mut positions = Vec::with_capacity(num_vertices);
        let mut normals = Vec::with_capacity(num_vertices);
        let mut uvs = Vec::with_capacity(num_vertices);
        for (group, face) in buffer.groups.into_iter().zip(faces.into_iter()) {
            for quad in group.into_iter() {
                indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
                positions.extend_from_slice(&face.quad_mesh_positions(&quad.into(), 1.0));
                normals.extend_from_slice(&face.quad_mesh_normals());

                let block = chunk[UVec3::from(quad.minimum) - UVec3::new(1, 1, 1)];
                let attributes = block.get_attributes();
                let normal = face.signed_normal();

                let mut tex_coord = if normal.x == 1 {attributes.tex_coords.east}
                             else if normal.x == -1 {attributes.tex_coords.west}
                             else if normal.y == 1 {attributes.tex_coords.top}
                             else if normal.y == -1 {attributes.tex_coords.bottom}
                             else if normal.z == 1 {attributes.tex_coords.north}
                             else {attributes.tex_coords.south};

                tex_coord.x += block.damage as i32;

                let quad_uvs = face.tex_coords(RIGHT_HANDED_Y_UP_CONFIG.u_flip_face, true, &UnorientedQuad::from(quad)).map(|uv| {
                    let mut u = uv[0] * 8.0;
                    let mut v = uv[1] * 8.0;
                    if u != 0.0 {u -= 1.0};
                    if v != 0.0 {v -= 1.0};
                    u += tex_coord.x as f32 * 8.0 + 1.0;
                    v += tex_coord.y as f32 * 8.0 + 1.0;
                    [u/256.0, v/256.0]
                });

                uvs.extend_from_slice(&quad_uvs);
            }
        }

        //println!("uvs: {:?}", uvs);
        //println!("positions: {:?}", positions);
        //println!("indices: {:?}", indices);

        // TODO: Should we maybe set this to RENDER_WORLD only instead?
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
            VertexAttributeValues::Float32x2(uvs),
        );
        render_mesh.insert_indices(Indices::U32(indices.clone()));

        //println!("{:?}", render_mesh);

        render_mesh.translate_by(Vec3::new(-1.5, -1.5, -1.5));

        let mesh_handle = meshes.add(render_mesh.clone());

        // TODO: Should we be reusing this material instead of remaking it every time?
        let mut material = StandardMaterial::from(Color::WHITE);
        material.unlit = true;
        material.base_color_texture = Some(atlas.res_8x8.clone());

        // Some quads were generated.
        //assert!(buffer.num_quads() > 0);
        //println!("quads: {}", buffer.num_quads());

        // TODO: Use insert_unique to insert a PbrBundle once insert_unique is available.
        /*
        commands.entity(entity).insert(PbrBundle{
            mesh: mesh_handle,
            material: materials.add(material),

            //visibility: Visibility::Visible,
            //inherited_visibility: InheritedVisibility::VISIBLE,

            ..default()
            /*
            global_transform: todo!(),
            view_visibility: todo!(),
             */
        });
        */
        commands.entity(entity)
            .try_insert(mesh_handle)
            .try_insert(materials.add(material))
            .try_insert(Visibility::default())
            .try_insert(InheritedVisibility::default())
            .try_insert(ViewVisibility::default())
            // TODO: This is a bandaid fix. Bevy isn't frustum culling correctly and we should properly fix it instead of just disabling it. Oh well.
            .try_insert(NoFrustumCulling);
    }
}