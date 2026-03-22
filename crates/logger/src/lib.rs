//! Hoshimi Logger
//! 
//! Provide the basic log service for Hoshimi Engine.
//! 
//! ## Usage
//! 
//! If you wanna enable tracing-subscriber to display log in the current terminal,
//! you should init Hoshimi Logger like below:
//! 
//! ```ignore
//! logger::init();
//! ``` 
//! 
//! When you use the Hoshimi Logger crate, 
//! the function `expect_log()` will be implemented for `Result<T,E>` and `Option<T>`.
//! 
//! By using the function above to replace the default function `expect()`,
//! the error message will be subscribed by Hoshimi Logger.
//! 
//! Example:
//! ```ignore
//! let sdl_context = sdl3::init().expect_log("SDL3 Context: Fail to init");
//! let video_subsystem = sdl_context
//!     .video()
//!     .expect_log("Video Subsystem: Fail to init");
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

/// The core module of Hoshimi Logger
pub mod logger;

pub use logger::*;