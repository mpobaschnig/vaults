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

use crate::{legacy, vault::*};
use gtk::glib::Properties;
use gtk::{
    gio::subclass::prelude::*,
    glib::{self, prelude::*, subclass::Signal, user_config_dir},
};
use once_cell::sync::Lazy;
use std::{cell::RefCell, collections::HashMap};
use toml::de::Error;
use uuid::Uuid;

static mut USER_CONFIG_MANAGER: Option<UserConfigManager> = None;

mod imp {
    use super::*;

    #[derive(Debug, Properties)]
    #[properties(wrapper_type = super::UserConfigManager)]
    pub struct UserConfigManager {
        pub vaults: RefCell<HashMap<Uuid, VaultConfig>>,
        pub user_config_directory: RefCell<Option<String>>,
        #[property(name = "has-vaults", default = false, get, set)]
        pub has_vaults: RefCell<bool>,
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
                has_vaults: RefCell::new(false),
            }
        }
    }

    #[glib::derived_properties]
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
                    Some(user_config_directory.to_owned() + "/user_config_v2.toml");
            }
            None => {
                log::error!("Could not get user data directory");
            }
        }

        object
    }

    pub fn get_map(&self) -> HashMap<Uuid, VaultConfig> {
        log::trace!("get_map()");

        self.imp().vaults.borrow().clone()
    }

    pub fn read_config(&self) {
        log::trace!("read_config()");

        if let Some(path) = self.imp().user_config_directory.borrow().as_ref() {
            if !std::path::Path::new(path).exists() {
                log::info!("User config file does not exist: {}", path);
                log::info!("Trying to read legacy user config...");
                let user_config = legacy::user_config::get_user_config_from_legacy_config();
                let map = &mut *self.imp().vaults.borrow_mut();
                map.clear();
                *map = user_config;
                return;
            }

            let map = &mut *self.imp().vaults.borrow_mut();
            map.clear();

            let contents = std::fs::read_to_string(path);
            match contents {
                Ok(content) => {
                    let res: Result<HashMap<Uuid, VaultConfig>, Error> =
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

            self.set_has_vaults(!map.is_empty());
        }
    }

    pub fn write_config(&self, map: &mut HashMap<Uuid, VaultConfig>) {
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

    pub fn add_vault(&self, vault: Vault) {
        log::debug!("Add vault: {:?}, {:?}", &vault.name(), &vault.config());

        #[allow(unused_assignments)]
        let mut is_map_empty = false;
        {
            let map = &mut self.imp().vaults.borrow_mut();
            map.insert(vault.get_uuid(), vault.config());
            self.write_config(map);
            is_map_empty = map.is_empty();
        };
        self.set_has_vaults(!is_map_empty);

        self.emit_by_name::<()>("add-vault", &[]);
    }

    pub fn remove_vault(self, uuid: Uuid) {
        log::trace!("remove_vault({:?})", &uuid);

        #[allow(unused_assignments)]
        let mut is_map_empty = false;
        {
            let map = &mut self.imp().vaults.borrow_mut();
            map.remove(&uuid);
            self.write_config(map);
            is_map_empty = map.is_empty();
        }
        self.set_has_vaults(!is_map_empty);

        self.emit_by_name::<()>("remove-vault", &[]);
        self.emit_by_name::<()>("refresh", &[&is_map_empty]);
    }

    pub fn change_vault(&self, uuid: Uuid, new_vault_config: VaultConfig) {
        log::trace!("change_vault({:?}, {:?})", &uuid, &new_vault_config);

        let map = &mut self.imp().vaults.borrow_mut();
        map.insert(uuid, new_vault_config);
        self.write_config(map);

        self.emit_by_name::<()>("change-vault", &[]);
    }
}
