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

pub struct Vault {
    pub name: String,
    pub backend: Backend,
    pub encrypted_data_directory: String,
    pub mount_directory: String,
}

impl Vault {
    pub fn init(&self) -> Result<(), BackendError> {
        return self.backend.init(self);
    }

    pub fn open(&self, _vault: Vault) -> Result<(), BackendError> {
        return self.backend.open(self);
    }

    pub fn close(&self, _vault: Vault) -> Result<(), BackendError> {
        return self.backend.close(self);
    }
}
