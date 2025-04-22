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
use gio::VolumeMonitor;
use gio::prelude::*;
use gio::subclass::prelude::*;
use gtk::{gio, glib};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VaultConfig {
    pub backend: Backend,
    pub encrypted_data_directory: String,
    pub mount_directory: String,
    pub session_lock: Option<bool>,
    pub use_custom_binary: Option<bool>,
    pub custom_binary_path: Option<String>,
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
        session_lock: Option<bool>,
        use_custom_binary: Option<bool>,
        custom_binary_path: Option<String>,
    ) -> Vault {
        let object: Self = glib::Object::new();

        object.imp().name.replace(Some(name));
        object.imp().config.replace(Some(VaultConfig {
            backend,
            encrypted_data_directory,
            mount_directory,
            session_lock,
            use_custom_binary,
            custom_binary_path,
        }));

        object
    }

    pub fn new_none() -> Vault {
        let object: Self = glib::Object::new();

        object
    }

    pub fn get_name(&self) -> Option<String> {
        log::trace!("get_name");
        let name = self.imp().name.borrow().clone();
        log::debug!("Name: {:?}", name);
        name
    }

    pub fn set_name(&self, name: String) {
        log::trace!("set_name({})", name);
        self.imp().name.borrow_mut().replace(name);
    }

    pub fn get_config(&self) -> Option<VaultConfig> {
        log::trace!("get_config");
        let config = self.imp().config.borrow().clone();
        log::debug!("Config: {:?}", config);
        config
    }

    pub fn set_config(&self, config: VaultConfig) {
        log::trace!("set_config({:?})", config);
        self.imp().config.borrow_mut().replace(config);
    }

    pub fn init(&self, password: String) -> Result<(), BackendError> {
        log::trace!("init(password: <redacted>)");
        Backend::init(&self.get_config().unwrap(), password)
    }

    pub fn unlock(&self, password: String) -> Result<(), BackendError> {
        log::trace!("unlock(password: <redacted>)");
        Backend::open(&self.get_config().unwrap(), password)
    }

    pub fn lock(&self) -> Result<(), BackendError> {
        log::trace!("lock");
        Backend::close(&self.get_config().unwrap())
    }

    pub fn is_mounted(&self) -> bool {
        log::trace!("is_mounted");

        let config_mount_directory = self.get_config().unwrap().mount_directory;

        if self.is_mount_hidden() {
            log::debug!("Vault is hidden");
            let is_vault_mounted_all = self.is_mounted_all();
            log::debug!("Vault is mounted (all): {}", is_vault_mounted_all);
            return is_vault_mounted_all;
        }

        let canon_config_path = std::path::Path::new(&config_mount_directory)
            .canonicalize()
            .ok();

        if let Some(canon_config_path) = canon_config_path {
            log::info!(
                "Opening canonical path: {}",
                &canon_config_path.as_os_str().to_str().unwrap()
            );
            for mount in VolumeMonitor::get().mounts() {
                let is_configured_mount = mount
                    .default_location()
                    .path()
                    .map(|mount_path| std::path::Path::canonicalize(&mount_path))
                    .and_then(Result::ok)
                    .map(|canon_mount_path| canon_mount_path == canon_config_path)
                    .unwrap_or(false);
                if is_configured_mount {
                    return true;
                }
            }
        } else {
            log::error!("Could not get canonical mount directory path");
        }

        log::debug!("Vault is not mounted");
        false
    }

    pub fn is_mount_hidden(&self) -> bool {
        log::trace!("is_mount_hidden");

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
        log::trace!("is_mounted_all");

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
        log::trace!("is_backend_available");
        if let Some(config) = self.get_config() {
            if let Ok(success) = config.backend.is_available(&config) {
                return success;
            }
        }
        false
    }

    pub fn delete_encrypted_data(&self) -> std::io::Result<()> {
        log::trace!("delete_encrypted_data");

        if let Some(config) = self.get_config() {
            let path = std::path::Path::new(&config.encrypted_data_directory);
            log::debug!("Deleting encrypted data directory: {:?}", path);
            return std::fs::remove_dir_all(path);
        }

        log::error!("Could not get config");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_hidden_paths() {
        let vault = Vault::new(
            "".to_string(),
            Backend::Gocryptfs,
            "".to_string(),
            "".to_string(),
            None,
            None,
            None,
        );
        assert!(!vault.is_mount_hidden());

        vault.set_config(VaultConfig {
            backend: Backend::Gocryptfs,
            encrypted_data_directory: "".to_string(),
            mount_directory: ".".to_string(),
            session_lock: None,
            use_custom_binary: None,
            custom_binary_path: None,
        });
        assert!(!vault.is_mount_hidden());

        vault.set_config(VaultConfig {
            backend: Backend::Gocryptfs,
            encrypted_data_directory: "".to_string(),
            mount_directory: "..".to_string(),
            session_lock: None,
            use_custom_binary: None,
            custom_binary_path: None,
        });
        assert!(!vault.is_mount_hidden());

        vault.set_config(VaultConfig {
            backend: Backend::Gocryptfs,
            encrypted_data_directory: "".to_string(),
            mount_directory: "./".to_string(),
            session_lock: None,
            use_custom_binary: None,
            custom_binary_path: None,
        });
        assert!(!vault.is_mount_hidden());

        vault.set_config(VaultConfig {
            backend: Backend::Gocryptfs,
            encrypted_data_directory: "".to_string(),
            mount_directory: "./Hidden".to_string(),
            session_lock: None,
            use_custom_binary: None,
            custom_binary_path: None,
        });
        assert!(!vault.is_mount_hidden());

        vault.set_config(VaultConfig {
            backend: Backend::Gocryptfs,
            encrypted_data_directory: "".to_string(),
            mount_directory: "Test/.Test".to_string(),
            session_lock: None,
            use_custom_binary: None,
            custom_binary_path: None,
        });
        assert!(vault.is_mount_hidden());

        vault.set_config(VaultConfig {
            backend: Backend::Gocryptfs,
            encrypted_data_directory: "".to_string(),
            mount_directory: "./Test/.Test".to_string(),
            session_lock: None,
            use_custom_binary: None,
            custom_binary_path: None,
        });
        assert!(vault.is_mount_hidden());

        vault.set_config(VaultConfig {
            backend: Backend::Gocryptfs,
            encrypted_data_directory: "".to_string(),
            mount_directory: "../.Test".to_string(),
            session_lock: None,
            use_custom_binary: None,
            custom_binary_path: None,
        });
        assert!(vault.is_mount_hidden());
    }
}
