// cryfs.rs
//
// Copyright 2021 Martin Pobaschnig <mpobaschnig@posteo.de>
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

use super::BackendError;
use crate::vault::VaultConfig;
use gettextrs::gettext;
use std::process::Command;
use std::{self, io::Write, process::Stdio};

pub fn is_available() -> Result<bool, BackendError> {
    let output = Command::new("flatpak-spawn")
        .arg("--host")
        .arg("cryfs")
        .arg("--version")
        .output()?;

    Ok(output.status.success())
}

pub fn init(_vault_config: &VaultConfig, _password: String) -> Result<(), BackendError> {
    Ok(())
}

pub fn open(vault_config: &VaultConfig, password: String) -> Result<(), BackendError> {
    let mut child = Command::new("flatpak-spawn")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .arg("--host")
        .arg("--env=CRYFS_FRONTEND=noninteractive")
        .arg("cryfs")
        .arg(&vault_config.encrypted_data_directory)
        .arg(&vault_config.mount_directory)
        .spawn()?;

    let mut pw = String::from(&password);
    pw.push_str(&"\n".to_owned());

    child
        .stdin
        .as_mut()
        .ok_or(BackendError::Generic)?
        .write_all(pw.as_bytes())?;

    let output = child.wait_with_output()?;
    if output.status.success() {
        Ok(())
    } else {
        std::io::stdout().write_all(&output.stdout).unwrap();
        std::io::stderr().write_all(&output.stderr).unwrap();

        Err(status_to_err(output.status.code()))
    }
}

pub fn close(vault_config: &VaultConfig) -> Result<(), BackendError> {
    let child = Command::new("flatpak-spawn")
        .stdout(Stdio::piped())
        .arg("--host")
        .arg("cryfs-unmount")
        .arg(&vault_config.mount_directory)
        .spawn()?;

    let output = child.wait_with_output()?;
    if output.status.success() {
        Ok(())
    } else {
        std::io::stdout().write_all(&output.stdout).unwrap();
        std::io::stderr().write_all(&output.stderr).unwrap();

        Err(status_to_err(output.status.code()))
    }
}

