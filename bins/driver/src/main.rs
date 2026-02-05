use hoshimi_renderer::{SceneRenderer, UIColor, UIRect};
use hoshimi_shared::logger::{self, ExpectLog};
use sdl3;

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
        .window("Test", 1280, 720)
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

    let mut renderer = SceneRenderer::new(1280, 720).expect_log("SceneRenderer: Fail to init.");

    logger::info!("Hoshimi Driver: Init successfully.");

    let mut event_pump = sdl_context
        .event_pump()
        .expect_log("Event Pump: Fail to init");
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl3::event::Event::Quit { .. } => break 'running,
                sdl3::event::Event::Window { win_event, .. } => match win_event {
                    sdl3::event::WindowEvent::Resized(w, h) => {
                        renderer
                            .resize(w, h)
                            .expect_log("Hoshimi Driver: Fail to resize.");
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        let rect_width = 256;
        let rect_height = 128;
        renderer
            .begin_frame(Some(UIColor::new(111, 66, 193)))
            .expect_log("Hoshimi Driver: Fail to begin a new frame");
        renderer.fill_rect(
            UIRect::new(
                ((renderer.width() as f32) - rect_width as f32) / 2.0,
                ((renderer.height() as f32) - rect_height as f32) / 2.0,
                rect_width as f32,
                rect_height as f32,
            ),
            UIColor::blue(),
        ).expect_log("Hoshimi Driver: Fail to draw rectangle.");
        renderer.end_frame().unwrap();
        window.gl_swap_window();
    }
}
