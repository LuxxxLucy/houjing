use bevy::log::LogPlugin;

// Default log configuration constants
const DEFAULT_LOG_FILTER: &str = "info,wgpu_core=warn,wgpu_hal=warn,houjing_main=debug";
const DEFAULT_LOG_LEVEL: bevy::log::Level = bevy::log::Level::DEBUG;

pub fn get_log_plugin() -> LogPlugin {
    LogPlugin {
        filter: DEFAULT_LOG_FILTER.into(),
        level: DEFAULT_LOG_LEVEL,
        update_subscriber: None,
    }
}
