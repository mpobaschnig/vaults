// gocryptfs.rs
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

use super::BackendError;
use super::backend::BackendImpl;
use crate::backend::{Backend, create_directories, host_command};
use crate::config::APP_ID;
use crate::global_config_manager::GlobalConfigManager;
use crate::vault_config::VaultConfig;
use gettextrs::gettext;
use gtk::gio::Settings;
use gtk::gio::prelude::SettingsExt;
use std::{io::Write, process::Stdio};

#[derive(Debug, Clone, Default)]
/// gocryptfs backend implementation.
pub struct Gocryptfs {}

impl BackendImpl for Gocryptfs {
    fn is_available(&self) -> Result<bool, BackendError> {
        let settings = Settings::new(APP_ID);
        is_available(&settings)
    }

    fn init(&self, vault_config: &VaultConfig, password: &str) -> Result<(), BackendError> {
        create_directories(vault_config)?;
        let settings = Settings::new(APP_ID);
        init(&settings, vault_config, password.to_owned())
    }

    fn open(&self, vault_config: &VaultConfig, password: &str) -> Result<(), BackendError> {
        let settings = Settings::new(APP_ID);
        open(&settings, vault_config, password.to_owned())
    }

    fn close(&self, vault_config: &VaultConfig) -> Result<(), BackendError> {
        let settings = Settings::new(APP_ID);
        close(&settings, vault_config)
    }

    fn name(&self) -> String {
        String::from("gocryptfs")
    }

    fn kind(&self) -> Backend {
        Backend::Gocryptfs
    }

    fn clone_box(&self) -> Box<dyn BackendImpl> {
        Box::new(self.clone())
    }
}

fn get_binary_path(settings: &Settings) -> Option<String> {
    log::trace!("get_binary_path()");

    if settings.boolean("use-custom-gocryptfs-binary") {
        return Some(settings.string("custom-gocryptfs-binary-path").to_string());
    }

    GlobalConfigManager::instance().get_gocryptfs_binary_path()
}

fn is_available(settings: &Settings) -> Result<bool, BackendError> {
    log::trace!("is_available()");

    let binary_path = get_binary_path(settings);
    if binary_path.is_none() {
        log::error!("gocryptfs binary path is not set");
        return Err(BackendError::ToUser(gettext(
            "No gocryptfs binary path set",
        )));
    }

    let output = host_command(&binary_path.unwrap())
        .arg("--version")
        .output()?;
    log::debug!("gocryptfs output: {:?}", output);

    let success = output.status.success();
    log::info!("gocryptfs is available: {}", success);
    Ok(success)
}

fn init(
    settings: &Settings,
    vault_config: &VaultConfig,
    password: String,
) -> Result<(), BackendError> {
    log::trace!("init({:?}, password: <redacted>)", vault_config);

    let binary_path = get_binary_path(settings);
    if binary_path.is_none() {
        log::error!("gocryptfs binary path is not set");
        return Err(BackendError::ToUser(gettext(
            "No gocryptfs binary path set",
        )));
    }

    let mut child = host_command(&binary_path.unwrap())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .arg("--init")
        .arg("-q")
        .arg("--")
        .arg(&vault_config.encrypted_data_directory)
        .spawn()?;

    let mut pw = String::from(&password);
    pw.push('\n');
    pw.push_str(&password);
    pw.push('\n');

    child
        .stdin
        .as_mut()
        .ok_or(BackendError::Generic)?
        .write_all(pw.as_bytes())?;

    let output = child.wait_with_output()?;
    log::debug!("gocryptfs output: {:?}", output);
    if output.status.success() {
        log::info!("gocryptfs init successful");
        Ok(())
    } else {
        std::io::stdout().write_all(&output.stdout)?;
        std::io::stderr().write_all(&output.stderr)?;

        let err_code = output.status.code();
        log::error!("gocryptfs init failed: {:?}", err_code);
        Err(gocryptfs_ret_status_to_err(err_code))
    }
}

fn open(
    settings: &Settings,
    vault_config: &VaultConfig,
    password: String,
) -> Result<(), BackendError> {
    log::trace!("open({:?}, password: <redacted>)", vault_config);

    let binary_path = get_binary_path(settings);
    if binary_path.is_none() {
        log::error!("gocryptfs binary path is not set");
        return Err(BackendError::ToUser(gettext(
            "No gocryptfs binary path set",
        )));
    }

    let mut child = host_command(&binary_path.unwrap())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .arg("-q")
        .arg("--")
        .arg(&vault_config.encrypted_data_directory)
        .arg(&vault_config.mount_directory)
        .spawn()?;

    let mut pw = String::from(&password);
    pw.push('\n');

    child
        .stdin
        .as_mut()
        .ok_or(BackendError::Generic)?
        .write_all(pw.as_bytes())?;

    let output = child.wait_with_output()?;
    log::debug!("gocryptfs output: {:?}", output);
    if output.status.success() {
        log::info!("gocryptfs open successful");
        Ok(())
    } else {
        std::io::stdout().write_all(&output.stdout)?;
        std::io::stderr().write_all(&output.stderr)?;

        let err_code = output.status.code();
        log::error!("gocryptfs open failed: {:?}", err_code);
        Err(gocryptfs_ret_status_to_err(err_code))
    }
}

