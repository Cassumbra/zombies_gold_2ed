#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::globals
#import bevy_pbr::mesh_view_bindings::view
// we can import items from shader modules in the assets folder with a quoted path
//#import "shaders/custom_material_import.wgsl"::COLOR_MULTIPLIER
const COLOR_MULTIPLIER: vec4<f32> = vec4<f32>(0.0, 0.0, 1.0, 1.0);

@group(2) @binding(0) var<uniform> material_color: vec4<f32>;
@group(2) @binding(1) var material_color_texture: texture_2d<f32>;
@group(2) @binding(2) var material_color_sampler: sampler;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    return hsv_to_rgb(vec4<f32>((globals.time + mesh.world_position.xyz[0] + mesh.world_position.xyz[1]) % 1, 1.0, 1.0, 1.0));
    //return material_color * textureSample(material_color_texture, material_color_sampler, mesh.uv) * vec4<f32>(fract(mesh.position[15]), 0.0, 0.0, 1.0);
    //return vec4<f32>(fract(mesh.position[0] / 10), 1.0, 1.0, 1.0);
    //return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}

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