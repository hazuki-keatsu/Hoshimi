use hoshimi_renderer::SimpleRenderer;
use sdl3;
use tracing::{info, error};
use tracing_subscriber::{self, EnvFilter};

trait ExpectLog<T> {
    fn expect_log(self, msg: &str) -> T;
}

impl<T, E: std::fmt::Debug> ExpectLog<T> for Result<T, E> {
    fn expect_log(self, msg: &str) -> T {
        match self {
            Ok(v) => v,
            Err(e) => {
                error!("{}: {:?}", msg, e);
                panic!("{}: {:?}", msg, e);
            }
        }
    }
}

fn main() {
    // logger subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let sdl_context = sdl3::init().expect_log("SDL3 Context: Fail to init");
    let video_subsystem = sdl_context.video().expect_log("Video Subsystem: Fail to init");

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl3::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    let window = video_subsystem.window("Test", 1280, 720)
        .position_centered()
        .resizable()
        .opengl()
        .build()
        .expect_log("Window: Fail to init");
    
    let _gl_context = window.gl_create_context().expect_log("GL Context: Fail to init");
    gl::load_with(|s| {
        video_subsystem.gl_get_proc_address(s)
            .map(|f| unsafe { std::mem::transmute::<_, *const std::ffi::c_void>(f) })
            .unwrap_or(std::ptr::null())
    });

    let renderer = SimpleRenderer::new();
    
    info!("Hoshimi Driver: Init successfully.");

    let mut event_pump = sdl_context.event_pump().expect_log("Event Pump: Fail to init");
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl3::event::Event::Quit { .. } => break 'running,
                sdl3::event::Event::Window { win_event, .. } => {
                    match win_event {
                        sdl3::event::WindowEvent::Resized(w, h) => {
                            renderer.resize(w, h);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        renderer.render();
        window.gl_swap_window();
    }
}