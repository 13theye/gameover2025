// Vertex shader for a fullscreen triangle
@vertex
fn vs_main(@builtin(vertex_index) vert_id: u32) -> @builtin(position) vec4<f32> {
    // Create a fullscreen triangle with just the vertex id
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0)
    );
    
    return vec4<f32>(positions[vert_id], 0.0, 1.0);
}

// Composite fragment shader
@group(0) @binding(0) var scene_tex: texture_2d<f32>;
@group(0) @binding(1) var bloom_tex: texture_2d<f32>;
@group(0) @binding(2) var tex_sampler: sampler;
@group(0) @binding(3) var<uniform> intensity_uniform: f32;

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let tex_size = vec2<f32>(textureDimensions(scene_tex));
    let tex_coord = pos.xy / tex_size;
    
    // Sample original scene
    let scene_color = textureSample(scene_tex, tex_sampler, tex_coord);
    
    // Sample bloom texture
    let bloom_color = textureSample(bloom_tex, tex_sampler, tex_coord);
    
    // Calculate scene brightness for adaptive bloom
    let luminance = dot(scene_color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    
    // Stronger bloom for brighter areas
    let adaptive_intensity = mix(0.2, 1.5, smoothstep(0.5, 0.9, luminance));
    
    // Combine scene with bloom effect using brightness-based intensity
    return scene_color + bloom_color * intensity_uniform * adaptive_intensity;
}