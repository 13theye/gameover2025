// src/main.rs

use nannou::prelude::*;
use std::time::Instant;
use tacit_gameover::config::*;

struct Model {
    // Nannou API
    draw: nannou::Draw,
    draw_renderer: nannou::draw::Renderer,

    texture: wgpu::Texture,
    texture_reshaper: wgpu::TextureReshaper,

    // FPS
    last_update: Instant,
    fps: f32,
    fps_update_interval: f32,
    frame_count: u32,
    last_fps_display_update: f32,
    frame_time_accumulator: f32,

    // When on, displays more verbose messages in terminal
    verbose: bool,
}

fn model(app: &App) -> Model {
    // Load config
    let config = Config::load().expect("Failed to load config file.");

    // Create window
    let window_id = app
        .new_window()
        .title("Tacit Group: Gameover 0.1.0")
        .size(config.window.width, config.window.height)
        .msaa_samples(1)
        .view(view)
        .key_pressed(key_pressed)
        .build()
        .unwrap();

    let window = app.window(window_id).unwrap();

    // Set up render texture
    let device = window.device();
    let draw = nannou::Draw::new();
    let texture = wgpu::TextureBuilder::new()
        .size([
            config.rendering.texture_width,
            config.rendering.texture_height,
        ])
        // Our texture will be used as the RENDER_ATTACHMENT for our `Draw` render pass.
        // It will also be SAMPLED by the `TextureCapturer` and `TextureResizer`.
        .usage(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING)
        // Use nannou's default multisampling sample count.
        .sample_count(config.rendering.texture_samples)
        // Use a spacious 16-bit linear sRGBA format suitable for high quality drawing: Rgba16Float
        // Use 8-bit for standard quality and better perforamnce: Rgba8Unorm Rgb10a2Unorm
        .format(wgpu::TextureFormat::Rgba16Float)
        // Build
        .build(device);

    // Set up rendering pipeline
    let draw_renderer = nannou::draw::RendererBuilder::new()
        .build_from_texture_descriptor(device, texture.descriptor());
    let sample_count = window.msaa_samples();

    // Create the texture reshaper.
    let texture_view = texture.view().build();
    let texture_sample_count = texture.sample_count();
    let texture_sample_type = texture.sample_type();
    let dst_format = Frame::TEXTURE_FORMAT;
    let texture_reshaper = wgpu::TextureReshaper::new(
        device,
        &texture_view,
        texture_sample_count,
        texture_sample_type,
        sample_count,
        dst_format,
    );

    Model {
        draw,
        draw_renderer,
        texture,
        texture_reshaper,

        last_update: Instant::now(),
        fps: 0.0,
        fps_update_interval: 0.3,
        last_fps_display_update: 0.0,
        frame_count: 0,
        frame_time_accumulator: 0.0,

        verbose: false,
    }
}

fn main() {
    nannou::app(model).update(update).run();
}

fn update(app: &App, model: &mut Model, _update: Update) {
    // Render to texture and handle frame recording
    render_and_capture(app, model);
}

fn view(_app: &App, model: &Model, frame: Frame) {
    //resize texture to screen
    let mut encoder = frame.command_encoder();

    model
        .texture_reshaper
        .encode_render_pass(frame.texture_view(), &mut encoder);
}

fn key_pressed(_app: &App, _model: &mut Model, _key: Key) {
    todo!();
}

fn render_and_capture(app: &App, model: &mut Model) {
    let window = app.main_window();
    let device = window.device();
    let ce_desc = wgpu::CommandEncoderDescriptor {
        label: Some("Texture renderer"),
    };
    let mut encoder = device.create_command_encoder(&ce_desc);
    let texture_view = model.texture.view().build();

    model.draw_renderer.encode_render_pass(
        device,
        &mut encoder,
        &model.draw,
        1.0,
        model.texture.size(),
        &texture_view,
        None,
    );
}
