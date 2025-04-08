// global_config_converter.rs
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

pub fn get_sem_version(current_version: &str) -> Option<(u32, u32, u32)> {
    let mut version = Vec::new();

    if current_version.contains("-") {
        let parts: Vec<&str> = current_version.split("-").collect();
        if let Some(first) = parts.first() {
            version.push(first.to_string());
        } else {
            return None;
        }
    } else {
        version.push(current_version.to_owned());
    }
    if version.len() != 1 {
        return None;
    }

    let versions: Vec<&str> = version[0].split(".").collect();
    if versions.len() != 3 {
        return None;
    }

    let major = versions[0].parse::<u32>().unwrap();
    let minor = versions[1].parse::<u32>().unwrap();
    let patch = versions[2].parse::<u32>().unwrap();

    Some((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correct_full() {
        assert_eq!(get_sem_version("0.0.0"), Some((0, 0, 0)));
        assert_eq!(get_sem_version("0.9.0"), Some((0, 9, 0)));
        assert_eq!(get_sem_version("0.10.0"), Some((0, 10, 0)));
        assert_eq!(get_sem_version("0.11.0"), Some((0, 11, 0)));
    }

    #[test]
    fn test_correct_full_dev() {
        assert_eq!(get_sem_version("0.7.0-920ce7a"), Some((0, 7, 0)));
        assert_eq!(get_sem_version("0.0.0-abcdef"), Some((0, 0, 0)));
    }

    #[test]
    fn test_incorrect() {
        assert_eq!(get_sem_version(""), None);
        assert_eq!(get_sem_version("0"), None);
        assert_eq!(get_sem_version("0.10"), None);
    }
}
