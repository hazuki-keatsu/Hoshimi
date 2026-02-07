mod event_adapter;

use hoshimi_renderer::{Color, SkiaRenderer};
use hoshimi_shared::logger::{self, ExpectLog};
use hoshimi_ui::painter::SkiaRendererPainter;
use hoshimi_ui::prelude::*;
use hoshimi_ui::animation::Curve;
use sdl3;
use std::time::Instant;

fn main() {
    // logger subscriber
    logger::init();

    let sdl_context = sdl3::init().expect_log("SDL3 Context: Fail to init");
    let video_subsystem = sdl_context
        .video()
        .expect_log("Video Subsystem: Fail to init");

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl3::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    let window = video_subsystem
        .window("Hoshimi UI Test", 1280, 720)
        .position_centered()
        .resizable()
        .opengl()
        .build()
        .expect_log("Window: Fail to init");

    let _gl_context = window
        .gl_create_context()
        .expect_log("GL Context: Fail to init");
    gl::load_with(|s| {
        video_subsystem
            .gl_get_proc_address(s)
            .map(|f| unsafe { std::mem::transmute::<_, *const std::ffi::c_void>(f) })
            .unwrap_or(std::ptr::null())
    });

    let mut renderer = SkiaRenderer::new(1280, 720).expect_log("SkiaRenderer: Fail to init.");

    // 预加载图片资源
    renderer
        .preload_image("logos/logo.png")
        .expect_log("Failed to load logo.png");

    // Application state
    let mut welcome_text = String::from("Welcome to Hoshimi!");
    
    // Create UI tree and build test scene
    let mut ui_tree = UiTree::with_root(build_test_ui(&welcome_text));
    ui_tree.set_size(1280.0, 720.0);

    logger::info!("Hoshimi Driver: Init successfully.");
    logger::info!("UI System: Test UI created with Text, Container, and Layout widgets.");

    let mut event_pump = sdl_context
        .event_pump()
        .expect_log("Event Pump: Fail to init");
    
    // Time tracking for animation
    let mut last_time = Instant::now();
    
    'running: loop {
        // Calculate delta time
        let now = Instant::now();
        let delta = (now - last_time).as_secs_f32();
        last_time = now;
        
        for event in event_pump.poll_iter() {
            match event {
                sdl3::event::Event::Quit { .. } => break 'running,
                sdl3::event::Event::Window { win_event, .. } => match win_event {
                    sdl3::event::WindowEvent::Resized(w, h) => {
                        renderer
                            .resize(w, h)
                            .expect_log("Hoshimi Driver: Fail to resize.");
                        ui_tree.set_size(w as f32, h as f32);
                        logger::info!("UI System: Resized to {}x{}", w, h);
                    }
                    _ => {}
                },
                _ => {
                    // Convert SDL event to UI InputEvent and push to queue
                    if let Some(input_event) = event_adapter::convert_event(&event) {
                        ui_tree.push_event(input_event);
                    }
                }
            }
        }
        
        // Process all queued events (includes gesture detection)
        ui_tree.process_events();
        
        // Handle UI messages
        let mut needs_update = false;
        for message in ui_tree.take_messages() {
            handle_ui_message(&message);
            if reforward_animation(&message, &mut welcome_text) {
                needs_update = true;
            }
        }
        
        // Update UI if state changed (incremental update preserves animation state)
        if needs_update {
            let new_ui = build_test_ui(&welcome_text);
            ui_tree.update_root(&new_ui);
        }

        // Update animations
        ui_tree.tick(delta);

        renderer
            .begin_frame(Some(Color::from_rgb8(30, 30, 40)))
            .expect_log("Hoshimi Driver: Fail to begin a new frame");

        let mut painter = SkiaRendererPainter::new(&mut renderer);
        ui_tree.paint(&mut painter);

        renderer.end_frame().unwrap();
        window.gl_swap_window();
    }
}

fn build_test_ui(welcome_text: &str) -> impl Widget {
    SizedBox::expand().with_child(Center::new(
        Column::new()
            .with_main_axis_alignment(MainAxisAlignment::Center)
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .child(
                // Fade in animation example
                FadeTransition::visible(
                    Container::new().with_padding_all(10f32).child(
                        Text::new(welcome_text)
                            .with_size(32f32)
                            .with_color(Color::white()),
                    ),
                )
                .with_duration(1.0)
                .with_curve(Curve::EaseOut),
            )
            .child(
                // Slide in animation example
                SlideTransition::visible(
                    Container::new().child(
                        Text::new("This is the UI system test page.")
                            .with_size(16f32)
                            .with_color(Color::white()),
                    ),
                )
                .from_left()
                .with_duration(0.8)
                .with_curve(Curve::EaseOutCubic),
            )
            .child(
                // Animated scale example
                AnimatedScale::new(
                    Container::new()
                        .child(
                            Image::new("logos/logo.png")
                                .with_fit(ImageFit::Contain)
                                .with_size(1024f32, 400f32),
                        )
                        .with_margin(EdgeInsets {
                            top: 6f32,
                            right: 0f32,
                            bottom: 0f32,
                            left: 0f32,
                        }),
                    1.0,
                )
                .with_duration(0.6)
                .with_curve(Curve::EaseOutBack),
            )
            .child(
                // Clickable button example
                GestureDetector::new(
                    Container::new()
                        .with_color(Color::from_rgb8(70, 130, 180))
                        .with_decoration(
                            BoxDecoration::default()
                                .with_color(Color::from_rgb8(70, 130, 180))
                                .with_border_radius(BorderRadius::all(8.0)),
                        )
                        .with_padding_all(12f32)
                        .child(
                            Text::new("Click Me!")
                                .with_size(18f32)
                                .with_color(Color::white()),
                        ),
                )
                .on_tap("test_button"),
            ),
    ))
}

/// Handle UI messages from the UI system
fn handle_ui_message(message: &UIMessage) {
    match message {
        UIMessage::DialogConfirm => {
            logger::info!("UI Message: Dialog confirmed");
        }
        UIMessage::OptionSelect { index, label } => {
            logger::info!(
                "UI Message: Option selected - index: {}, label: {:?}",
                index,
                label
            );
        }
        UIMessage::MenuAction(action) => {
            logger::info!("UI Message: Menu action - {:?}", action);
        }
        UIMessage::ButtonClick { id } => {
            logger::info!("UI Message: Button clicked - id: {}", id);
        }
        UIMessage::Gesture { id, kind } => {
            logger::info!("UI Message: Gesture {:?} on element '{}'", kind, id);
        }
        UIMessage::Custom(custom) => {
            logger::info!(
                "UI Message: Custom - type: {}, payload: {:?}",
                custom.type_id,
                custom.payload
            );
        }
    }
}

fn reforward_animation(message: &UIMessage, welcome_text: &mut String) -> bool {
    if let UIMessage::Gesture { id, kind: GestureKind::Tap } = message {
        if id == "test_button" {
            *welcome_text = String::from("Look forward to your dream");
            return true;
        }
    }
    false
}
