// reader.rs
//
// Copyright 2026 Martin Pobaschnig
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

//! Reads vault configurations from a TOML file.
//!
//! Each vault is parsed individually so that one malformed vault does not
//! prevent the others from being loaded. Errors are collected and returned
//! alongside the successfully parsed vaults.

use crate::util::Uuid;
use crate::vault_config::VaultConfig;
use std::collections::HashMap;
use std::path::Path;

/// Describes what went wrong while reading a configuration file.
#[derive(Debug)]
#[allow(dead_code)]
pub enum ReadError {
    /// The file could not be read from the disk.
    Io(std::io::Error),
    /// The file content is not valid TOML.
    TomlParse(toml::de::Error),
    /// A single vault entry (identified by its UUID string) could not be deserialized into a
    /// [VaultConfig].
    VaultParse { uuid: String, error: String },
}

/// The result of a [read_config] call.
///
/// `vaults` contains every vault that was successfully parsed.
///
/// `errors` lists any problems encountered; it may be non-empty even when some vaults
/// were loaded.
#[derive(Debug)]
pub struct ReadResult {
    pub vaults: HashMap<Uuid, VaultConfig>,
    pub errors: Vec<ReadError>,
}

/// Read all vault configurations from the TOML file at `path`.
///
/// The expected format is a flat map where each top-level key is a UUID and
/// the value is a table with vault fields.  Parsing is resilient: if one
/// vault entry is malformed, the others are still returned.
pub fn read_config(path: &Path) -> Result<ReadResult, ReadError> {
    let content = std::fs::read_to_string(path).map_err(ReadError::Io)?;

    // First, try the fast path: parse the entire file as a HashMap directly.
    let parsed: toml::map::Map<String, toml::Value> =
        toml::from_str(&content).map_err(ReadError::TomlParse)?;

    let mut vaults = HashMap::new();
    let mut errors = Vec::new();

    for (uuid_str, value) in parsed {
        match parse_single_vault(&uuid_str, &value) {
            Ok((uuid, config)) => {
                vaults.insert(uuid, config);
            }
            Err(e) => {
                errors.push(e);
            }
        }
    }

    Ok(ReadResult { vaults, errors })
}

