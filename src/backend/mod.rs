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

use crate::vault::Vault;

quick_error! {
    #[derive(Debug)]
    pub enum BackendError {
        NotImplemented {}
    }
}

pub trait Backend {
    fn is_available(&self) -> bool {
        log::error!("function is_available not implemented!");
        false
    }
    fn create(&self, _vault: Vault) -> Result<(), BackendError> {
        log::error!("function create not implemented!");
        Err(BackendError::NotImplemented)
    }
    fn open(&self, _vault: Vault) -> Result<(), BackendError> {
        log::error!("function open not implemented!");
        Err(BackendError::NotImplemented)
    }
    fn close(&self, _vault: Vault) -> Result<(), BackendError> {
        log::error!("function close not implemented!");
        Err(BackendError::NotImplemented)
    }
}
