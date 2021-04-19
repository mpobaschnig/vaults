// vault.rs
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

use crate::backend::{Backend, BackendError};
use gio::subclass::prelude::*;
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

    impl Clone for Vault {
        fn clone(&self) -> Vault {
            Vault {
                name: self.name.clone(),
                config: self.config.clone(),
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
        let o: Self = glib::Object::new(&[]).expect("Failed to create UserConfig");

        let self_ = imp::Vault::from_instance(&o);

        self_.name.replace(Some(name));
        self_.config.replace(Some(VaultConfig {
            backend,
            encrypted_data_directory,
            mount_directory,
        }));

        o
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

    pub fn new_none() -> Vault {
        let o: Self = glib::Object::new(&[]).expect("Failed to create UserConfig");

        o
    }

    pub fn init(&self, password: String) -> Result<(), BackendError> {
        log::debug!("Init vault!");
        Backend::init(&self.get_config().unwrap(), password)
    }

    pub fn unlock(&self, password: String) -> Result<(), BackendError> {
        log::debug!("Unlock vault!");
        Backend::open(&self.get_config().unwrap(), password)
    }

    pub fn lock(&self) -> Result<(), BackendError> {
        log::debug!("Lock vault!");
        Backend::close(&self.get_config().unwrap())
    }

    pub fn is_mounted(&self) -> bool {
        use proc_mounts::*;

        let mount_list = MountList::new();
        match mount_list {
            Ok(mount_list) => {
                let is_mounted = MountList::get_mount_by_dest(
                    &mount_list,
                    self.get_config().unwrap().mount_directory,
                );

                match is_mounted {
                    Some(_) => true,
                    None => false,
                }
            }
            Err(e) => {
                log::error!("Could not check if mounted: {}", e);
                return false;
            }
        }
    }
}
