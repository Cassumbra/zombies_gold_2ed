use std::mem::size_of;

use bevy::prelude::*;
use bevy::render::mesh::{Indices, MeshVertexAttribute, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::Face;
use bevy::render::view::NoFrustumCulling;
use bevy_asset_loader::prelude::*;
use itertools::iproduct;

use crate::{block_pos_from_global, chunk_pos_from_global, ChunkMap, UpdateChunkEvent, CHUNK_SIZE};

use block_mesh::ndshape::{ConstShape, ConstShape3u32};
use block_mesh::{greedy_quads, visible_block_faces, GreedyQuadsBuffer, MergeVoxel, UnitQuadBuffer, UnorientedQuad, Voxel, VoxelVisibility, RIGHT_HANDED_Y_UP_CONFIG};

#[derive(Clone, Copy, Eq, PartialEq)]
struct VisVoxel(VoxelVisibility);

const EMPTY: VisVoxel = VisVoxel(VoxelVisibility::Empty);
const TRANSLUCENT: VisVoxel = VisVoxel(VoxelVisibility::Translucent);
const FULL: VisVoxel = VisVoxel(VoxelVisibility::Opaque);

impl Voxel for VisVoxel {
    fn get_visibility(&self) -> VoxelVisibility {
        self.0
    }
}

impl MergeVoxel for VisVoxel {
    type MergeValue = Self;

    fn merge_value(&self) -> Self::MergeValue {
        *self
    }
}

type ChunkShape = ConstShape3u32<18, 18, 18>;



//Plugin
/*
#[derive(Default)]
pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<Materials>()
            .init_resource::<Atlas>();
    }
}
 */



// systems
pub fn update_chunk_meshes (
    mut commands: Commands,

    mut evr_update_chunk: EventReader<UpdateChunkEvent>,

    atlas: Res<Atlas>,
    materials: Res<Materials>,
    mut materials_assets: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_map: ResMut<ChunkMap>,
) {
    let mut seen_events = Vec::new();

    for (ev) in evr_update_chunk.read() {
        if !chunk_map.contains_key(&**ev) {
            continue
        }
        if seen_events.contains(ev) {
            continue
        }

        seen_events.push(*ev);

        // TODO: We can optimize further by having "fully full" and "fully empty" precalculated and stored in the chunk_map.
        // TODO: We can also further optimize by having voxels/water_voxels (with padding) already set up and stored in the chunk_map or something.
        let mut voxels = [EMPTY; ChunkShape::SIZE as usize];
        let mut voxels_fully_full = true;
        let mut voxels_fully_empty = false;

        let mut water_voxels = [EMPTY; ChunkShape::SIZE as usize];
        let mut water_voxels_fully_full = true;
        let mut water_voxels_fully_empty = false;

        let offset = **ev * CHUNK_SIZE;

        for (x, y, z) in iproduct!(-1..=CHUNK_SIZE, -1..=CHUNK_SIZE, -1..=CHUNK_SIZE) {
            let global_block_position = offset + IVec3::new(x, y, z);
            let chunk_position = chunk_pos_from_global(global_block_position);
            let block_position = block_pos_from_global(global_block_position);


            voxels[ChunkShape::linearize([(x + 1) as u32, (y + 1) as u32, (z + 1) as u32]) as usize] = 
            if let Some(chunk) = chunk_map.get(&chunk_position) {
                match chunk.blocks[block_position].id {
                    crate::BlockID::Air | crate::BlockID::Water => {
                        voxels_fully_full = false;
                        EMPTY
                    },
                    // TODO: We should make this use our attributes, later, once we have more blocks that are translucent.
                    crate::BlockID::Leaves => {
                        voxels_fully_empty = false;
                        TRANSLUCENT
                    },
                    _ => {
                        voxels_fully_empty = false;
                        FULL
                    }
                }
            }
            else {
                voxels_fully_empty = false;
                FULL
            };

            water_voxels[ChunkShape::linearize([(x + 1) as u32, (y + 1) as u32, (z + 1) as u32]) as usize] = 
            if let Some(chunk) = chunk_map.get(&chunk_position) {
                match chunk.blocks[block_position].id {
                    crate::BlockID::Water => {
                        water_voxels_fully_empty = false;
                        TRANSLUCENT
                    },
                    _ => {
                        water_voxels_fully_full = false;
                        EMPTY
                    },
                }
            }
            else {
                water_voxels_fully_empty = false;
                FULL
            };
        }

        let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;

        if !voxels_fully_empty || !voxels_fully_full {
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
    
                    let block = chunk_map[&**ev].blocks[UVec3::from(quad.minimum) - UVec3::new(1, 1, 1)];
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
                        //if u != 0.0 {u -= 1.0};
                        //if v != 0.0 {v -= 1.0};
                        u += tex_coord.x as f32 * 8.0;
                        v += tex_coord.y as f32 * 8.0;
                        [u/256.0, v/256.0]
                    });
                     
    
                     /*
                    let top_left = (tex_coord.as_vec2() * 8.0).to_array();
                    let top_right = [top_left[0], top_left[1] + 8.0];
                    let bottom_left = [top_left[0] + 8.0, top_left[1]];
                    let bottom_right = [top_left[0] + 8.0, top_left[1] + 8.0];
    
                    let quad_uvs = [bottom_right, top_right,
                                    bottom_left, top_left,];
    
                    let quad_uvs = quad_uvs.map(|[u, v]| [u/256.0, v/256.0]);
                     */
    
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
    
            if render_mesh.count_vertices() != 0 {
                commands.entity(chunk_map[&**ev].render_entity.unwrap())
                    .try_insert(mesh_handle)
                    .try_insert(materials.world_res_8x8.clone())
                    .try_insert(Visibility::default())
                    .try_insert(InheritedVisibility::default())
                    .try_insert(ViewVisibility::default())
                    // TODO: This is a bandaid fix. Bevy isn't frustum culling correctly and we should properly fix it instead of just disabling it. Oh well.
                    .try_insert(NoFrustumCulling);
            }
        }

        if !water_voxels_fully_empty || !water_voxels_fully_full {
            let mut buffer = UnitQuadBuffer::new();
            visible_block_faces(
                &water_voxels,
                &ChunkShape {},
                [0; 3],
                [CHUNK_SIZE as u32 + 1, CHUNK_SIZE as u32 + 1, CHUNK_SIZE as u32 + 1],
                &faces,
                &mut buffer,
            );
    
            //println!("unitquadbuffer: {:?}", buffer.groups.clone());
    
            /*
            let num_indices = buffer.num_quads() * 6 * 2;
            let num_vertices = buffer.num_quads() * 4 * 2;
            let mut indices = Vec::with_capacity(num_indices * 2);
            let mut positions = Vec::with_capacity(num_vertices * 2);
            let mut normals = Vec::with_capacity(num_vertices * 2);
            let mut uvs = Vec::with_capacity(num_vertices * 2);
            */

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
    
                    let block = chunk_map[&**ev].blocks[UVec3::from(quad.minimum) - UVec3::new(1, 1, 1)];
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
                        u += tex_coord.x as f32 * 8.0;
                        v += tex_coord.y as f32 * 8.0;
                        [u/256.0, v/256.0]
                    });
    
                    uvs.extend_from_slice(&quad_uvs);

                    /*
                    let mut block_indices = face.quad_mesh_indices(positions.len() as u32);
                    block_indices.reverse();

                    indices.extend_from_slice(&block_indices);
                    positions.extend_from_slice(&face.quad_mesh_positions(&quad.into(), 1.0));
                    normals.extend_from_slice(&face.quad_mesh_normals().map(|[x, y, z]| [-x, -y, -z]));
    
                    let block = chunk_map[&**ev].blocks[UVec3::from(quad.minimum) - UVec3::new(1, 1, 1)];
                    let attributes = block.get_attributes();
                    let normal = face.signed_normal();
    
                    let mut tex_coord = if normal.x == 1 {attributes.tex_coords.east}
                                 else if normal.x == -1 {attributes.tex_coords.west}
                                 else if normal.y == 1 {attributes.tex_coords.top}
                                 else if normal.y == -1 {attributes.tex_coords.bottom}
                                 else if normal.z == 1 {attributes.tex_coords.north}
                                 else {attributes.tex_coords.south};
    
                    if normal.y != 1 {tex_coord.x += 1};
    
                    
                    let quad_uvs = face.tex_coords(RIGHT_HANDED_Y_UP_CONFIG.u_flip_face, true, &UnorientedQuad::from(quad)).map(|uv| {
                        let mut u = uv[0] * 8.0;
                        let mut v = uv[1] * 8.0;
                        u += tex_coord.x as f32 * 8.0;
                        v += tex_coord.y as f32 * 8.0;
                        [u/256.0, v/256.0]
                    });
    
                    uvs.extend_from_slice(&quad_uvs);
                     */
                }
            }
    
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

            if render_mesh.count_vertices() != 0 {
                commands.entity(chunk_map[&**ev].water_render_entity.unwrap())
                    .try_insert(mesh_handle)
                    .try_insert(materials.water_res_8x8.clone())
                    .try_insert(Visibility::default())
                    .try_insert(InheritedVisibility::default())
                    .try_insert(ViewVisibility::default())
                    // TODO: This is a bandaid fix. Bevy isn't frustum culling correctly and we should properly fix it instead of just disabling it. Oh well.
                    .try_insert(NoFrustumCulling);
            }
        }
    }
}

