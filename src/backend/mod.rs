// mod.rs
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

pub mod cryfs;
pub mod gocryptfs;

use crate::vault::VaultConfig;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::string::String;
use std::sync::Mutex;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub static AVAILABLE_BACKENDS: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(vec![]));

quick_error! {
    #[derive(Debug)]
    pub enum BackendError {
        ToUser(e: String) {
            display("{}", e)
        }
        Generic {
            from(std::io::Error)
        }
    }
}

#[derive(
    Debug,
    EnumIter,
    strum_macros::Display,
    Serialize,
    Deserialize,
    strum_macros::EnumString,
    Copy,
    Clone,
)]
pub enum Backend {
    Cryfs,
    Gocryptfs,
}

impl Backend {
    pub fn is_available(&self) -> Result<bool, BackendError> {
        match &self {
            Backend::Cryfs => cryfs::is_available(),
            Backend::Gocryptfs => gocryptfs::is_available(),
        }
    }

    pub fn init(vault_config: &VaultConfig, password: String) -> Result<(), BackendError> {
        match vault_config.backend {
            Backend::Cryfs => cryfs::init(vault_config, password),
            Backend::Gocryptfs => gocryptfs::init(vault_config, password),
        }
    }

    pub fn open(vault_config: &VaultConfig, password: String) -> Result<(), BackendError> {
        match vault_config.backend {
            Backend::Cryfs => cryfs::open(vault_config, password),
            Backend::Gocryptfs => gocryptfs::open(vault_config, password),
        }
    }

    pub fn close(vault_config: &VaultConfig) -> Result<(), BackendError> {
        match vault_config.backend {
            Backend::Cryfs => cryfs::close(vault_config),
            Backend::Gocryptfs => gocryptfs::close(vault_config),
        }
    }

    pub fn get_ui_string(&self) -> String {
        match self {
            Backend::Cryfs => String::from("CryFS"),
            Backend::Gocryptfs => String::from("gocryptfs"),
        }
    }
}

pub fn probe_backends() {
    if let Ok(mut available_backends) = AVAILABLE_BACKENDS.lock() {
        available_backends.clear();

        for backend in Backend::iter() {
            if let Ok(success) = backend.is_available() {
                if success {
                    available_backends.push(backend.get_ui_string());
                }
            }
        }
    }
}
