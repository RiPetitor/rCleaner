// Copyright (C) 2026 RiPetitor
// SPDX-License-Identifier: GPL-3.0-or-later

use clap::Parser;
use rcleaner::config::Config;
use rcleaner::error::Result;
use rcleaner::tui::App;
use rcleaner::{NAME, VERSION};
use std::io::IsTerminal;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = NAME, version = VERSION, about = "TUI system cleaner for Linux")]
struct Cli {
    /// Run in dry-run mode (simulate cleanup without deleting)
    #[arg(short = 'n', long)]
    dry_run: bool,

    /// Path to config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Enable verbose logging (debug level)
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let log_level = if cli.verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    env_logger::Builder::from_env(env_logger::Env::default())
        .filter_level(log_level)
        .init();

    log::info!("Starting {} v{}", NAME, VERSION);

    if !std::io::stdout().is_terminal() {
        eprintln!("rCleaner requires a terminal (TTY). Run it in a terminal emulator.");
        return Ok(());
    }

    let config_path = cli.config.unwrap_or_else(Config::default_path);

    let mut terminal = ratatui::try_init()?;
    let mut app = App::new(config_path, cli.dry_run);
    let result = app.run(&mut terminal);
    ratatui::restore();
    result?;

    log::info!("rCleaner exited successfully");
    Ok(())
}
