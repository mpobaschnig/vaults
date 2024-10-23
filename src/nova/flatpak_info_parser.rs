// flatpak_info_parser.rs
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

extern crate ini;

use ini::Ini;

pub fn get_instance_path() -> Option<String> {
    match Ini::load_from_file("/.flatpak-info") {
        Ok(conf) => match conf.section(Some("Instance")) {
            Some(section) => match section.get("instance-path") {
                Some(instance_path) => {
                    log::info!("Found instance path: {}", instance_path);

                    return Some(instance_path.to_owned());
                }
                None => {
                    log::error!(
                        "instance-path was not found. Flatpak binary paths are not available"
                    );
                    return None;
                }
            },
            None => {
                log::error!("[Instance] is not available. Flatpak binary paths are not available");
                return None;
            }
        },
        Err(e) => {
            log::error!(
                ".flatpak-info was not found. Flatpak binary paths are not available. Error: {}",
                e
            );
            return None;
        }
    }
}
