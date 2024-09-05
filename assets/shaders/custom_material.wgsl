#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::globals
#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::utils::rand_f
//#import bevy_render::color_operations::hsv_to_rgb
// we can import items from shader modules in the assets folder with a quoted path
//#import "shaders/custom_material_import.wgsl"::COLOR_MULTIPLIER
const COLOR_MULTIPLIER: vec4<f32> = vec4<f32>(0.0, 0.0, 1.0, 1.0);

//@group(2) @binding(0) var<uniform> material_color: vec4<f32>;
@group(2) @binding(1) var material_color_texture: texture_2d<f32>;
@group(2) @binding(2) var material_color_sampler: sampler;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    let xyz = mesh.world_position.xyz;
    let x = bitcast<u32>(floor(xyz.x + 0.501));
    let y = bitcast<u32>(floor(xyz.y + 0.501));
    let z = bitcast<u32>(floor(xyz.z + 0.501));
    var paired = cantor_pair(cantor_pair(x, y), z);
    return hsv_to_rgb(vec4<f32>(rand_f(&paired), 1.0, 1.0, 1.0));

    //-Make each block its own color. Kinda. The + 0.001 is only to make things more visually appealing. We get something akin to z-fighting otherwise. 
    //return hsv_to_rgb(vec4<f32>(round(mesh.world_position.xyz[0] + 0.001) / 6 % 1, 1.0, 1.0, 1.0));

    //-push uvs into top left corner. 2x2 res. can make it 4x4 by dividing by 2 instead. we can also get the other corners by adding various values to x and y. presumably.
    //return textureSample(material_color_texture, material_color_sampler, vec2<f32>(mesh.uv.x - (mesh.uv.x % 0.03125) / 1.3333, mesh.uv.y - (mesh.uv.y % 0.03125) / 1.33333));

    //-keygen
    //return hsv_to_rgb(vec4<f32>((globals.time + mesh.world_position.xyz[0] + mesh.world_position.xyz[1]) % 1, 1.0, 1.0, 1.0));

    //return material_color * textureSample(material_color_texture, material_color_sampler, mesh.uv) * vec4<f32>(fract(mesh.position[15]), 0.0, 0.0, 1.0);
    //return vec4<f32>(fract(mesh.position[0] / 10), 1.0, 1.0, 1.0);
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