/// Try to deserialize a single vault entry from its TOML value.
fn parse_single_vault(
    uuid_str: &str,
    value: &toml::Value,
) -> Result<(Uuid, VaultConfig), ReadError> {
    let uuid = match Uuid::parse_str(uuid_str) {
        Ok(u) => u,
        Err(e) => {
            log::warn!(
                "Vault key '{}' is not a valid UUID: {}. Skipping this entry.",
                uuid_str,
                e
            );
            return Err(ReadError::VaultParse {
                uuid: uuid_str.to_string(),
                error: format!("Invalid UUID: {}", e),
            });
        }
    };

    let config: VaultConfig = match serde::Deserialize::deserialize(value.clone()) {
        Ok(c) => c,
        Err(e) => {
            log::warn!(
                "Failed to parse vault configuration for '{}': {}. Skipping this entry.",
                uuid_str,
                e
            );
            return Err(ReadError::VaultParse {
                uuid: uuid_str.to_string(),
                error: e.to_string(),
            });
        }
    };

    Ok((uuid, config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::Backend;

    #[test]
    fn test_read_empty_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("empty.toml");
        std::fs::write(&path, "").unwrap();
        let result = read_config(&path);
        assert!(result.is_ok());
        let read_result = result.unwrap();
        assert!(read_result.vaults.is_empty());
        assert!(read_result.errors.is_empty());
    }

    #[test]
    fn test_read_single_gocryptfs_vault() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("one_gocryptfs.toml");
        let content = r#"[a19a941c-505a-4b46-86c0-abf39bcdc2e1]
name = "Test Vault"
backend = "Cryfs"
encrypted_data_directory = "/var/home/user/edd/test"
mount_directory = "/var/home/user/md/test"
session_lock = false
"#;
        std::fs::write(&path, content).unwrap();
        let result = read_config(&path);
        assert!(result.is_ok());
        let read_result = result.unwrap();
        assert_eq!(read_result.vaults.len(), 1);
        assert!(read_result.errors.is_empty());

        let uuid = Uuid::parse_str("a19a941c-505a-4b46-86c0-abf39bcdc2e1").unwrap();
        let vault = read_result.vaults.get(&uuid).expect("vault should exist");
        assert_eq!(vault.name, "Test Vault");
        assert_eq!(vault.backend.kind(), Backend::Cryfs);
        assert_eq!(vault.encrypted_data_directory, "/var/home/user/edd/test");
        assert_eq!(vault.mount_directory, "/var/home/user/md/test");
        assert!(!vault.session_lock);
    }

    #[test]
    fn test_read_single_cryfs_vault() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("one_cryfs.toml");
        let content = r#"[a19a941c-505a-4b46-86c0-abf39bcdc2e1]
name = "Test Vault"
backend = "Gocryptfs"
encrypted_data_directory = "/var/home/user/edd/test"
mount_directory = "/var/home/user/md/test"
session_lock = false
"#;
        std::fs::write(&path, content).unwrap();
        let result = read_config(&path);
        assert!(result.is_ok());
        let read_result = result.unwrap();
        assert_eq!(read_result.vaults.len(), 1);
        assert!(read_result.errors.is_empty());

        let uuid = Uuid::parse_str("a19a941c-505a-4b46-86c0-abf39bcdc2e1").unwrap();
        let vault = read_result.vaults.get(&uuid).expect("vault should exist");
        assert_eq!(vault.name, "Test Vault");
        assert_eq!(vault.backend.kind(), Backend::Gocryptfs);
        assert_eq!(vault.encrypted_data_directory, "/var/home/user/edd/test");
        assert_eq!(vault.mount_directory, "/var/home/user/md/test");
        assert!(!vault.session_lock);
    }

    #[test]
    fn test_read_nonexistent_file() {
        let path = Path::new("/tmp/does-not-exist-vault-config.toml");
        let result = read_config(path);
        assert!(result.is_err());
        matches!(result.unwrap_err(), ReadError::Io(_));
    }

    #[test]
    fn test_read_invalid_toml() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("invalid.toml");
        std::fs::write(&path, "this is not [[ valid toml {{{").unwrap();
        let result = read_config(&path);
        assert!(result.is_err());
        matches!(result.unwrap_err(), ReadError::TomlParse(_));
    }

    #[test]
    fn test_read_invalid_uuid_key() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("bad_uuid.toml");
        let content = r#"
[not-a-uuid]
name = "Bad"
backend = "Cryfs"
encrypted_data_directory = "/tmp/bad"
mount_directory = "/tmp/mount"
session_lock = false
"#;
        std::fs::write(&path, content).unwrap();
        let result = read_config(&path).unwrap();
        assert!(result.vaults.is_empty());
        assert_eq!(result.errors.len(), 1);
        matches!(result.errors[0], ReadError::VaultParse { .. });
    }

    #[test]
    fn test_read_mixed_valid_and_invalid_vaults() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("mixed.toml");
        let content = r#"
[a19a941c-505a-4b46-86c0-abf39bcdc2e1]
name = "Good Vault"
backend = "Cryfs"
encrypted_data_directory = "/tmp/good"
mount_directory = "/tmp/mount"
session_lock = false

[not-a-uuid]
name = "Bad UUID"
backend = "Cryfs"
encrypted_data_directory = "/tmp/bad"
mount_directory = "/tmp/mount"
session_lock = false

[b29b941c-505a-4b46-86c0-abf39bcdc2e2]
name = "Another Good Vault"
backend = "Gocryptfs"
encrypted_data_directory = "/tmp/good2"
mount_directory = "/tmp/mount2"
session_lock = true
"#;
        std::fs::write(&path, content).unwrap();
        let result = read_config(&path).unwrap();

        // Two valid vaults should be loaded
        assert_eq!(result.vaults.len(), 2);
        // One error for the invalid UUID
        assert_eq!(result.errors.len(), 1);

        let uuid1 = Uuid::parse_str("a19a941c-505a-4b46-86c0-abf39bcdc2e1").unwrap();
        let uuid2 = Uuid::parse_str("b29b941c-505a-4b46-86c0-abf39bcdc2e2").unwrap();
        assert!(result.vaults.contains_key(&uuid1));
        assert!(result.vaults.contains_key(&uuid2));
    }

    #[test]
    fn test_read_vault_with_missing_fields() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("missing_fields.toml");
        let content = r#"
