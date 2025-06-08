// uuid.rs
//
// Copyright 2025 Martin Pobaschnig
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

use crate::user_config_manager::UserConfigManager;
use uuid::Uuid;

pub fn generate_uuid() -> Uuid {
    let map = UserConfigManager::instance().get_map();
    for _ in 0..1000 {
        let uuid = Uuid::new_v4();
        if map.contains_key(&uuid) {
            log::debug!(
                "Generated UUID {} already exists, generating a new one",
                uuid
            );
            continue;
        }
        log::debug!("Generated unique UUID: {}", uuid);
        return uuid;
    }
    log::error!("Failed to generate a unique UUID after 10 attempts");
    Uuid::nil()
}
