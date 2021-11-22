// global_config_manager.rs
//
// Copyright 2021 Martin Pobaschnig
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

use gtk::{
    gio::subclass::prelude::*,
    glib::{self, home_dir, user_config_dir, user_data_dir},
};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use toml::de::Error;

use self::imp::GlobalConfig;

static mut GLOBAL_CONFIG_MANAGER: Option<GlobalConfigManager> = None;

mod imp {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct GlobalConfig {
        pub encrypted_data_directory: RefCell<String>,
        pub mount_directory: RefCell<String>,
    }

    impl Clone for GlobalConfig {
        fn clone(&self) -> GlobalConfig {
            GlobalConfig {
                encrypted_data_directory: self.encrypted_data_directory.clone(),
                mount_directory: self.mount_directory.clone(),
            }
        }
    }

    #[derive(Debug)]
    pub struct GlobalConfigManager {
        pub user_config_directory: RefCell<Option<String>>,

        pub global_config: RefCell<GlobalConfig>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GlobalConfigManager {
        const NAME: &'static str = "GlobalConfigManager";
        type ParentType = glib::Object;
        type Type = super::GlobalConfigManager;

        fn new() -> Self {
            Self {
                user_config_directory: RefCell::new(None),
                global_config: RefCell::new(GlobalConfig {
                    encrypted_data_directory: RefCell::new(String::from("".to_string())),
                    mount_directory: RefCell::new(String::from("".to_string())),
                }),
            }
        }
    }

    impl ObjectImpl for GlobalConfigManager {}
}

glib::wrapper! {
    pub struct GlobalConfigManager(ObjectSubclass<imp::GlobalConfigManager>);
}

impl GlobalConfigManager {
    pub fn instance() -> Self {
        unsafe {
            match GLOBAL_CONFIG_MANAGER.as_ref() {
                Some(user_config) => user_config.clone(),
                None => {
                    let user_config = GlobalConfigManager::new();
                    GLOBAL_CONFIG_MANAGER = Some(user_config.clone());
                    user_config
                }
            }
        }
    }

    fn new() -> Self {
        let object: Self = glib::Object::new(&[]).expect("Failed to create GlobalConfigManager");

        match user_config_dir().as_os_str().to_str() {
            Some(user_config_directory) => {
                log::debug!("Got user data dir: {}", user_config_directory);

                let self_ = &mut imp::GlobalConfigManager::from_instance(&object);
                *self_.user_config_directory.borrow_mut() =
                    Some(user_config_directory.to_owned() + "/global_config.toml");
            }
            None => {
                log::error!("Could not get user data directory");
            }
        }

        object
    }

    pub fn read_config(&self) {
        let self_ = &mut imp::GlobalConfigManager::from_instance(&self);

        if let Some(path) = self_.user_config_directory.borrow().as_ref() {
            let global_config = &mut *self_.global_config.borrow_mut();

            let contents = std::fs::read_to_string(path);
            match contents {
                Ok(content) => {
                    let res: Result<GlobalConfig, Error> = toml::from_str(&content.clone());
                    match res {
                        Ok(v) => {
                            *global_config = v;
                        }
                        Err(e) => {
                            log::error!("Failed to parse user data config: {}", e);
                        }
                    }
                }
                Err(e) => {
                    log::error!(
                        "Failed to read user data config: {}. Falling back to default directories.",
                        e
                    );

                    match user_data_dir().as_os_str().to_str() {
                        Some(user_data_directory) => {
                            log::debug!("Got user data directory: {}", user_data_directory);

                            *global_config.encrypted_data_directory.borrow_mut() =
                                user_data_directory.to_owned() + "/";
                        }
                        None => {
                            log::error!("Could not get user data directory");
                        }
                    }

                    match home_dir().to_str() {
                        Some(home_directory) => {
                            log::debug!("Got home directory: {}", &home_directory);

                            *global_config.mount_directory.borrow_mut() =
                                home_directory.to_owned() + "/Vaults/";
                        }
                        None => {
                            log::error!("Could not get home directory");
                        }
                    }

                    match toml::to_string_pretty(&global_config) {
                        Ok(contents) => match std::fs::write(path, &contents) {
                            Ok(_) => {
                                log::debug!("Successfully wrote user config: {}", &contents);
                            }
                            Err(e) => {
                                log::error!("Failed to write user config: {}", e);
                            }
                        },
                        Err(e) => {
                            log::error!("Failed to parse config: {}", e);
                        }
                    }
                }
            }
        }
    }

    pub fn write_config(&self) {
        let self_ = &mut imp::GlobalConfigManager::from_instance(&self);

        if let Some(path) = self_.user_config_directory.borrow().as_ref() {
            match toml::to_string_pretty(&self_.global_config.borrow().clone()) {
                Ok(contents) => match std::fs::write(path, &contents) {
                    Ok(_) => {
                        log::debug!("Successfully wrote user config: {}", &contents);
                    }
                    Err(e) => {
                        log::error!("Failed to write user config: {}", e);
                    }
                },
                Err(e) => {
                    log::error!("Failed to parse config: {}", e);
                }
            }
        }
    }

    pub fn get_global_config(&self) -> GlobalConfig {
        let self_ = &mut imp::GlobalConfigManager::from_instance(&self);

        self_.global_config.borrow().clone()
    }

    pub fn set_encrypted_data_directory(&self, path: String) {
        let self_ = &mut imp::GlobalConfigManager::from_instance(&self);

        *self_
            .global_config
            .borrow_mut()
            .encrypted_data_directory
            .borrow_mut() = path;
    }

    pub fn set_mount_directory(&self, path: String) {
        let self_ = &mut imp::GlobalConfigManager::from_instance(&self);

        *self_
            .global_config
            .borrow_mut()
            .mount_directory
            .borrow_mut() = path;
    }
}