[a19a941c-505a-4b46-86c0-abf39bcdc2e1]
name = "Incomplete"
backend = "Cryfs"
"#;
        std::fs::write(&path, content).unwrap();
        let result = read_config(&path).unwrap();
        // Missing required fields should cause a parse error for this vault
        assert!(result.vaults.is_empty());
        assert_eq!(result.errors.len(), 1);
        matches!(result.errors[0], ReadError::VaultParse { .. });
    }

    #[test]
    fn test_read_vault_with_invalid_backend() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("bad_backend.toml");
        let content = r#"
[a19a941c-505a-4b46-86c0-abf39bcdc2e1]
name = "Bad Backend"
backend = "InvalidBackend"
encrypted_data_directory = "/tmp/bad"
mount_directory = "/tmp/mount"
session_lock = false
"#;
        std::fs::write(&path, content).unwrap();
        let result = read_config(&path).unwrap();
        assert!(result.vaults.is_empty());
        assert_eq!(result.errors.len(), 1);
        matches!(result.errors[0], ReadError::VaultParse { .. });
    }

    #[test]
    fn test_read_vault_with_invalid_session_lock_type() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("bad_session_lock.toml");
        let content = r#"
[a19a941c-505a-4b46-86c0-abf39bcdc2e1]
name = "Bad Session Lock"
backend = "Cryfs"
encrypted_data_directory = "/tmp/bad"
mount_directory = "/tmp/mount"
session_lock = "not-a-bool"
"#;
        std::fs::write(&path, content).unwrap();
        let result = read_config(&path).unwrap();
        assert!(result.vaults.is_empty());
        assert_eq!(result.errors.len(), 1);
        matches!(result.errors[0], ReadError::VaultParse { .. });
    }

    #[test]
    fn test_read_multiple_valid_vaults() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("multiple.toml");
        let content = r#"
[a19a941c-505a-4b46-86c0-abf39bcdc2e1]
name = "Vault 1"
backend = "Cryfs"
encrypted_data_directory = "/tmp/v1"
mount_directory = "/tmp/m1"
session_lock = false

[b29b941c-505a-4b46-86c0-abf39bcdc2e2]
name = "Vault 2"
backend = "Gocryptfs"
encrypted_data_directory = "/tmp/v2"
mount_directory = "/tmp/m2"
session_lock = true

[c39c941c-505a-4b46-86c0-abf39bcdc2e3]
name = "Vault 3"
backend = "Cryfs"
encrypted_data_directory = "/tmp/v3"
mount_directory = "/tmp/m3"
session_lock = false
"#;
        std::fs::write(&path, content).unwrap();
        let result = read_config(&path).unwrap();
        assert_eq!(result.vaults.len(), 3);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_read_vault_with_empty_name() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("empty_name.toml");
        let content = r#"
[a19a941c-505a-4b46-86c0-abf39bcdc2e1]
name = ""
backend = "Cryfs"
encrypted_data_directory = "/tmp/empty"
mount_directory = "/tmp/mount"
session_lock = false
"#;
        std::fs::write(&path, content).unwrap();
        let result = read_config(&path).unwrap();
        // Empty name is technically valid TOML, so it should parse
        assert_eq!(result.vaults.len(), 1);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_read_vault_with_extra_fields() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let path = temp_dir.path().join("extra_fields.toml");
        let content = r#"
[a19a941c-505a-4b46-86c0-abf39bcdc2e1]
name = "Extra"
backend = "Cryfs"
encrypted_data_directory = "/tmp/extra"
mount_directory = "/tmp/mount"
session_lock = false
extra_field = "should be ignored"
"#;
        std::fs::write(&path, content).unwrap();
        let result = read_config(&path).unwrap();
        // Extra fields should be ignored by default serde behavior
        assert_eq!(result.vaults.len(), 1);
        assert!(result.errors.is_empty());
    }
}