fn close(_settings: &Settings, vault_config: &VaultConfig) -> Result<(), BackendError> {
    log::trace!("close({:?}, password: <redacted>)", vault_config);

    // First, try a normal unmount
    let child = host_command("umount")
        .stdout(Stdio::piped())
        .arg(&vault_config.mount_directory)
        .spawn()?;

    let output = child.wait_with_output()?;
    log::debug!("umount output: {:?}", output);
    if output.status.success() {
        log::info!("umount close successful");
        return Ok(());
    }

    let err_code = output.status.code();
    if err_code == Some(32) {
        log::warn!(
            "Mount point is busy, attempting lazy unmount for: {}",
            vault_config.mount_directory
        );
        std::io::stdout().write_all(&output.stdout)?;
        std::io::stderr().write_all(&output.stderr)?;

        let child = host_command("umount")
            .arg("-l")
            .stdout(Stdio::piped())
            .arg(&vault_config.mount_directory)
            .spawn()?;

        let output = child.wait_with_output()?;
        log::debug!("lazy umount output: {:?}", output);
        if output.status.success() {
            log::info!("lazy umount close successful");
            return Ok(());
        }

        std::io::stdout().write_all(&output.stdout)?;
        std::io::stderr().write_all(&output.stderr)?;
        let lazy_err_code = output.status.code();
        log::error!("gocryptfs lazy close failed: {:?}", lazy_err_code);
        return Err(umount_ret_status_to_err(lazy_err_code));
    }

    // Other error codes
    std::io::stdout().write_all(&output.stdout)?;
    std::io::stderr().write_all(&output.stderr)?;
    log::error!("gocryptfs close failed: {:?}", err_code);
    Err(umount_ret_status_to_err(err_code))
}

fn gocryptfs_ret_status_to_err(status: Option<i32>) -> BackendError {
    log::trace!("status_to_err({:?})", status);

    struct GocryptfsExitStatus {}

    #[allow(dead_code)]
    impl GocryptfsExitStatus {
        pub const SUCCESS: i32 = 0;
        pub const NON_EMPTY_CIPHER_DIR: i32 = 6;
        pub const NON_EMPTY_MOUNT_POINT: i32 = 10;
        pub const WRONG_PASSWORD: i32 = 12;
        pub const EMPTY_PASSWORD: i32 = 22;
        pub const CANNOT_READ_CONFIG: i32 = 23;
        pub const CANNOT_WRITE_CONFIG: i32 = 24;
        pub const FSCK_ERROR: i32 = 26;
    }

    match status {
        Some(status) => match status {
            GocryptfsExitStatus::NON_EMPTY_CIPHER_DIR => {
                BackendError::ToUser(gettext("The encrypted data directory is not empty."))
            }
            GocryptfsExitStatus::NON_EMPTY_MOUNT_POINT => {
                BackendError::ToUser(gettext("The mount directory is not empty."))
            }
            GocryptfsExitStatus::WRONG_PASSWORD => {
                BackendError::ToUser(gettext("The password is wrong."))
            }
            GocryptfsExitStatus::EMPTY_PASSWORD => {
                BackendError::ToUser(gettext("The password is empty."))
            }
            GocryptfsExitStatus::CANNOT_READ_CONFIG => {
                BackendError::ToUser(gettext("Vaults cannot read configuration file."))
            }
            GocryptfsExitStatus::CANNOT_WRITE_CONFIG => {
                BackendError::ToUser(gettext("Vaults cannot write configuration file."))
            }
            GocryptfsExitStatus::FSCK_ERROR => {
                BackendError::ToUser(gettext("The file system check reported an error."))
            }
            _ => BackendError::ToUser(gettext("An unknown error occurred.")),
        },
        None => BackendError::Generic,
    }
}

fn umount_ret_status_to_err(status: Option<i32>) -> BackendError {
    log::trace!("umount_ret_status_to_err({:?})", status);

    // We are guaranteed to have a non-zero errno here
    if let Some(status) = status {
        match status {
            1 => BackendError::ToUser(gettext(
                "You don't have the necessary privileges to unmount the directory.",
            )),
            2 => BackendError::ToUser(gettext("The data directory (mount point) does not exist.")),
            4 => BackendError::ToUser(gettext("Internal error.")),
            32 => BackendError::ToUser(gettext(
                "The data directory (mount point) is busy. There are open files or processes using the filesystem.",
            )),
            _ => BackendError::Generic,
        }
    } else {
        BackendError::Generic
    }
}
