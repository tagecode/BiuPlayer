use std::path::{Path, PathBuf};

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;

pub struct LogGuard(#[allow(dead_code)] WorkerGuard);

pub fn init(log_dir: &Path) -> Result<(LogGuard, PathBuf), String> {
    std::fs::create_dir_all(log_dir).map_err(|e| e.to_string())?;

    let appender = tracing_appender::rolling::daily(log_dir, "biu.log");
    let (writer, guard) = tracing_appender::non_blocking(appender);
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("info,biuplayer=debug,biu_audio=debug,biu_library=debug")
    });

    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(writer)
        .with_ansi(false)
        .with_target(true)
        .try_init();

    tracing::info!(
        log_dir = %log_dir.display(),
        "BiuPlayer logging initialized"
    );

    Ok((LogGuard(guard), log_dir.to_path_buf()))
}
