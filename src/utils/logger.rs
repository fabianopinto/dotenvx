use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialize logging with the specified level
///
/// # Arguments
///
/// * `level` - The log level (trace, debug, info, warn, error)
/// * `verbose` - Whether to enable verbose output
pub fn init_logging(level: Option<&str>, verbose: bool) {
    let filter = if verbose {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"))
    } else {
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(level.unwrap_or("info")))
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(false).with_thread_ids(false))
        .init();
}
