// vault.rs
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

use crate::backend::{Backend, BackendError};
use gio::prelude::*;
use gio::subclass::prelude::*;
use gio::VolumeMonitor;
use gtk::{gio, glib};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultConfig {
    pub backend: Backend,
    pub encrypted_data_directory: String,
    pub mount_directory: String,
}

impl Clone for VaultConfig {
    fn clone(&self) -> VaultConfig {
        VaultConfig {
            backend: self.backend.clone(),
            encrypted_data_directory: self.encrypted_data_directory.clone(),
            mount_directory: self.mount_directory.clone(),
        }
    }
}

mod imp {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Vault {
        pub name: RefCell<Option<String>>,
        pub config: RefCell<Option<VaultConfig>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Vault {
        const NAME: &'static str = "Vault";
        type ParentType = glib::Object;
        type Type = super::Vault;

        fn new() -> Self {
            Self {
                name: RefCell::new(None),
                config: RefCell::new(None),
            }
        }
    }

    impl Default for Vault {
        fn default() -> Self {
            Vault {
                name: RefCell::new(None),
                config: RefCell::new(None),
            }
        }
    }

    impl ObjectImpl for Vault {}
}

glib::wrapper! {
    pub struct Vault(ObjectSubclass<imp::Vault>);
}

impl Vault {
    pub fn new(
        name: String,
        backend: Backend,
        encrypted_data_directory: String,
        mount_directory: String,
    ) -> Vault {
        let object: Self = glib::Object::new(&[]);

        let self_ = imp::Vault::from_instance(&object);

        self_.name.replace(Some(name));
        self_.config.replace(Some(VaultConfig {
            backend,
            encrypted_data_directory,
            mount_directory,
        }));

        object
    }

    pub fn new_none() -> Vault {
        let object: Self = glib::Object::new(&[]);

        object
    }

    pub fn get_name(&self) -> Option<String> {
        let self_ = imp::Vault::from_instance(&self);

        self_.name.borrow().clone()
    }

    pub fn set_name(&self, name: String) {
        let self_ = &mut imp::Vault::from_instance(&self);

        self_.name.borrow_mut().replace(name);
    }

    pub fn get_config(&self) -> Option<VaultConfig> {
        let self_ = imp::Vault::from_instance(&self);

        self_.config.borrow().clone()
    }

    pub fn set_config(&self, config: VaultConfig) {
        let self_ = &mut imp::Vault::from_instance(&self);

        self_.config.borrow_mut().replace(config);
    }

    pub fn init(&self, password: String) -> Result<(), BackendError> {
        Backend::init(&self.get_config().unwrap(), password)
    }

    pub fn unlock(&self, password: String) -> Result<(), BackendError> {
        Backend::open(&self.get_config().unwrap(), password)
    }

    pub fn lock(&self) -> Result<(), BackendError> {
        Backend::close(&self.get_config().unwrap())
    }

    pub fn is_mounted(&self) -> bool {
        let config_mount_directory = self.get_config().unwrap().mount_directory;

        let config_mount_directory_path = std::path::Path::new(&config_mount_directory);

        let config_mount_directory_file_name = config_mount_directory_path.file_name();

        if self.is_mount_hidden() {
            return self.is_mounted_all();
        }

        match config_mount_directory_file_name {
            Some(config_mount_directory_file_name) => {
                match config_mount_directory_file_name.to_str() {
                    Some(file_name) => {
                        let volume_monitor = VolumeMonitor::get();

                        for mount in volume_monitor.mounts() {
                            if mount.name() == file_name {
                                return true;
                            }
                        }
                    }
                    None => {
                        log::debug!("Could not get mount directory path");
                    }
                }
            }
            None => {
                log::debug!("Could not get config mount directory file name");
            }
        }

        false
    }

    pub fn is_mount_hidden(&self) -> bool {
        let vault_config = self.get_config().unwrap();

        let components: Vec<_> = std::path::Path::new(&vault_config.mount_directory)
            .components()
            .map(|c| c.as_os_str().to_str())
            .collect();

        components
            .iter()
            .flatten()
            .any(|c| c.starts_with(".") && !c.eq(&"..".to_string()) && c.len() > 1)
    }

    pub fn is_mounted_all(&self) -> bool {
        use proc_mounts::*;

        let mount_list = MountList::new();
        match mount_list {
            Ok(mount_list) => MountList::get_mount_by_dest(
                &mount_list,
                &self.get_config().unwrap().mount_directory,
            )
            .is_some(),
            Err(e) => {
                log::error!("Could not check if there exists any mounted vaults: {}", e);
                false
            }
        }
    }

    pub fn is_backend_available(&self) -> bool {
        if let Some(config) = self.get_config() {
            if let Ok(success) = config.backend.is_available() {
                return success;
            }
        }
        false
    }
}
