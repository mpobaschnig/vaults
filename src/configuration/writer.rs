/*
 * writer.rs
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

//! Writes vault configurations to a TOML file.
//!
//! Each vault is serialized individually so that one problematic vault does
//! not prevent the others from being saved.  Errors are collected and
//! returned alongside the count of successfully written vaults.

use crate::vault_config::VaultConfig;
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

/// Describes what went wrong while writing a configuration file.
#[derive(Debug)]
#[allow(dead_code)]
pub enum WriteError {
    /// The file could not be written to disk.
    Io(std::io::Error),
    /// A single vault entry could not be serialized.
    VaultSerialize { uuid: Uuid, error: String },
}

/// The result of a [write_config] call.
///
/// `written_count` is the number of vaults successfully serialized.  `errors`
/// lists any problems encountered; it may be non-empty even when some vaults
/// were written.
#[derive(Debug)]
pub struct WriteResult {
    pub written_count: usize,
    pub errors: Vec<WriteError>,
}

/// Write all vault configurations in `vaults` to the TOML file at `path`.
///
/// Each vault is serialized individually.  If one vault cannot be serialized,
/// the others are still written.  The file is atomically updated by writing
/// to a temporary file first and then renaming it.
pub fn write_config(
    path: &Path,
    vaults: &HashMap<Uuid, VaultConfig>,
) -> Result<WriteResult, WriteError> {
    let mut toml_map = toml::map::Map::new();
    let mut errors = Vec::new();
    let mut written_count = 0usize;

    // Serialize each vault individually
    for (uuid, config) in vaults {
        match toml::to_string_pretty(config) {
            Ok(serialized) => {
                // Parse the serialized vault back into a toml::Value to insert into the map
                match toml::from_str::<toml::Value>(&serialized) {
                    Ok(value) => {
                        toml_map.insert(uuid.to_string(), value);
                        written_count += 1;
                    }
                    Err(e) => {
                        log::warn!(
                            "Failed to serialize vault '{}': {}. Skipping this entry.",
                            uuid,
                            e
                        );
                        errors.push(WriteError::VaultSerialize {
                            uuid: *uuid,
                            error: e.to_string(),
                        });
                    }
                }
            }
            Err(e) => {
                log::warn!(
                    "Failed to serialize vault '{}': {}. Skipping this entry.",
                    uuid,
                    e
                );
                errors.push(WriteError::VaultSerialize {
                    uuid: *uuid,
                    error: e.to_string(),
                });
            }
        }
    }

    // Build the final TOML document
    let doc = toml::Value::Table(toml_map);
    let content = match toml::to_string_pretty(&doc) {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to build TOML document: {}", e);
            // This shouldn't happen since we already serialized each vault,
            // but return what we have
            return Ok(WriteResult {
                written_count,
                errors,
            });
        }
    };

    // Write to a temporary file first, then rename for atomicity
    let temp_path = match path.parent() {
        Some(parent) => parent.join(format!(
            ".{}.tmp",
            path.file_name().unwrap_or_default().to_string_lossy()
        )),
        None => path.with_extension("tmp"),
    };

    if let Err(e) = std::fs::write(&temp_path, &content) {
        log::error!("Failed to write config to temp file: {}", e);
        let _ = std::fs::remove_file(&temp_path);
        return Err(WriteError::Io(e));
    }

    if let Err(e) = std::fs::rename(&temp_path, path) {
        log::error!("Failed to rename temp file to config file: {}", e);
        let _ = std::fs::remove_file(&temp_path);
        return Err(WriteError::Io(e));
    }

    Ok(WriteResult {
        written_count,
        errors,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::Backend;

    /// Helper to create a test vault configuration.
    fn create_test_vault(name: &str, backend: Backend) -> VaultConfig {
        VaultConfig {
            name: name.to_string(),
            backend,
            encrypted_data_directory: format!("/tmp/test/{}", name),
            mount_directory: format!("/tmp/mount/{}", name),
            session_lock: false,
        }
    }

    #[test]
    fn test_write_empty_vaults() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("empty.toml");
        let vaults: HashMap<Uuid, VaultConfig> = HashMap::new();

        let result = write_config(&path, &vaults);
        assert!(result.is_ok());
        let write_result = result.unwrap();
        assert_eq!(write_result.written_count, 0);
        assert!(write_result.errors.is_empty());

        // File should exist and be empty (just a table)
        assert!(path.exists());
    }

    #[test]
    fn test_write_single_vault() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("single.toml");
        let mut vaults = HashMap::new();
        let uuid = Uuid::parse_str("a19a941c-505a-4b46-86c0-abf39bcdc2e1").unwrap();
        vaults.insert(uuid, create_test_vault("test_vault", Backend::Cryfs));

        let result = write_config(&path, &vaults);
        assert!(result.is_ok());
        let write_result = result.unwrap();
        assert_eq!(write_result.written_count, 1);
        assert!(write_result.errors.is_empty());

        // Verify the file can be read back
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("test_vault"));
        assert!(content.contains("Cryfs"));
    }

    #[test]
    fn test_write_multiple_vaults() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("multiple.toml");
        let mut vaults = HashMap::new();

        let uuid1 = Uuid::parse_str("a19a941c-505a-4b46-86c0-abf39bcdc2e1").unwrap();
        let uuid2 = Uuid::parse_str("b29b941c-505a-4b46-86c0-abf39bcdc2e2").unwrap();
        let uuid3 = Uuid::parse_str("c39c941c-505a-4b46-86c0-abf39bcdc2e3").unwrap();

        vaults.insert(uuid1, create_test_vault("cryfs_vault", Backend::Cryfs));
        vaults.insert(
            uuid2,
            create_test_vault("gocryptfs_vault", Backend::Gocryptfs),
        );
        vaults.insert(uuid3, create_test_vault("another_cryfs", Backend::Cryfs));

        let result = write_config(&path, &vaults);
        assert!(result.is_ok());
        let write_result = result.unwrap();
        assert_eq!(write_result.written_count, 3);
        assert!(write_result.errors.is_empty());

        // Verify the file can be read back
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("cryfs_vault"));
        assert!(content.contains("gocryptfs_vault"));
        assert!(content.contains("another_cryfs"));
    }

    #[test]
    fn test_write_to_nonexistent_directory() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("nonexistent").join("config.toml");
        let vaults: HashMap<Uuid, VaultConfig> = HashMap::new();

        let result = write_config(&path, &vaults);
        assert!(result.is_err());
        matches!(result.unwrap_err(), WriteError::Io(_));
    }

    #[test]
    fn test_write_preserves_vault_data() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("preserve.toml");
        let mut vaults = HashMap::new();
        let uuid = Uuid::parse_str("a19a941c-505a-4b46-86c0-abf39bcdc2e1").unwrap();

        let config = VaultConfig {
            name: "Preserve Test".to_string(),
            backend: Backend::Gocryptfs,
            encrypted_data_directory: "/var/home/user/encrypted".to_string(),
            mount_directory: "/var/home/user/mounted".to_string(),
            session_lock: true,
        };
        vaults.insert(uuid, config);

        let result = write_config(&path, &vaults);
        assert!(result.is_ok());

        // Read back and verify all fields
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("Preserve Test"));
        assert!(content.contains("Gocryptfs"));
        assert!(content.contains("/var/home/user/encrypted"));
        assert!(content.contains("/var/home/user/mounted"));
        assert!(content.contains("session_lock = true"));
    }

    #[test]
    fn test_write_overwrites_existing_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("overwrite.toml");

        // Write initial content
        let mut vaults1 = HashMap::new();
        let uuid1 = Uuid::parse_str("a19a941c-505a-4b46-86c0-abf39bcdc2e1").unwrap();
        vaults1.insert(uuid1, create_test_vault("old_vault", Backend::Cryfs));
        write_config(&path, &vaults1).unwrap();

        // Write new content
        let mut vaults2 = HashMap::new();
        let uuid2 = Uuid::parse_str("b29b941c-505a-4b46-86c0-abf39bcdc2e2").unwrap();
        vaults2.insert(uuid2, create_test_vault("new_vault", Backend::Gocryptfs));
        let result = write_config(&path, &vaults2);

        assert!(result.is_ok());
        let write_result = result.unwrap();
        assert_eq!(write_result.written_count, 1);

        // Verify old content is gone
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(!content.contains("old_vault"));
        assert!(content.contains("new_vault"));
    }

    #[test]
    fn test_write_with_special_characters_in_name() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("special.toml");
        let mut vaults = HashMap::new();
        let uuid = Uuid::parse_str("a19a941c-505a-4b46-86c0-abf39bcdc2e1").unwrap();

        let config = VaultConfig {
            name: "Vault with \"quotes\" & <special> chars".to_string(),
            backend: Backend::Cryfs,
            encrypted_data_directory: "/tmp/special/path".to_string(),
            mount_directory: "/tmp/mount/special".to_string(),
            session_lock: false,
        };
        vaults.insert(uuid, config);

        let result = write_config(&path, &vaults);
        assert!(result.is_ok());
        let write_result = result.unwrap();
        assert_eq!(write_result.written_count, 1);
    }

    #[test]
    fn test_write_round_trip() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("roundtrip.toml");

        // Create vaults
        let mut original_vaults = HashMap::new();
        let uuid1 = Uuid::parse_str("a19a941c-505a-4b46-86c0-abf39bcdc2e1").unwrap();
        let uuid2 = Uuid::parse_str("b29b941c-505a-4b46-86c0-abf39bcdc2e2").unwrap();

        original_vaults.insert(
            uuid1,
            VaultConfig {
                name: "Vault 1".to_string(),
                backend: Backend::Cryfs,
                encrypted_data_directory: "/tmp/v1".to_string(),
                mount_directory: "/tmp/m1".to_string(),
                session_lock: false,
            },
        );
        original_vaults.insert(
            uuid2,
            VaultConfig {
                name: "Vault 2".to_string(),
                backend: Backend::Gocryptfs,
                encrypted_data_directory: "/tmp/v2".to_string(),
                mount_directory: "/tmp/m2".to_string(),
                session_lock: true,
            },
        );

        // Write
        let write_result = write_config(&path, &original_vaults).unwrap();
        assert_eq!(write_result.written_count, 2);

        // Read back using the reader module
        use crate::configuration::reader::read_config;
        let read_result = read_config(&path).unwrap();
        assert_eq!(read_result.vaults.len(), 2);
        assert!(read_result.errors.is_empty());

        // Verify data integrity
        let read_v1 = read_result.vaults.get(&uuid1).unwrap();
        assert_eq!(read_v1.name, "Vault 1");
        assert_eq!(read_v1.backend, Backend::Cryfs);
        assert_eq!(read_v1.session_lock, false);

        let read_v2 = read_result.vaults.get(&uuid2).unwrap();
        assert_eq!(read_v2.name, "Vault 2");
        assert_eq!(read_v2.backend, Backend::Gocryptfs);
        assert_eq!(read_v2.session_lock, true);
    }

    #[test]
    fn test_write_large_number_of_vaults() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("many_vaults.toml");
        let mut vaults = HashMap::new();

        for i in 0..100 {
            let uuid = Uuid::new_v4();
            vaults.insert(
                uuid,
                VaultConfig {
                    name: format!("Vault {}", i),
                    backend: if i % 2 == 0 {
                        Backend::Cryfs
                    } else {
                        Backend::Gocryptfs
                    },
                    encrypted_data_directory: format!("/tmp/vaults/{}", i),
                    mount_directory: format!("/tmp/mounts/{}", i),
                    session_lock: i % 3 == 0,
                },
            );
        }

        let result = write_config(&path, &vaults);
        assert!(result.is_ok());
        let write_result = result.unwrap();
        assert_eq!(write_result.written_count, 100);
        assert!(write_result.errors.is_empty());
    }
}
