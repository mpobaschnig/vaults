// gocryptfs.rs
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

use crate::vault::Vault;

use super::BackendError;
use std::process::Command;

pub fn is_available() -> bool {
    let output_res = Command::new("gocryptfs").arg("--version").output();

    match output_res {
        Ok(output) => {
            if output.status.success() {
                return true;
            }
        }
        Err(e) => {
            log::error!("Failed to probe gocryptfs: {}", e);
        }
    }

    false
}

pub fn init(vault: &Vault) -> Result<(), BackendError> {
    Err(BackendError::NotImplemented)
}

pub fn open(vault: &Vault) -> Result<(), BackendError> {
    Err(BackendError::NotImplemented)
}

pub fn close(vault: &Vault) -> Result<(), BackendError> {
    Err(BackendError::NotImplemented)
}
