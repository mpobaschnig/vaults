// gocryptfs.rs
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
use std::{io::Write, process::Stdio};

pub fn is_available() -> bool {
    let output_res = Command::new("flatpak-spawn")
        .arg("--host")
        .arg("gocryptfs")
        .arg("--version")
        .output();

    match output_res {
        Ok(output) => {
            if output.status.success() {
                return true;
            }
        }
        Err(e) => {
            log::error!("Failed to probe gocryptfs: {}", e);
        }
    }

    false
}

pub fn init(vault_config: &VaultConfig, password: String) -> Result<(), BackendError> {
    let child = Command::new("flatpak-spawn")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .arg("--host")
        .arg("gocryptfs")
        .arg("--init")
        .arg("-q")
        .arg("--")
        .arg(&vault_config.encrypted_data_directory)
        .spawn();

    match child {
        Ok(mut child) => {
            let stdin = child.stdin.as_mut();
            match stdin {
                Some(stdin) => {
                    let mut pw = String::from(&password);
                    pw.push_str(&"\n".to_owned());
                    pw.push_str(&password);
                    pw.push_str(&"\n".to_owned());
                    match stdin.write_all(pw.as_bytes()) {
                        Ok(_) => match child.wait_with_output() {
                            Ok(output) => {
                                if output.status.success() {
                                    log::info!("Successfully opened vault");
                                } else {
                                    match output.status.code() {
                                        Some(status) => {
                                            return Err(status_to_err(status));
                                        }
                                        None => {
                                            return Err(BackendError::Generic);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to wait for child: {}", e);
                                return Err(BackendError::Generic);
                            }
                        },
                        Err(e) => {
                            log::error!("Failed to write to stdin: {}", e);
                            return Err(BackendError::Generic);
                        }
                    }
                }
                None => {
                    log::error!("Could not get stdin of child!");
                    return Err(BackendError::Generic);
                }
            }

            Ok(())
        }
        Err(e) => {
            log::error!("Failed to init vault: {}", e);
            Err(BackendError::Generic)
        }
    }
}

pub fn open(vault_config: &VaultConfig, password: String) -> Result<(), BackendError> {
    let child = Command::new("flatpak-spawn")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .arg("--host")
        .arg("gocryptfs")
        .arg("-q")
        .arg("--")
        .arg(&vault_config.encrypted_data_directory)
        .arg(&vault_config.mount_directory)
        .spawn();

    match child {
        Ok(mut child) => {
            match child.stdin.as_mut() {
                Some(stdin) => {
                    let mut pw = String::from(&password);
                    pw.push_str(&"\n".to_owned());
                    match stdin.write_all(pw.as_bytes()) {
                        Ok(_) => match child.wait_with_output() {
                            Ok(output) => {
                                if output.status.success() {
                                    log::debug!("Successfully opened vault");
                                } else {
                                    match output.status.code() {
                                        Some(status) => {
                                            return Err(status_to_err(status));
                                        }
                                        None => {
                                            return Err(BackendError::Generic);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to wait for child: {}", e);
                                return Err(BackendError::Generic);
                            }
                        },
                        Err(e) => {
                            log::error!("Failed to write to stdin: {}", e);
                            return Err(BackendError::Generic);
                        }
                    }
                }
                None => {
                    log::error!("Could not get stdin of child!");
                    return Err(BackendError::Generic);
                }
            }

            Ok(())
        }
        Err(e) => {
            log::error!("Failed to init vault: {}", e);
            Err(BackendError::Generic)
        }
    }
}

pub fn close(vault_config: &VaultConfig) -> Result<(), BackendError> {
    let child = Command::new("flatpak-spawn")
        .stdout(Stdio::piped())
        .arg("--host")
        .arg("fusermount")
        .arg("-u")
        .arg(&vault_config.mount_directory)
        .spawn();

    match child {
        Ok(child) => {
            match child.wait_with_output() {
                Ok(output) => {
                    if output.status.success() {
                        log::debug!("Successfully closed vault");
                    } else {
                        match output.status.code() {
                            Some(status) => {
                                return Err(status_to_err(status));
                            }
                            None => {
                                return Err(BackendError::Generic);
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to wait for child: {}", e);
                    return Err(BackendError::Generic);
                }
            }

            Ok(())
        }
        Err(e) => {
            log::error!("Failed to close vault: {}", e);
            Err(BackendError::Generic)
        }
    }
}

fn status_to_err(status: i32) -> BackendError {
    struct GocryptfsExitStatus {}

    #[allow(dead_code)]
    impl GocryptfsExitStatus {
        pub const SUCCESS: i32 = 0;
        // TODO: Change to correct error code once gocryptfs 2.0 is out
        // see: https://github.com/rfjakob/gocryptfs/pull/503
        pub const INVALID_CIPHER_DIR: i32 = 6;
        pub const NON_EMPTY_CIPHER_DIR: i32 = 7;
        pub const NON_EMPTY_MOUNT_POINT: i32 = 10;
        pub const WRONG_PASSWORD: i32 = 12;
        pub const EMPTY_PASSWORD: i32 = 22;
        pub const CANNOT_READ_CONFIG: i32 = 23;
        pub const CANNOT_WRITE_CONFIG: i32 = 24;
        pub const FSCK_ERROR: i32 = 26;
    }

    match status {
        GocryptfsExitStatus::INVALID_CIPHER_DIR => {
            BackendError::ToUser(gettext("The encrypted data directory is not valid."))
        }
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
    }
}
