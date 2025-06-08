// user_config.rs
//
// Copyright 2025 Martin Pobaschnig
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{backend::Backend, util, vault::VaultConfig};
use gtk::glib::user_config_dir;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use toml::de::Error;
use uuid::Uuid;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct LegacyVaultConfig {
    pub backend: Backend,
    pub encrypted_data_directory: String,
    pub mount_directory: String,
    pub session_lock: Option<bool>,
    pub use_custom_binary: Option<bool>,
    pub custom_binary_path: Option<String>,
}

pub fn get_user_config_from_legacy_config() -> HashMap<Uuid, VaultConfig> {
    log::trace!("convert_legacy_user_config_to_new()");

    let mut new_user_config: HashMap<Uuid, VaultConfig> = HashMap::new();
    let legacy_user_config = read_legacy_user_config();
    if let Some(legacy_user_config) = legacy_user_config {
        for (name, vault_config) in legacy_user_config {
            let new_vault_config = VaultConfig {
                name,
                backend: vault_config.backend,
                encrypted_data_directory: vault_config.encrypted_data_directory,
                mount_directory: vault_config.mount_directory,
                session_lock: vault_config.session_lock,
                use_custom_binary: vault_config.use_custom_binary,
                custom_binary_path: vault_config.custom_binary_path,
            };
            new_user_config.insert(util::generate_uuid(), new_vault_config);
        }
    }

    new_user_config
}

pub fn read_legacy_user_config() -> Option<HashMap<String, LegacyVaultConfig>> {
    log::trace!("read_legacy_user_config()");

    let legacy_user_config_path = get_legacy_user_config_path();

    if let Some(legacy_user_config_path) = legacy_user_config_path {
        let contents = std::fs::read_to_string(&legacy_user_config_path);
        match contents {
            Ok(content) => {
                let res: Result<HashMap<String, LegacyVaultConfig>, Error> =
                    toml::from_str(&content.clone());
                match res {
                    Ok(v) => {
                        return Some(v);
                    }
                    Err(e) => {
                        log::error!("Failed to parse user data config: {}", e);
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to read user data config: {}", e);
            }
        }
    }

    return None;
}

fn get_legacy_user_config_path() -> Option<String> {
    log::trace!("get_legacy_user_config_path()");

    match user_config_dir().as_os_str().to_str() {
        Some(user_config_directory) => {
            log::info!("Got user config dir: {}", user_config_directory);
            Some(user_config_directory.to_owned() + "/user_config.toml")
        }
        None => {
            log::error!("Could not get user data directory");
            None
        }
    }
}