fn status_to_err(status: Option<i32>) -> BackendError {
    struct CryfsExitStatus {}

    // Error codes and text from:
    // https://github.com/cryfs/cryfs/blob/develop/src/cryfs/impl/ErrorCodes.h
    impl CryfsExitStatus {
        pub const _SUCCESS: i32 = 0;
        // An error happened that doesn't have an error code associated with it
        pub const UNSPECIFIED_ERROR: i32 = 1;
        // The command line arguments are invalid.
        pub const INVALID_ARGUMENTS: i32 = 10;
        // Couldn't load config file. Probably the password is wrong
        pub const WRONG_PASSWORD: i32 = 11;
        // Password cannot be empty
        pub const EMPTY_PASSWORD: i32 = 12;
        // The file system format is too new for this CryFS version. Please update your CryFS version.
        pub const TOO_NEW_FILESYSTEM_FORMAT: i32 = 13;
        // The file system format is too old for this CryFS version. Run with --allow-filesystem-upgrade to upgrade it.
        pub const TOO_OLD_FILESYSTEM_FORMAT: i32 = 14;
        // The file system uses a different cipher than the one specified on the command line using the --cipher argument.
        pub const WRONG_CIPHER: i32 = 15;
        // Base directory doesn't exist or is inaccessible (i.e. not read or writable or not a directory)
        pub const INACCESSIBLE_BASE_DIR: i32 = 16;
        // Mount directory doesn't exist or is inaccessible (i.e. not read or writable or not a directory)
        pub const INACCESSIBLE_MOUNT_DIR: i32 = 17;
        // Base directory can't be a subdirectory of the mount directory
        pub const BASE_DIR_INSIDE_MOUNT_DIR: i32 = 18;
        // Something's wrong with the file system.
        pub const INVALID_FILESYSTEM: i32 = 19;
        // The filesystem id in the config file is different to the last time we loaded a filesystem from this basedir. This could mean an attacker replaced the file system with a different one. You can pass the --allow-replaced-filesystem option to allow this.
        pub const FILESYSTEM_ID_CHANGED: i32 = 20;
        // The filesystem encryption key differs from the last time we loaded this filesystem. This could mean an attacker replaced the file system with a different one. You can pass the --allow-replaced-filesystem option to allow this.
        pub const ENCRYPTION_KEY_CHANGED: i32 = 21;
        // The command line options and the file system disagree on whether missing blocks should be treated as integrity violations.
        pub const FILESYSTEM_HAS_DIFFERENT_INTEGRITY_SETUP: i32 = 22;
        // File system is in single-client mode and can only be used from the client that created it.
        pub const SINGLE_CLIENT_FILE_SYSTEM: i32 = 23;
        // A previous run of the file system detected an integrity violation. Preventing access to make sure the user notices. The file system will be accessible again after the user deletes the integrity state file.
        pub const INTEGRITY_VIOLATION_ON_PREVIOUS_RUN: i32 = 24;
        // An integrity violation was detected and the file system unmounted to make sure the user notices.
        pub const INTEGRITY_VIOLATION: i32 = 25;
    }

    match status {
        Some(status) => match status {
            CryfsExitStatus::UNSPECIFIED_ERROR => BackendError::ToUser(gettext("An unknown error occurred.")),
            CryfsExitStatus::INVALID_ARGUMENTS => BackendError::ToUser(gettext("Invalid arguments were given.")),
            CryfsExitStatus::WRONG_PASSWORD => BackendError::ToUser(gettext("The password is wrong.")),
            CryfsExitStatus::EMPTY_PASSWORD => BackendError::ToUser(gettext("The password is empty.")),
            CryfsExitStatus::TOO_NEW_FILESYSTEM_FORMAT => BackendError::ToUser(gettext("The file system format is too new for this CryFS version. Please update cryFs.")),
            CryfsExitStatus::TOO_OLD_FILESYSTEM_FORMAT => BackendError::ToUser(gettext("The file system format is too old for this CryFS version.")),
            CryfsExitStatus::WRONG_CIPHER => BackendError::ToUser(gettext("The vault uses a different cipher than the default for CryFS.")),
            CryfsExitStatus::INACCESSIBLE_BASE_DIR => BackendError::ToUser(gettext("The encrypted data directory does not exist or is inaccessible.")),
            CryfsExitStatus::INACCESSIBLE_MOUNT_DIR => BackendError::ToUser(gettext("The mount directory does not exist or is inaccessible.")),
            CryfsExitStatus::BASE_DIR_INSIDE_MOUNT_DIR => BackendError::ToUser(gettext("The mount directory is inside the Encrypted Data Directory.")),
            CryfsExitStatus::INVALID_FILESYSTEM => BackendError::ToUser(gettext("The file system is invalid.")),
            CryfsExitStatus::FILESYSTEM_ID_CHANGED => BackendError::ToUser(gettext("The file system id in the configuration file is different to the last time this Vault was mounted. This could mean someone replaced the file system with a different one.")),
            CryfsExitStatus::ENCRYPTION_KEY_CHANGED => BackendError::ToUser(gettext("The file system encryption key is different to the last time this Vault was mounted. This could mean someone replaced the file system with a different one.")),
            CryfsExitStatus::FILESYSTEM_HAS_DIFFERENT_INTEGRITY_SETUP => BackendError::ToUser(gettext("Vaults and the file system disagree if missing blocks should be treated as integrity violations.")),
            CryfsExitStatus::SINGLE_CLIENT_FILE_SYSTEM => BackendError::ToUser(gettext("The file system is in single-client mode and can only be used from the client that created it.")),
            CryfsExitStatus::INTEGRITY_VIOLATION_ON_PREVIOUS_RUN => BackendError::ToUser(gettext("A previous run of the file system detected an integrity violation. The file system will be accessible again after integrity state file is deleted.")),
            CryfsExitStatus::INTEGRITY_VIOLATION => BackendError::ToUser(gettext("An integrity violation was detected.")),
            _ => BackendError::ToUser(gettext("An unknown error occurred.")),
        }
        None => BackendError::Generic
    }
}
