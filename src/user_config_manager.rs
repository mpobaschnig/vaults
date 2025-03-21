// user_config_manager.rs
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

use crate::vault::*;
use gtk::{
    gio::subclass::prelude::*,
    glib::{self, prelude::*, subclass::Signal, user_config_dir},
};
use once_cell::sync::Lazy;
use std::{cell::RefCell, collections::HashMap};
use toml::de::Error;

static mut USER_CONFIG_MANAGER: Option<UserConfigManager> = None;

mod imp {
    use super::*;

    #[derive(Debug)]
    pub struct UserConfigManager {
        pub vaults: RefCell<HashMap<String, VaultConfig>>,
        pub user_config_directory: RefCell<Option<String>>,

        pub current_vault: RefCell<Option<Vault>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for UserConfigManager {
        const NAME: &'static str = "UserConfigManager";
        type ParentType = glib::Object;
        type Type = super::UserConfigManager;

        fn new() -> Self {
            Self {
                vaults: RefCell::new(HashMap::new()),
                user_config_directory: RefCell::new(None),
                current_vault: RefCell::new(None),
            }
        }
    }

    impl ObjectImpl for UserConfigManager {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("refresh")
                        .param_types([bool::static_type()])
                        .build(),
                    Signal::builder("add-vault").build(),
                    Signal::builder("remove-vault").build(),
                    Signal::builder("change-vault").build(),
                ]
            });

            SIGNALS.as_ref()
        }
    }
}

glib::wrapper! {
    pub struct UserConfigManager(ObjectSubclass<imp::UserConfigManager>);
}

impl UserConfigManager {
    pub fn connect_refresh<F: Fn(bool) + 'static>(&self, callback: F) -> glib::SignalHandlerId {
        self.connect_local("refresh", false, move |args| {
            let map_is_empty = args.get(1).unwrap().get::<bool>().unwrap();
            callback(map_is_empty);
            None
        })
    }

    pub fn connect_add_vault<F: Fn() + 'static>(&self, callback: F) -> glib::SignalHandlerId {
        self.connect_local("add-vault", false, move |_| {
            callback();
            None
        })
    }

    pub fn connect_remove_vault<F: Fn() + 'static>(&self, callback: F) -> glib::SignalHandlerId {
        self.connect_local("remove-vault", false, move |_| {
            callback();
            None
        })
    }

    pub fn connect_change_vault<F: Fn() + 'static>(&self, callback: F) -> glib::SignalHandlerId {
        self.connect_local("change-vault", false, move |_| {
            callback();
            None
        })
    }

    pub fn instance() -> Self {
        unsafe {
            #[allow(static_mut_refs)]
            match USER_CONFIG_MANAGER.as_ref() {
                Some(user_config) => user_config.clone(),
                None => {
                    let user_config = UserConfigManager::new();
                    USER_CONFIG_MANAGER = Some(user_config.clone());
                    user_config
                }
            }
        }
    }

    fn new() -> Self {
        log::trace!("new()");

        let object: Self = glib::Object::new();

        match user_config_dir().as_os_str().to_str() {
            Some(user_config_directory) => {
                log::info!("Got user config dir: {}", user_config_directory);

                *object.imp().user_config_directory.borrow_mut() =
                    Some(user_config_directory.to_owned() + "/user_config.toml");
            }
            None => {
                log::error!("Could not get user data directory");
            }
        }

        object
    }

    pub fn get_map(&self) -> HashMap<String, VaultConfig> {
        log::trace!("get_map()");

        self.imp().vaults.borrow().clone()
    }

    pub fn read_config(&self) {
        log::trace!("read_config()");

        if let Some(path) = self.imp().user_config_directory.borrow().as_ref() {
            let map = &mut *self.imp().vaults.borrow_mut();

            map.clear();

            let contents = std::fs::read_to_string(path);
            match contents {
                Ok(content) => {
                    let res: Result<HashMap<String, VaultConfig>, Error> =
                        toml::from_str(&content.clone());
                    match res {
                        Ok(v) => {
                            *map = v;
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
    }

    pub fn write_config(&self, map: &mut HashMap<String, VaultConfig>) {
        log::trace!("write_config({:?})", &map);

        if let Some(path) = self.imp().user_config_directory.borrow().as_ref() {
            match toml::to_string_pretty(&map) {
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

    pub fn get_current_vault(&self) -> Option<Vault> {
        log::trace!("get_current_vault()");

        self.imp().current_vault.borrow().clone()
    }

    pub fn set_current_vault(&self, vault: Vault) {
        log::trace!("set_current_vault({:?})", &vault);

        self.imp().current_vault.borrow_mut().replace(vault);
    }

    pub fn add_vault(&self, vault: Vault) {
        log::trace!("add_vault({:?})", &vault);

        let map = &mut self.imp().vaults.borrow_mut();
        match (vault.get_name(), vault.get_config()) {
            (Some(name), Some(config)) => {
                log::debug!("Add vault: {:?}, {:?}", &name, &config);

                *self.imp().current_vault.borrow_mut() = Some(vault.clone());
                map.insert(name, config);

                self.write_config(map);

                self.emit_by_name::<()>("add-vault", &[]);
            }
            (_, _) => {
                log::error!("Vault not initialised!");
            }
        }
    }

    pub fn remove_vault(self, vault: Vault) {
        log::trace!("remove_vault({:?})", &vault);

        let map = &mut self.imp().vaults.borrow_mut();
        match vault.get_name() {
            Some(name) => {
                log::debug!("Remove vault: {:?}", &name);

                *self.imp().current_vault.borrow_mut() = Some(vault.clone());

                map.remove(&name);

                self.write_config(map);

                self.emit_by_name::<()>("remove-vault", &[]);
                self.emit_by_name::<()>("refresh", &[&map.is_empty()]);
            }
            None => {
                log::error!("Vault not initialised!");
            }
        }
    }

    pub fn change_vault(&self, old_vault: Vault, new_vault: Vault) {
        log::trace!("change_vault({:?}, {:?})", &old_vault, &new_vault);

        match (
            old_vault.get_name(),
            new_vault.get_name(),
            new_vault.get_config(),
        ) {
            (Some(old_name), Some(new_name), Some(config)) => {
                log::debug!(
                    "Change vault: {:?}, {:?}, {:?}",
                    &old_name,
                    &new_name,
                    &config
                );

                let map = &mut self.imp().vaults.borrow_mut();

                map.remove(&old_name);

                map.insert(new_name, config);

                self.write_config(map);

                *self.imp().current_vault.borrow_mut() = Some(new_vault);

                self.emit_by_name::<()>("change-vault", &[]);
            }
            (_, _, _) => {
                log::error!("Vault(s) not initialised!");
            }
        }
    }
}
