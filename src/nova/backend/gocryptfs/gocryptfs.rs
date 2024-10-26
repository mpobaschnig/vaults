// gocryptfs.rs
//
// Copyright 2024 Martin Pobaschnig
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

use crate::nova::{backend::encryption_backend::EncryptionBackend, vault::settings::Settings};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Gocryptfs {}

#[typetag::serde]
impl EncryptionBackend for Gocryptfs {
    fn init(&self, settings: &Settings) {
        todo!()
    }

    fn mount(&self, settings: &Settings) {
        todo!()
    }

    fn unmount(&self, settings: &Settings) {
        todo!()
    }
}
