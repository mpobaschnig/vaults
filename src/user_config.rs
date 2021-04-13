// user_config.rs
//
// Copyright 2021 Martin Pobaschnig <mpobaschnig@posteo.de>
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

use gtk::glib::get_user_data_dir;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use toml::de::Error;

use crate::vault::{Vault, Vaults};

pub static USER_DATA_DIRECTORY: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

pub static VAULTS: Lazy<Mutex<Vaults>> = Lazy::new(|| Mutex::new(Vaults { vault: vec![] }));

pub fn init() {
    match USER_DATA_DIRECTORY.lock() {
        Ok(mut user_data_directory) => match get_user_data_dir().as_os_str().to_str() {
            Some(user_data_dir) => {
                user_data_directory.replace(user_data_dir.to_string() + "/user_config.toml");
                log::debug!("Using user data directory: {}", user_data_dir.to_string());
            }
            None => {
                log::error!("Could not get user data directory");
            }
        },
        Err(e) => {
            log::error!("Failed to aquire mutex lock of USER_DATA_DIRECTORY: {}", e);
        }
    }

    read();
}

pub fn read() {
    match USER_DATA_DIRECTORY.lock() {
        Ok(user_data_directory) => match VAULTS.lock() {
            Ok(mut vaults) => {
                vaults.vault.clear();
                if let Some(dir) = &*user_data_directory {
                    let contents = std::fs::read_to_string(dir);
                    match contents {
                        Ok(s) => {
                            let v_res: Result<Vaults, Error> = toml::from_str(&s);
                            match v_res {
                                Ok(v) => {
                                    vaults.vault = v.vault;
                                    log::debug!(
                                        "Successfully loaded user config: {:?}",
                                        vaults.vault
                                    );
                                }
                                Err(e) => {
                                    log::warn!("Failed to load user config: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            log::warn!("Failed to read user data config: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to aquire mutex lock of VAULTS: {}", e);
            }
        },
        Err(e) => {
            log::error!("Failed to aquire mutex lock of USER_DATA_DIRECTORY: {}", e);
        }
    }
}

pub fn write() {
    match USER_DATA_DIRECTORY.lock() {
        Ok(user_data_directory) => {
            if let Some(dir) = &*user_data_directory {
                match VAULTS.lock() {
                    Ok(vaults) => {
                        let v_res = toml::to_string(&*vaults);
                        match v_res {
                            Ok(s) => {
                                let res = std::fs::write(dir, &s);
                                match res {
                                    Ok(_) => {
                                        log::debug!("Successfully wrote user config: {}", &s);
                                    }
                                    Err(e) => {
                                        log::error!("Failed to write user config: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to write user config: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to aquire mutex lock of VAULTS: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            log::error!("Failed to aquire mutex lock of USER_DATA_DIRECTORY: {}", e);
        }
    }
}

pub fn is_empty() -> bool {
    match VAULTS.lock() {
        Ok(v) => {
            if v.vault.is_empty() {
                return true;
            }
        }
        Err(e) => {
            log::error!("Failed to aquire mutex lock of VAULTS: {}", e);
        }
    }
    false
}
