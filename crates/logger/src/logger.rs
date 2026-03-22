pub use tracing::{info, error, warn, debug, trace, instrument};
use tracing_subscriber::{self, EnvFilter};

/// Init the subscriber
pub fn init() {
    tracing_subscriber::fmt()
        .with_level(true)
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("trace")),
        )
        .init();
}

/// expect_log() trait
pub trait ExpectLog<T> {
    /// expect_log() function
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

impl<T: std::fmt::Debug> ExpectLog<T> for Option<T> {
    fn expect_log(self, msg: &str) -> T {
        match self {
            Some(v) => v,
            None => {
                error!("{}", msg);
                panic!("{}", msg);
            }
        }
    }
}
