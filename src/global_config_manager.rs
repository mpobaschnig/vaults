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
use ini::Ini;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, fs, process::ExitStatus};
use toml::de::Error;

use self::imp::GlobalConfig;

static mut GLOBAL_CONFIG_MANAGER: Option<GlobalConfigManager> = None;

mod imp {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct GlobalConfig {
        pub encrypted_data_directory: RefCell<Option<String>>,
        pub mount_directory: RefCell<Option<String>>,
        pub gocryptfs_custom_binary: RefCell<Option<bool>>,
        pub gocryptfs_custom_binary_path: RefCell<Option<String>>,
        pub cryfs_custom_binary: RefCell<Option<bool>>,
        pub cryfs_custom_binary_path: RefCell<Option<String>>,
    }

    impl Clone for GlobalConfig {
        fn clone(&self) -> GlobalConfig {
            GlobalConfig {
                encrypted_data_directory: self.encrypted_data_directory.clone(),
                mount_directory: self.mount_directory.clone(),
                gocryptfs_custom_binary: self.gocryptfs_custom_binary.clone(),
                gocryptfs_custom_binary_path: self.gocryptfs_custom_binary_path.clone(),
                cryfs_custom_binary: self.cryfs_custom_binary.clone(),
                cryfs_custom_binary_path: self.cryfs_custom_binary_path.clone(),
            }
        }
    }

    #[derive(Debug)]
    pub struct GlobalConfigManager {
        pub user_config_directory: RefCell<Option<String>>,

