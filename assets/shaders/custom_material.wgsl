#import bevy_pbr::mesh_view_bindings::globals
#import bevy_pbr::utils::rand_f
#import bevy_pbr::utils::rand_u
#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    vertex_output_view_bindings::view,
    pbr_types::{STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT, STANDARD_MATERIAL_FLAGS_ALPHA_MODE_BLEND, PbrInput, pbr_input_new},
    pbr_functions as fns,
    mesh_functions::{get_world_from_local, mesh_position_local_to_clip, mesh_position_local_to_world},
}
#import bevy_core_pipeline::tonemapping::tone_mapping

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

//#import bevy_render::color_operations::hsv_to_rgb
// we can import items from shader modules in the assets folder with a quoted path
//#import "shaders/custom_material_import.wgsl"::COLOR_MULTIPLIER
const COLOR_MULTIPLIER: vec4<f32> = vec4<f32>(0.0, 0.0, 1.0, 1.0);

// We should be getting this from somewhere.
const tile_size: f32 = 8.0;
const texture_size: f32 = 256.0;

//@group(2) @binding(0) var<uniform> material_color: vec4<f32>;
@group(2) @binding(100) var material_color_texture: texture_2d<f32>;
@group(2) @binding(101) var material_color_sampler: sampler;

const MASK2: u32 = 3;
const MASK3: u32 = 7;
const MASK4: u32 = 15;
const MASK6: u32 = 63;
const MASK9: u32 = 511;
const MASK16: u32 = 65535;

struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) voxel_data: vec2<u32>,
};

struct CustomVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) texture_layer: u32,
    @location(5) face_light: vec4<f32>,
};


fn normal_from_id(id: u32) -> vec3<f32> {
    var n: vec3<f32>;
    switch id {
        case 0u {
            n = vec3(0.0, 1.0, 0.0);
        }
        case 1u {
            n = vec3(0.0, -1.0, 0.0);
        }
        case 2u {
            n = vec3(1.0, 0.0, 0.0);
        }
        case 3u {
            n = vec3(-1.0, 0.0, 0.0);
        }
        case 4u {
            n = vec3(0.0, 0.0, 1.0);
        }
        case 5u {
            n = vec3(0.0, 0.0, -1.0);
        }
        default {
            n = vec3(0.0);
        }
    }
    return n;
}

fn light_from_id(id: u32) -> vec4<f32> {
    switch id {
        case 0u {
            return vec4(1.0, 1.0, 1.0, 1.0); // top
        }
        case 2u, 3u, 4u, 5u {
            return vec4(0.7, 0.7, 0.7, 1.0); // sides
        }
        case 1u {
            return vec4(0.3, 0.3, 0.3, 1.0); // bottom
        }
        default {
            return vec4(0.0, 0.0, 0.0, 1.0);
        }
    }
}

fn color_from_id(id: u32) -> vec4<f32> {
    var r = f32(id & MASK3)/f32(MASK3);
    var g = f32((id >> 3) & MASK3)/f32(MASK3);
    var b = f32((id >> 6) & MASK3)/f32(MASK3);
    return vec4(r, g, b, 1.0);
}

@vertex
fn vertex(vertex: VertexInput) -> CustomVertexOutput {
    var out: CustomVertexOutput;

    // Vertex specific information
    var vertex_info = vertex.voxel_data.x;
    var x = f32(vertex_info & MASK6);
    var y = f32((vertex_info >> 6) & MASK6);
    var z = f32((vertex_info >> 12) & MASK6);
    var u = f32((vertex_info >> 18) & MASK6);
    var v = f32((vertex_info >> 24) & MASK6);
    var position = vec4(x, y, z, 1.0);
    
    // Quad specific information
    var quad_info = vertex.voxel_data.y;
    var n_id = quad_info & MASK3;
    var normal = normal_from_id(n_id);
    var c_id = (quad_info >> 3) & MASK9;
    var face_color = color_from_id(c_id);
    var texture_layer = quad_info >> 12;
    var face_light = light_from_id(n_id);
    var light = f32((quad_info >> 28) & MASK4) / f32(MASK4);

    out.position = mesh_position_local_to_clip(
        get_world_from_local(vertex.instance_index),
        position,
    );
    out.world_position = mesh_position_local_to_world(
        get_world_from_local(vertex.instance_index),
        position,
    );
    out.world_normal = normal;
    out.uv = vec2(u, v);
    out.color = face_color;
    out.texture_layer = texture_layer;
    out.face_light = face_light;
    return out;
}

