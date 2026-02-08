// Copyright (C) 2026 RiPetitor
// SPDX-License-Identifier: GPL-3.0-or-later

pub fn init_logger(level: log::LevelFilter) {
    env_logger::Builder::from_env(env_logger::Env::default())
        .filter_level(level)
        .init();
}