pub fn modify_materials (
    materials: Res<Materials>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
) {
    if let Some(world_res_8x8) = material_assets.get_mut(&materials.world_res_8x8) {
        world_res_8x8.unlit = true;
        world_res_8x8.alpha_mode = AlphaMode::Mask(0.0);
    }

    if let Some(water_res_8x8) = material_assets.get_mut(&materials.water_res_8x8) {
        water_res_8x8.unlit = true;
        water_res_8x8.alpha_mode = AlphaMode::Blend;
        water_res_8x8.cull_mode = Some(Face::Back);
        water_res_8x8.double_sided = true;
    }
}

#[derive(AssetCollection, Resource)]
pub struct Atlas{
    #[asset(path = "textures_8x8.png")]
    pub res_8x8: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 8., tile_size_y = 8., columns = 32, rows = 32, padding_x = 0., padding_y = 0., offset_x = 0., offset_y = 0.))]
    pub items_8x8_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "items_8x8.png")]
    pub items_8x8: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct Materials{
    #[asset(standard_material)]
    #[asset(path = "textures_8x8.png")]
    pub world_res_8x8: Handle<StandardMaterial>,
    #[asset(standard_material)]
    #[asset(path = "textures_8x8.png")]
    pub water_res_8x8: Handle<StandardMaterial>,

}