use tauri_plugin_log::{Builder, RotationStrategy, Target, TargetKind};

pub fn init() -> Builder {
    Builder::new()
        .targets([
            Target::new(TargetKind::LogDir { file_name: None }),
            Target::new(TargetKind::Stdout),
        ])
        .rotation_strategy(RotationStrategy::KeepAll)
        .level(if cfg!(debug_assertions) {
            log::LevelFilter::Debug
        } else {
            // Im Release Mode reicht Warn oder Info
            log::LevelFilter::Debug
        })
        .filter(|metadata| {
            metadata.target().starts_with("palaxy")
                || metadata.target().starts_with("common")
                || metadata.target().starts_with("packager")
                || metadata.target().starts_with("scanner")
        })
        .format(|out, message, record| {
            let time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            out.finish(format_args!(
                "[{} {} {}] [{}:{}] {}",
                time,
                record.level(),
                record.target(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                message
            ))
        })
}