@fragment
fn fragment(
    in: CustomVertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var vertex_output: VertexOutput;
    vertex_output.position = in.position;
    vertex_output.world_position = in.world_position;
    vertex_output.world_normal = in.world_normal;
#ifdef VERTEX_UVS
    vertex_output.uv = in.uv;
#endif
#ifdef VERTEX_UVS_B
    vertex_output.uv_b = in.uv;
#endif
#ifdef VERTEX_COLORS
    vertex_output.color = in.color;
#endif
    // generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = pbr_input_from_standard_material(vertex_output, is_front);
    
    // sample texture
        let mip_map_level = mip_map_level(vertex_output.uv * texture_size) * 1.0;
    var scale = 0.0;

    let xyz = vertex_output.world_position.xyz;
    let x = bitcast<u32>(floor(xyz.x + 0.501));
    let y = bitcast<u32>(floor(xyz.y + 0.501));
    let z = bitcast<u32>(floor(xyz.z + 0.501));
    var paired = cantor_pair(cantor_pair(x, y), z);
    var rand = f32(rand_u(&paired));
    var add_u = 0.0;
    var add_v = 0.0;
    if mip_map_level < 1.0 {
        scale = 0.0;
    }
    else if mip_map_level < 2.0 {
        scale = 0.5;
        rand = rand % 4.0;
        add_u = (rand % 2.0) * 4.0;
        add_v = floor(rand / 2.0) * 4.0;
    }
    else if mip_map_level < 3.0 {
        scale = 0.75;
        rand = rand % 16.0;
        add_u = (rand % 4.0) * 2.0;
        add_v = floor(rand / 4.0) * 2.0;
    }
    else {
        scale = 1.0;
        rand = rand % 64.0;
        add_u = rand % 8.0;
        add_v = floor(rand / 8.0);
    }
    add_u = add_u / texture_size;
    add_v = add_v / texture_size;
    pbr_input.material.base_color = in.color * in.face_light * textureSample(material_color_texture, material_color_sampler, vec2<f32>(vertex_output.uv.x - ((vertex_output.uv.x % 0.03125) * scale) + add_u, vertex_output.uv.y - ((vertex_output.uv.y % 0.03125) * scale) + add_v));

    //pbr_input.material.base_color = in.color * in.face_light * textureSampleBias(material_color_texture, material_color_sampler, in.uv, in.texture_layer, view.mip_bias);
    
    // alpha discard
    pbr_input.material.base_color = fns::alpha_discard(pbr_input.material, pbr_input.material.base_color);

#ifdef PREPASS_PIPELINE
    // in deferred mode we can't modify anything after that, as lighting is run in a separate fullscreen shader.
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    // apply lighting
    out.color = apply_pbr_lighting(pbr_input);

    // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif

    return out;

    //if vertex_output.uv.y > 40.0 / texture_size && vertex_output.uv.y < 47.0 / texture_size {
    //    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    //}


    //return hsv_to_rgb(vec4<f32>(round(mip_map_level(vertex_output.uv) * -1.0) / 4.0, 1.0, 1.0, 1.0));

    //let xyz = vertex_output.world_position.xyz;
    //let x = bitcast<u32>(floor(xyz.x + 0.501));
    //let y = bitcast<u32>(floor(xyz.y + 0.501));
    //let z = bitcast<u32>(floor(xyz.z + 0.501));
    //var paired = cantor_pair(cantor_pair(x, y), z);
    //return hsv_to_rgb(vec4<f32>(rand_f(&paired), 1.0, 1.0, 1.0));

    //-Make each block its own color. Kinda. The + 0.001 is only to make things more visually appealing. We get something akin to z-fighting otherwise. 
    //return hsv_to_rgb(vec4<f32>(round(vertex_output.world_position.xyz[0] + 0.001) / 6 % 1, 1.0, 1.0, 1.0));

    //-push uvs into top left corner. 2x2 res. can make it 4x4 by dividing by 2 instead. we can also get the other corners by adding various values to x and y. presumably.
    //return textureSample(material_color_texture, material_color_sampler, vec2<f32>(vertex_output.uv.x - (vertex_output.uv.x % 0.03125) / 32.0, vertex_output.uv.y - (vertex_output.uv.y % 0.03125) / 32.0));

    //-keygen
    //return hsv_to_rgb(vec4<f32>((globals.time + vertex_output.world_position.xyz[0] + vertex_output.world_position.xyz[1]) % 1, 1.0, 1.0, 1.0));

    //return material_color * textureSample(material_color_texture, material_color_sampler, vertex_output.uv) * vec4<f32>(fract(vertex_output.position[15]), 0.0, 0.0, 1.0);
    //return vec4<f32>(fract(vertex_output.position[0] / 10), 1.0, 1.0, 1.0);
    //return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}

// We apparently don't need this because bevy already implements this. Cry
// Actually importing the one bevy implements isnt working? IDFK.
fn hsv_to_rgb(hsv: vec4<f32>) -> vec4<f32> {
    let chroma = hsv[2] * hsv[1];
    let hue_prime = hsv[0] * 6;
    let x = chroma * (1 - abs(hue_prime % 2 - 1));

    if 0 <= hue_prime && hue_prime < 1 {
        return vec4<f32>(chroma, x, 0, hsv[3]);
    }
    else if hue_prime < 2 {
        return vec4<f32>(x, chroma, 0, hsv[3]);
    }
    else if hue_prime < 3 {
        return vec4<f32>(0, chroma, x, hsv[3]);
    }
    else if hue_prime < 4 {
        return vec4<f32>(0, x, chroma, hsv[3]);
    }
    else if hue_prime < 5 {
        return vec4<f32>(x, 0, chroma, hsv[3]);
    }
    else {
        return vec4<f32>(chroma, 0, x, hsv[3]);
    }
}

fn cantor_pair(x: u32, y: u32) -> u32 {
    return (((x + y) * (x + y + 1)) / 2) + y;
}

fn mip_map_level(texture_coordinate: vec2<f32>) -> f32 {
    let dx_vtc = dpdx(texture_coordinate);
    let dy_vtc = dpdy(texture_coordinate);
    let delta_max_sqr = max(dot(dx_vtc, dx_vtc), dot(dy_vtc, dy_vtc));
    return 0.5 * log2(delta_max_sqr);
}

//fn silly_texture_sample (texture: texture_2d<f32>, samplerr: sampler) -> vec4<f32> {

//}