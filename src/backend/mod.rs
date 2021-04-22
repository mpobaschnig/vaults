// mod.rs
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
        Generic
    }
}

#[derive(
    Debug,
    EnumIter,
    strum_macros::ToString,
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
    fn is_available(&self) -> bool {
        match &self {
            Backend::Cryfs => {
                return cryfs::is_available();
            }
            Backend::Gocryptfs => {
                return gocryptfs::is_available();
            }
        }
    }

    pub fn init(vault_config: &VaultConfig, password: String) -> Result<(), BackendError> {
        match vault_config.backend {
            Backend::Cryfs => {
                return cryfs::init(vault_config, password);
            }
            Backend::Gocryptfs => {
                return gocryptfs::init(vault_config, password);
            }
        }
    }

    pub fn open(vault_config: &VaultConfig, password: String) -> Result<(), BackendError> {
        match vault_config.backend {
            Backend::Cryfs => {
                return cryfs::open(vault_config, password);
            }
            Backend::Gocryptfs => {
                return gocryptfs::open(vault_config, password);
            }
        }
    }

    pub fn close(vault_config: &VaultConfig) -> Result<(), BackendError> {
        match vault_config.backend {
            Backend::Cryfs => {
                return cryfs::close(vault_config);
            }
            Backend::Gocryptfs => {
                return gocryptfs::close(vault_config);
            }
        }
    }
}

pub fn probe_backends() {
    let available_backends_res = AVAILABLE_BACKENDS.lock();
    match available_backends_res {
        Ok(mut available_backends) => {
            available_backends.clear();
            for backend_enum in Backend::iter() {
                if backend_enum.is_available() {
                    let backend = backend_enum.to_string();
                    log::debug!("Found backend: {}", backend);
                    available_backends.push(backend);
                }
            }
        }
        Err(e) => {
            log::error!("Failed to aquire mutex lock of AVAILABLE_BACKENDS: {}", e);
        }
    }
}
