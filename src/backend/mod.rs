// mod.rs
//
// Copyright 2021 Martin Pobaschnig
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

pub mod backend_impl;
pub mod cryfs;
pub mod gocryptfs;
#[cfg(any(test, debug_assertions))]
pub mod mock;

use gettextrs::gettext;
use gtk::glib;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use std::string::String;
use strum_macros::EnumIter;

quick_error! {
    #[derive(Debug)]
    pub enum BackendError {
        ToUser(e: String) {
            display("{}", e)
        }
        Generic {
            from(std::io::Error)
        }
    }
}

#[derive(
    Debug,
    PartialEq,
    EnumIter,
    strum_macros::Display,
    Serialize,
    Deserialize,
    strum_macros::EnumString,
    Copy,
    Clone,
    glib::Enum,
    Default,
)]
#[enum_type(name = "Backend")]
pub enum Backend {
    #[default]
    #[enum_value(name = "cryfs")]
    Cryfs,
    #[enum_value(name = "gocryptfs")]
    Gocryptfs,
    /// Mock backend, only available in debug and test builds.
    #[cfg(any(test, debug_assertions))]
    #[enum_value(name = "mock")]
    Mock,
}

pub fn get_ui_string_from_backend(backend: &Backend) -> String {
    log::trace!("get_ui_string_from_backend({:?})", backend);

    match backend {
        Backend::Cryfs => String::from("CryFS"),
        Backend::Gocryptfs => String::from("gocryptfs"),
        #[cfg(any(test, debug_assertions))]
        Backend::Mock => String::from("mock"),
    }
}

pub fn get_backend_from_ui_string(backend: &str) -> Option<Backend> {
    log::trace!("get_backend_from_ui_string({:?})", backend);

    if backend == "CryFS" {
        Some(Backend::Cryfs)
    } else if backend == "gocryptfs" {
        Some(Backend::Gocryptfs)
    } else {
        None
    }
}

/// Returns whether the application is running inside a Flatpak sandbox.
///
/// Detects the Flatpak environment by checking for the existence of the
/// `/.flatpak-info` file, which is present in every Flatpak sandbox.
pub fn is_flatpak() -> bool {
    log::trace!("is_flatpak()");

    Path::new("/.flatpak-info").exists()
}

/// Creates a command running `binary` on the host system.
///
/// When running inside a Flatpak sandbox, the command is wrapped in
/// `flatpak-spawn --host` so the binary runs on the host. When running
/// natively, the binary is executed directly.
///
/// * `binary`: Path to or name of the binary to run.
///
/// Returns the command to which further arguments can be added.
pub fn host_command(binary: &str) -> Command {
    log::trace!("host_command({:?})", binary);

    if is_flatpak() {
        let mut command = Command::new("flatpak-spawn");
        command.arg("--host").arg(binary);
        command
    } else {
        Command::new(binary)
    }
}

/// Creates the encrypted data and mount directories for a vault.
pub fn create_directories(
    vault_config: &crate::vault_config::VaultConfig,
) -> Result<(), BackendError> {
    create_edd_if_not_exists(&vault_config.encrypted_data_directory)?;
    create_md_if_not_exists(&vault_config.mount_directory)
}

fn create_edd_if_not_exists(encrypted_data_directory: &str) -> Result<(), BackendError> {
    log::trace!("create_edd_if_not_exists({:?}", encrypted_data_directory);

    match std::fs::create_dir_all(encrypted_data_directory) {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("Failed to create encrypted data directory: {}", e);

            match e.kind() {
                std::io::ErrorKind::PermissionDenied => Err(BackendError::ToUser(gettext(
                    "Failed to create encrypted data directory: Permission denied.",
                ))),
                std::io::ErrorKind::ConnectionRefused => Err(BackendError::ToUser(gettext(
                    "Failed to create encrypted data directory: Connection refused.",
                ))),
                std::io::ErrorKind::ConnectionReset => Err(BackendError::ToUser(gettext(
                    "Failed to create encrypted data directory: Connection reset.",
                ))),
                std::io::ErrorKind::ConnectionAborted => Err(BackendError::ToUser(gettext(
                    "Failed to create encrypted data directory: Connection aborted.",
                ))),
                std::io::ErrorKind::NotConnected => Err(BackendError::ToUser(gettext(
                    "Failed to create encrypted data directory: Not connected.",
                ))),
                _ => Err(BackendError::ToUser(gettext(
                    "Failed to create encrypted data directory.",
                ))),
            }
        }
    }
}

fn create_md_if_not_exists(mount_directory: &str) -> Result<(), BackendError> {
    log::trace!("create_md_if_not_exists({:?}", mount_directory);

    match std::fs::create_dir_all(mount_directory) {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("Failed to create mount directory: {}", e);

            match e.kind() {
                std::io::ErrorKind::PermissionDenied => Err(BackendError::ToUser(gettext(
                    "Failed to create mount directory: Permission denied.",
                ))),
                std::io::ErrorKind::ConnectionRefused => Err(BackendError::ToUser(gettext(
                    "Failed to create mount directory: Connection refused.",
                ))),
                std::io::ErrorKind::ConnectionReset => Err(BackendError::ToUser(gettext(
                    "Failed to create mount directory: Connection reset.",
                ))),
                std::io::ErrorKind::ConnectionAborted => Err(BackendError::ToUser(gettext(
                    "Failed to create mount directory: Connection aborted.",
                ))),
                std::io::ErrorKind::NotConnected => Err(BackendError::ToUser(gettext(
                    "Failed to create mount directory: Not connected.",
                ))),
                _ => Err(BackendError::ToUser(gettext(
                    "Failed to create mount directory.",
                ))),
            }
        }
    }
}
