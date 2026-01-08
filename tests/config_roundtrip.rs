use rcleaner::config::Config;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_config_path() -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut path = std::env::temp_dir();
    path.push(format!("rcleaner-it-config-{nanos}-{}", std::process::id()));
    path.push("config.toml");
    path
}

#[test]
fn test_config_roundtrip_integration() {
    let path = temp_config_path();
    let mut config = Config::default();
    config.safety.level = "aggressive".to_string();
    config.profiles.aggressive.keep_recent_kernels = 3;

    config.save(&path).unwrap();
    let loaded = Config::load(&path).unwrap();

    assert_eq!(loaded.safety.level, "aggressive");
    assert_eq!(loaded.profiles.aggressive.keep_recent_kernels, 3);
}
