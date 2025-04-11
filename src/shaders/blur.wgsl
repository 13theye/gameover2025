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

// Gaussian blur fragment shader
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;
@group(0) @binding(2) var<uniform> direction: vec2<f32>; // (1,0) or (0,1)

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let tex_size = vec2<f32>(textureDimensions(tex));
    let tex_coord = pos.xy / tex_size;
    
    // Blur parameters
    let blur_radius = 8.0;
    let sigma = blur_radius / 2.0;
    
    // Gaussian blur calculation
    var result = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    var weight_sum = 0.0;
    
    // Sample multiple pixels along the blur direction
    for (var i = -blur_radius; i <= blur_radius; i += 1.0) {
        let offset = direction * i / tex_size;
        let sample_pos = tex_coord + offset;
        
        // Calculate Gaussian weight
        let weight = exp(-(i * i) / (2.0 * sigma * sigma));
        
        // Sample and accumulate
        let sample = textureSample(tex, tex_sampler, sample_pos);
        result += sample * weight;
        weight_sum += weight;
    }
    
    // Normalize by weight sum
    return result / weight_sum;
}