        pub global_config: RefCell<GlobalConfig>,
        pub flatpak_info: RefCell<Ini>,
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
                    encrypted_data_directory: RefCell::new(Some("".to_string())),
                    mount_directory: RefCell::new(Some("".to_string())),
                    cryfs_custom_binary: RefCell::new(Some(false)),
                    cryfs_custom_binary_path: RefCell::new(Some("".to_string())),
                    gocryptfs_custom_binary: RefCell::new(Some(false)),
                    gocryptfs_custom_binary_path: RefCell::new(Some("".to_string())),
                }),
                flatpak_info: RefCell::new(Ini::new()),
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
            #[allow(static_mut_refs)]
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
        log::trace!("new()");

        let object: Self = glib::Object::new();

        *object.imp().flatpak_info.borrow_mut() =
            Ini::load_from_file("/.flatpak-info").expect("Could not load .flatpak-info");

        match user_config_dir().as_os_str().to_str() {
            Some(user_config_directory) => {
                log::info!("Got user data dir: {}", user_config_directory);

                *object.imp().user_config_directory.borrow_mut() =
                    Some(user_config_directory.to_owned() + "/global_config.toml");
            }
            None => {
                log::error!("Could not get user data directory");
            }
        }

        object
    }

    pub fn read_config(&self) {
        log::trace!("read_config()");

        if let Some(path) = self.imp().user_config_directory.borrow().as_ref() {
            let global_config = &mut *self.imp().global_config.borrow_mut();

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
                            log::info!("Got user data directory: {}", user_data_directory);

                            *global_config.encrypted_data_directory.borrow_mut() =
                                Some(user_data_directory.to_owned() + "/");
                        }
                        None => {
                            log::error!("Could not get user data directory");
                        }
                    }

                    match home_dir().to_str() {
                        Some(home_directory) => {
                            log::debug!("Got home directory: {}", &home_directory);

                            *global_config.mount_directory.borrow_mut() =
                                Some(home_directory.to_owned() + "/Vaults/");
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
        log::trace!("write_config()");

        if let Some(path) = self.imp().user_config_directory.borrow().as_ref() {
            match toml::to_string_pretty(&self.imp().global_config.borrow().clone()) {
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
        log::trace!("get_global_config()");

        self.imp().global_config.borrow().clone()
    }

    pub fn set_encrypted_data_directory(&self, path: String) {
        log::trace!("set_encrypted_data_directory({})", path);

        *self
            .imp()
            .global_config
            .borrow_mut()
            .encrypted_data_directory
            .borrow_mut() = Some(path);
    }

    pub fn set_mount_directory(&self, path: String) {
        log::trace!("set_mount_directory({})", path);

        *self
            .imp()
            .global_config
            .borrow_mut()
            .mount_directory
            .borrow_mut() = Some(path);
    }

    pub fn get_flatpak_info(&self) -> Ini {
        log::trace!("get_flatpak_info()");

        self.imp().flatpak_info.borrow().clone()
    }

    pub fn cryfs_custom_binary(&self) -> bool {
        log::trace!("cryfs_custom_binary()");

        (*self
            .imp()
            .global_config
            .borrow()
            .cryfs_custom_binary
            .borrow())
        .unwrap_or(false)
    }

    pub fn set_cryfs_custom_binary(&self, enabled: bool) {
        log::trace!("set_cryfs_custom_binary({})", enabled);

        *self
            .imp()
            .global_config
            .borrow_mut()
            .cryfs_custom_binary
            .borrow_mut() = Some(enabled);
    }

    pub fn cryfs_custom_binary_path(&self) -> String {
        log::trace!("cryfs_custom_binary_path()");

        self.imp()
            .global_config
            .borrow()
            .cryfs_custom_binary_path
            .borrow()
            .clone()
            .unwrap()
    }

    pub fn set_cryfs_custom_binary_path(&self, path: String) {
        log::trace!("set_cryfs_custom_binary_path({})", path);

        *self
            .imp()
            .global_config
            .borrow_mut()
            .cryfs_custom_binary_path
            .borrow_mut() = Some(path);
    }

    pub fn gocryptfs_custom_binary(&self) -> bool {
        log::trace!("gocryptfs_custom_binary()");

        (*self
            .imp()
            .global_config
            .borrow()
            .gocryptfs_custom_binary
            .borrow())
        .unwrap_or(false)
    }

    pub fn set_gocryptfs_custom_binary(&self, enabled: bool) {
        log::trace!("set_gocryptfs_custom_binary({})", enabled);

        *self
            .imp()
            .global_config
            .borrow_mut()
            .gocryptfs_custom_binary
            .borrow_mut() = Some(enabled);
    }

    pub fn gocryptfs_custom_binary_path(&self) -> String {
        log::trace!("gocryptfs_custom_binary_path()");

        self.imp()
            .global_config
            .borrow()
            .gocryptfs_custom_binary_path
            .borrow()
            .clone()
            .unwrap()
    }

    pub fn set_gocryptfs_custom_binary_path(&self, path: String) {
        log::trace!("set_gocrypfs_custom_binary_path({})", path);

        *self
            .imp()
            .global_config
            .borrow_mut()
            .gocryptfs_custom_binary_path
            .borrow_mut() = Some(path);
    }

    pub fn get_cryfs_binary_path(&self) -> String {
        let flatpak_info = self.get_flatpak_info();
        let instance_path = flatpak_info
            .section(Some("Instance"))
            .unwrap()
            .get("app-path")
            .unwrap();
        let cryfs_instance_path = instance_path.to_owned() + "/bin/cryfs";
        log::info!("CryFS binary path: {}", cryfs_instance_path);
        cryfs_instance_path
    }

    pub fn get_gocryptfs_binary_path(&self) -> Option<String> {
        let flatpak_info = self.get_flatpak_info();
        let instance_path = flatpak_info.section(Some("Instance"))?.get("app-path")?;
        let gocryptfs_instance_path = instance_path.to_owned() + "/bin/gocryptfs";
        log::info!("gocryptfs binary path: {}", gocryptfs_instance_path);
        let exists = fs::exists(&gocryptfs_instance_path);
        match exists {
            Ok(exists) => {
                if exists {
                    log::info!("gocryptfs binary exists, taking it");
                    return Some(gocryptfs_instance_path);
                }
            }
            Err(e) => {
                log::error!("Error checking gocryptfs binary path existence: {}", e);
            }
        }

        None
    }
}
