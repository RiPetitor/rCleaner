use rcleaner::config::Config;
use rcleaner::error::Result;
use rcleaner::{NAME, VERSION, tui::App};
use std::io::IsTerminal;

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default())
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("Starting {} v{}", NAME, VERSION);

    let config_path = Config::default_path();
    let _config = match Config::load(&config_path) {
        Ok(config) => {
            log::info!("Loaded configuration from {:?}", config_path);
            config
        }
        Err(e) => {
            log::warn!("Failed to load config, using defaults: {}", e);
            Config::default()
        }
    };

    log::info!("Initializing TUI...");
    if !std::io::stdout().is_terminal() {
        log::error!("TUI requires a TTY. Run rCleaner in a terminal.");
        return Ok(());
    }

    let mut terminal = ratatui::try_init()?;
    log::info!("TUI initialized successfully");

    let mut app = App::new();
    let result = app.run(&mut terminal);
    ratatui::restore();
    result?;
    log::info!("rCleaner exited successfully");

    Ok(())
}
