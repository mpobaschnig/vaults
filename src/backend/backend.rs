/*
 * backend.rs
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

use crate::backend::{Backend, BackendError};
use crate::vault_config::VaultConfig;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Debug;

/// Trait defining the interface for encryption backends.
/// Implementations provide the core functionality for initializing, opening, and closing encrypted vaults.
pub trait BackendImpl: Debug + Send + Sync {
    /// Checks if the backend binary is available on the system.
    ///
    /// # Returns
    /// `true` if the backend is available, `false` otherwise, or an error if the binary path is not set.
    fn is_available(&self) -> Result<bool, BackendError>;

    /// Initializes a new encrypted vault.
    ///
    /// # Arguments
    /// * `vault_config` - The configuration for the vault to be initialized.
    /// * `password` - The password for encrypting the vault.
    ///
    /// # Returns
    /// `Ok(())` if initialization succeeds, or a `BackendError` describing the failure.
    fn init(&self, vault_config: &VaultConfig, password: &str) -> Result<(), BackendError>;

    /// Opens an existing encrypted vault and mounts it.
    ///
    /// # Arguments
    /// * `vault_config` - The configuration for the vault to be opened.
    /// * `password` - The password for decrypting the vault.
    ///
    /// # Returns
    /// `Ok(())` if the vault opens successfully, or a `BackendError` describing the failure.
    fn open(&self, vault_config: &VaultConfig, password: &str) -> Result<(), BackendError>;

    /// Closes and unmounts an encrypted vault.
    ///
    /// # Arguments
    /// * `vault_config` - The configuration for the vault to be closed.
    ///
    /// # Returns
    /// `Ok(())` if the vault closes successfully, or a `BackendError` describing the failure.
    fn close(&self, vault_config: &VaultConfig) -> Result<(), BackendError>;

    /// Returns a human-readable name for the backend.
    ///
    /// # Returns
    /// A string identifying the backend (e.g., "CryFS", "gocryptfs").
    fn name(&self) -> String;

    /// Returns the enum representation used for serialization and UI selection.
    fn kind(&self) -> Backend;

    /// Clones this backend implementation into a boxed trait object.
    fn clone_box(&self) -> Box<dyn BackendImpl>;
}

impl Clone for Box<dyn BackendImpl> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl Default for Box<dyn BackendImpl> {
    fn default() -> Self {
        Box::new(super::cryfs::CryFs::default())
    }
}

impl PartialEq for Box<dyn BackendImpl> {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind()
    }
}

impl From<Backend> for Box<dyn BackendImpl> {
    fn from(backend: Backend) -> Self {
        match backend {
            Backend::Cryfs => Box::new(super::cryfs::CryFs::default()),
            Backend::Gocryptfs => Box::new(super::gocryptfs::Gocryptfs::default()),
        }
    }
}

pub mod serde_backend {
    use super::*;

    /// Serializes a backend implementation as its stable enum representation.
    pub fn serialize<S>(backend: &Box<dyn BackendImpl>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        backend.kind().serialize(serializer)
    }

    /// Deserializes the stable enum representation into a backend implementation.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Box<dyn BackendImpl>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Backend::deserialize(deserializer).map(Into::into)
    }
}
