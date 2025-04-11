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

// Brightness extraction fragment shader
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;
@group(0) @binding(2) var<uniform> threshold_uniform: f32;

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let tex_size = vec2<f32>(textureDimensions(tex));
    let tex_coord = pos.xy / tex_size;
    
    let color = textureSample(tex, tex_sampler, tex_coord);
    
    // Calculate luminance
    let luminance = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    
    // Apply threshold with smooth transition
    let threshold = threshold_uniform;
    let knee = 0.1; // Softness of the threshold
    
    // Soft thresholding
    let brightness = smoothstep(threshold - knee, threshold + knee, luminance);
    
    // Adaptive intensity - brighter pixels bloom more intensely
    let intensity = pow(brightness, 2.0); // Square for stronger effect on bright areas
    
    // Apply to color
    return vec4<f32>(color.rgb * intensity, color.a);
}