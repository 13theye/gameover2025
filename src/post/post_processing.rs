// src/post/post_processing.rs
//
// Apply post-effects (bloom) to the rendered texture

use nannou::prelude::*;

pub struct PostProcessing {
    scene_texture: wgpu::Texture,
    brightness_texture: wgpu::Texture,
    blur_h_texture: wgpu::Texture,
    blur_v_texture: wgpu::Texture,
    brightness_threshold: f32,
    blur_radius: f32,
    bloom_intensity: f32,
}

impl PostProcessing {
    pub fn new(app: &App, width: u32, height: u32) -> Self {
        // Create textures for the pipeline
        let scene_texture = create_render_texture(app, width, height);
        let brightness_texture = create_render_texture(app, width, height);
        let blur_h_texture = create_render_texture(app, width, height);
        let blur_v_texture = create_render_texture(app, width, height);

        Self {
            scene_texture,
            brightness_texture,
            blur_h_texture,
            blur_v_texture,
            brightness_threshold: 0.7, // Only bloom parts brighter than this
            blur_radius: 3.0,
            bloom_intensity: 0.8,
        }
    }

    pub fn process(&self, app: &App, draw: &Draw) {
        // 1. Render scene to texture
        render_to_texture(app, draw, &self.scene_texture);

        // 2. Extract bright areas to brightness texture
        extract_brightness(
            app,
            &self.scene_texture,
            &self.brightness_texture,
            self.brightness_threshold,
        );

        // 3. Apply horizontal blur pass
        blur_horizontal(
            app,
            &self.brightness_texture,
            &self.blur_h_texture,
            self.blur_radius,
        );

        // 4. Apply vertical blur pass
        blur_vertical(
            app,
            &self.blur_h_texture,
            &self.blur_v_texture,
            self.blur_radius,
        );

        // 5. Combine original scene with bloom
        composite_final(
            app,
            &self.scene_texture,
            &self.blur_v_texture,
            self.bloom_intensity,
        );
    }
}
