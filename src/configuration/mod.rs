/*
 * mod.rs
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

//! Configuration module for reading and writing user vault configuration files.
//!
//! This module handles the serialization and deserialization of vault configurations
//! stored in TOML format. It supports versioned configuration files and provides
//! graceful error handling for malformed or missing configuration data.

pub mod reader;
pub mod writer;

pub use reader::read_config;
pub use writer::write_config;

#[cfg(test)]
mod tests {
    use super::read_config;
    use super::write_config;
    use crate::backend::Backend;
    use crate::vault_config::VaultConfig;
    use std::collections::HashMap;
    use tempfile::TempDir;
    use uuid::Uuid;

    /// Helper function to create a test vault configuration with the given name and backend.
    fn create_test_vault(_uuid: Uuid, name: &str, backend: Backend) -> VaultConfig {
        VaultConfig {
            name: name.to_string(),
            backend: backend.into(),
            encrypted_data_directory: format!("/tmp/test/{}", name),
            mount_directory: format!("/tmp/mount/{}", name),
            session_lock: false,
        }
    }

    /// Verifies that writing multiple vaults and reading them back preserves all data correctly.
    #[test]
    fn test_multiple_vaults_round_trip() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("multiple_vaults.toml");

        // Create multiple test vaults
        let mut vaults = HashMap::new();
        let uuid1 = Uuid::parse_str("a19a941c-505a-4b46-86c0-abf39bcdc2e1").unwrap();
        let uuid2 = Uuid::parse_str("b29b941c-505a-4b46-86c0-abf39bcdc2e2").unwrap();
        let uuid3 = Uuid::parse_str("c39c941c-505a-4b46-86c0-abf39bcdc2e3").unwrap();

        vaults.insert(
            uuid1,
            create_test_vault(uuid1, "cryfs_vault", Backend::Cryfs),
        );
        vaults.insert(
            uuid2,
            create_test_vault(uuid2, "gocryptfs_vault", Backend::Gocryptfs),
        );
        vaults.insert(
            uuid3,
            create_test_vault(uuid3, "another_cryfs", Backend::Cryfs),
        );

        // Write config
        let write_result = write_config(&path, &vaults);
        assert!(write_result.is_ok());
        let write_result = write_result.unwrap();
        assert_eq!(write_result.written_count, 3);
        assert!(write_result.errors.is_empty());

        // Read back
        let read_result = read_config(&path);
        assert!(read_result.is_ok());
        let read_result = read_result.unwrap();

        assert_eq!(read_result.vaults.len(), 3);
        assert!(read_result.errors.is_empty());
        assert!(read_result.vaults.contains_key(&uuid1));
        assert!(read_result.vaults.contains_key(&uuid2));
        assert!(read_result.vaults.contains_key(&uuid3));
    }
}
