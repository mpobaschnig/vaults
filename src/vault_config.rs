/*
 * vault_config.rs
 *
 * Copyright 2026 Martin Pobaschnig
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use crate::backend::backend::{BackendImpl, serde_backend};
use gtk::glib;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Clone, Serialize, glib::Boxed)]
#[boxed_type(name = "VaultConfig")]
pub struct VaultConfig {
    pub name: String,
    #[serde(with = "serde_backend")]
    pub backend: Box<dyn BackendImpl>,
    pub encrypted_data_directory: String,
    pub mount_directory: String,
    pub session_lock: bool,
}
