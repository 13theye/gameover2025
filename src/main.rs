// src/main.rs

use nannou::prelude::*;
use std::{collections::HashMap, time::Instant};
use tacit_gameover::{
    config::*,
    post::PostProcessing,
    views::{BackgroundManager, BoardInstance, PlayerInput},
};

struct Model {
    // Tetris Boards
    boards: HashMap<String, BoardInstance>,
    board_config: BoardConfig,

    // Background
    background: BackgroundManager,

    // Player input pending update
    player_input: Option<PlayerInput>,

    // Random
    rng: nannou::rand::rngs::ThreadRng,

    // Nannou API
    draw: nannou::Draw,
    draw_renderer: nannou::draw::Renderer,

    texture: wgpu::Texture,
    texture_reshaper: wgpu::TextureReshaper,
    post_processing: PostProcessing,

    // FPS
    last_update: Instant,
    fps: f32,
    fps_update_interval: f32,
    frame_count: usize,
    last_fps_display_update: f32,
    frame_time_accumulator: f32,

    // When on, displays more verbose messages in terminal
    verbose: bool,
}

fn model(app: &App) -> Model {
    // Load config
    let config = Config::load().expect("\nGameOver: FAILED TO LOAD CONFIG.TOML\n");

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
    let post_processing = PostProcessing::new(
        device,
        config.rendering.texture_width,
        config.rendering.texture_height,
        config.rendering.texture_samples,
    );

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
        boards: HashMap::new(),
        board_config: config.board,

        background: BackgroundManager::new(rgb(0.05, 0.03, 0.0)),

        player_input: None,

        rng: nannou::rand::thread_rng(),

        draw,
        draw_renderer,
        texture,
        texture_reshaper,
        post_processing,

        last_update: Instant::now(),
        fps: 0.0,
        fps_update_interval: 0.3,
        last_fps_display_update: 0.0,
        frame_count: 0,
        frame_time_accumulator: 0.0,

        verbose: false,
    }
}

impl Model {
    fn make_board(&mut self, id: &str, location: Vec2) {
        let config = &self.board_config;
        let board = BoardInstance::new(
            id,
            location,
            config.width,
            config.height,
            config.cell_size,
            config.gravity_interval,
            config.lock_delay,
        );
        self.boards.insert(board.id.to_owned(), board);
        println!("\n<------ Board Created: <{}> ----->", id);
        println!(
            "size: {}x{} blocks\nlocation: {}\n",
            config.width, config.height, location
        );
    }
}

fn main() {
    nannou::app(model).update(update).run();
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let now = Instant::now();
    let duration = now - model.last_update;
    let dt = duration.as_secs_f32();
    model.last_update = now;

    // FPS calculations
    if model.verbose {
        calculate_fps(app, model, dt);
    }

    // Handle the background
    model.background.draw(&model.draw, app.time);

    // Update & draw the boards
    for board in model.boards.values_mut() {
        board.update(dt, &model.player_input, &mut model.rng);
        board.draw(&model.draw);
    }

    model.player_input = None;

    // Handle FPS and origin display
    if model.verbose {
        draw_fps(model);
    }

    render_and_post(app, model);
}

fn view(_app: &App, model: &Model, frame: Frame) {
    //resize texture to screen
    let mut encoder = frame.command_encoder();

    model
        .texture_reshaper
        .encode_render_pass(frame.texture_view(), &mut encoder);
}

// ******************************* Key Capture *****************************

fn key_pressed(app: &App, model: &mut Model, key: Key) {
    match key {
        Key::Left => model.player_input = Some(PlayerInput::L),
        Key::Right => model.player_input = Some(PlayerInput::R),
        Key::Up => model.player_input = Some(PlayerInput::Rotate),
        Key::Space => model.player_input = Some(PlayerInput::HardDrop),
        Key::Return => model.player_input = Some(PlayerInput::Pause),
        Key::Key1 => model.player_input = Some(PlayerInput::SaveState),
        Key::Key2 => model.player_input = Some(PlayerInput::ResumeState),

        Key::G => {
            model.make_board("board", vec2(-300.0, 0.0));
        }
        Key::P => {
            model.verbose = !model.verbose;
            init_fps(app, model);
        }
        _ => {}
    }
}

// ******************************* Rendering and Capture *****************************
fn render_and_post(app: &App, model: &mut Model) {
    // Get the window device and queue
    let window = app.main_window();
    let device = window.device();
    let queue = window.queue();

    // Process the scene with post-processing
    let texture_view = model.texture.view().build();
    model.post_processing.process(
        device,
        queue,
        &texture_view,
        &mut model.draw_renderer,
        &model.draw,
    );
}

// Old render funciton kept here for reference
fn _render_and_capture(app: &App, model: &mut Model) {
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

    window.queue().submit(Some(encoder.finish()));
    device.poll(wgpu::Maintain::Wait);
}

// ************************ FPS and debug display  *************************************

fn draw_fps(model: &Model) {
    let draw = &model.draw;
    // Draw (+,+) axes
    draw.line()
        .points(pt2(0.0, 0.0), pt2(50.0, 0.0))
        .color(RED)
        .stroke_weight(1.0);
    draw.line()
        .points(pt2(0.0, 0.0), pt2(0.0, 50.0))
        .color(BLUE)
        .stroke_weight(1.0);

    // Visualize FPS (Optional)
    draw.text(&format!("FPS: {:.1}", model.fps))
        .x_y(900.0, 520.0)
        .color(RED)
        .font_size(20);
}

fn init_fps(app: &App, model: &mut Model) {
    model.fps = 0.0;
    model.frame_count = 0;
    model.frame_time_accumulator = 0.0;
    model.last_fps_display_update = app.time;
}

fn calculate_fps(app: &App, model: &mut Model, dt: f32) {
    model.frame_count += 1;
    model.frame_time_accumulator += dt;
    let elapsed_since_last_fps_update = app.time - model.last_fps_display_update;
    if elapsed_since_last_fps_update >= model.fps_update_interval {
        if model.frame_count > 0 {
            let avg_frame_time = model.frame_time_accumulator / model.frame_count as f32;
            model.fps = if avg_frame_time > 0.0 {
                1.0 / avg_frame_time
            } else {
                0.0
            };
        }

        // Reset accumulators
        model.frame_count = 0;
        model.frame_time_accumulator = 0.0;
        model.last_fps_display_update = app.time;
    }
}
