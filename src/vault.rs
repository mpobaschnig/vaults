// vaults.rs
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

use crate::backend::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Vault {
    pub name: String,
    pub backend: Backend,
    pub encrypted_data_directory: String,
    pub mount_directory: String,
}

impl Clone for Vault {
    fn clone(&self) -> Vault {
        Vault {
            name: self.name.clone(),
            backend: self.backend,
            encrypted_data_directory: self.encrypted_data_directory.clone(),
            mount_directory: self.mount_directory.clone(),
        }
    }
}

impl Default for Vault {
    fn default() -> Self {
        Vault {
            name: "".to_owned(),
            backend: crate::backend::Backend::Cryfs,
            encrypted_data_directory: "".to_owned(),
            mount_directory: "".to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Vaults {
    pub vault: Vec<Vault>,
}

impl Vault {
    pub fn init(&self) -> Result<(), BackendError> {
        return Backend::init(self);
    }

    pub fn open(&self, _vault: Vault) -> Result<(), BackendError> {
        return Backend::open(self);
    }

    pub fn close(&self, _vault: Vault) -> Result<(), BackendError> {
        return Backend::close(self);
    }
}
