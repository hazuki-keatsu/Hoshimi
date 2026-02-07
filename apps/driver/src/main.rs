mod event_adapter;
mod counter_page;
mod animation_test_page;

use counter_page::CounterPage;
use animation_test_page::AnimationTestPage;
use hoshimi_renderer::{Color, SkiaRenderer};
use hoshimi_shared::logger::{self, ExpectLog};
use hoshimi_ui::painter::SkiaRendererPainter;
use hoshimi_ui::prelude::*;
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
        .window("Hoshimi UI Test - Counter Page", 1280, 720)
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

    // Create Router with CounterPage
    let mut router = Router::with_initial_page(CounterPage::new());
    router.set_size(1280.0, 720.0);

    logger::info!("Hoshimi Driver: Init successfully.");
    logger::info!("Router: CounterPage loaded.");
    logger::info!("Controls: Up/Down arrows to change count, R to reset, Escape to quit.");

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
                        router.set_size(w as f32, h as f32);
                        logger::info!("Router: Resized to {}x{}", w, h);
                    }
                    _ => {}
                },
                sdl3::event::Event::KeyDown { keycode: Some(keycode), .. } => {
                    handle_key_input(&mut router, keycode);
                }
                _ => {
                    // Convert SDL event to UI InputEvent and push to queue
                    if let Some(input_event) = event_adapter::convert_event(&event) {
                        router.push_event(input_event);
                    }
                }
            }
        }
        
        // Process all queued events (triggers gesture detection)
        router.process_events();
        
        // Handle UI messages from router
        for message in router.take_messages() {
            handle_ui_message(&message, &mut router);
        }

        // Update animations and check for page rebuilds
        router.tick(delta);

        renderer
            .begin_frame(Some(Color::from_rgb8(30, 30, 40)))
            .expect_log("Hoshimi Driver: Fail to begin a new frame");

        let mut painter = SkiaRendererPainter::new(&mut renderer);
        router.paint(&mut painter);

        renderer.end_frame().unwrap();
        window.gl_swap_window();
    }
}

/// Handle keyboard input for the counter page
fn handle_key_input(router: &mut Router, keycode: sdl3::keyboard::Keycode) {
    use sdl3::keyboard::Keycode;
    
    if let Some(page) = router.current_page_mut() {
        if let Some(counter) = page.as_any_mut().downcast_mut::<CounterPage>() {
            match keycode {
                Keycode::Up | Keycode::KpPlus | Keycode::Equals => {
                    counter.increment();
                    logger::info!("Counter: Incremented to {}", counter.count());
                }
                Keycode::Down | Keycode::KpMinus | Keycode::Minus => {
                    counter.decrement();
                    logger::info!("Counter: Decremented to {}", counter.count());
                }
                Keycode::R => {
                    counter.reset();
                    logger::info!("Counter: Reset to 0");
                }
                _ => {}
            }
        }
    }
}

/// Handle UI messages from the router
fn handle_ui_message(message: &UIMessage, router: &mut Router) {
    match message {
        UIMessage::Gesture { id, kind: GestureKind::Tap } => {
            logger::info!("UI Message: Tap on '{}'", id);
            
            // Handle navigation
            match id.as_str() {
                "btn_to_animation_test" => {
                    router.push(AnimationTestPage::new());
                    logger::info!("Router: Navigated to Animation Test Page");
                    return;
                }
                "btn_back_to_counter" => {
                    router.pop();
                    logger::info!("Router: Navigated back to Counter Page");
                    return;
                }
                _ => {}
            }
            
            // Handle button clicks in CounterPage
            if let Some(page) = router.current_page_mut() {
                if let Some(counter) = page.as_any_mut().downcast_mut::<CounterPage>() {
                    match id.as_str() {
                        "btn_increment" => {
                            counter.increment();
                            logger::info!("Counter: Incremented to {}", counter.count());
                        }
                        "btn_decrement" => {
                            counter.decrement();
                            logger::info!("Counter: Decremented to {}", counter.count());
                        }
                        "btn_reset" => {
                            counter.reset();
                            logger::info!("Counter: Reset to 0");
                        }
                        _ => {}
                    }
                }
            }
        }
        UIMessage::Gesture { id, kind: GestureKind::Press } => {
            logger::trace!("UI Message: Press on '{}'", id);
            
            // Handle button press - activate shadow
            if let Some(page) = router.current_page_mut() {
                if let Some(counter) = page.as_any_mut().downcast_mut::<CounterPage>() {
                    counter.set_button_pressed(id, true);
                } else if let Some(anim_page) = page.as_any_mut().downcast_mut::<AnimationTestPage>() {
                    anim_page.set_button_pressed(true);
                }
            }
        }
        UIMessage::Gesture { id, kind: GestureKind::Release } => {
            logger::trace!("UI Message: Release on '{}'", id);
            
            // Handle button release - deactivate shadow
            if let Some(page) = router.current_page_mut() {
                if let Some(counter) = page.as_any_mut().downcast_mut::<CounterPage>() {
                    counter.set_button_pressed(id, false);
                } else if let Some(anim_page) = page.as_any_mut().downcast_mut::<AnimationTestPage>() {
                    anim_page.set_button_pressed(false);
                }
            }
        }
        UIMessage::ButtonClick { id } => {
            logger::info!("UI Message: Button clicked - id: {}", id);
        }
        UIMessage::Gesture { id, kind } => {
            logger::info!("UI Message: Gesture {:?} on element '{}'", kind, id);
        }
        _ => {
            logger::info!("UI Message: {:?}", message);
        }
    }
}
