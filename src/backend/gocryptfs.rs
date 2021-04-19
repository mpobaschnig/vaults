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
use crate::vault::Vault;
use std::process::Command;
use std::{io::Write, process::Stdio};

pub fn is_available() -> bool {
    let output_res = Command::new("gocryptfs").arg("--version").output();

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

pub fn init(vault: &Vault, password: String) -> Result<(), BackendError> {
    let child = Command::new("gocryptfs")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .arg("--init")
        .arg("-q")
        .arg("--")
        .arg(vault.get_config().unwrap().encrypted_data_directory)
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
                                    log::info!("Failed to init vault");
                                    return Err(BackendError::GenericError);
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to wait for child: {}", e);
                                return Err(BackendError::GenericError);
                            }
                        },
                        Err(e) => {
                            log::error!("Failed to write to stdin: {}", e);
                            return Err(BackendError::GenericError);
                        }
                    }
                }
                None => {
                    log::error!("Could not get stdin of child!");
                    return Err(BackendError::GenericError);
                }
            }

            Ok(())
        }
        Err(e) => {
            log::error!("Failed to init vault: {}", e);
            Err(BackendError::GenericError)
        }
    }
}

pub fn open(vault: &Vault, password: String) -> Result<(), BackendError> {
    let child = Command::new("flatpak-spawn")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .arg("--host")
        .arg("gocryptfs")
        .arg("-q")
        .arg("--")
        .arg(vault.get_config().unwrap().encrypted_data_directory)
        .arg(vault.get_config().unwrap().mount_directory)
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
                                    log::error!(
                                        "Failed to open vault {}",
                                        std::str::from_utf8(&output.stdout).unwrap()
                                    );
                                    return Err(BackendError::GenericError);
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to wait for child: {}", e);
                                return Err(BackendError::GenericError);
                            }
                        },
                        Err(e) => {
                            log::error!("Failed to write to stdin: {}", e);
                            return Err(BackendError::GenericError);
                        }
                    }
                }
                None => {
                    log::error!("Could not get stdin of child!");
                    return Err(BackendError::GenericError);
                }
            }

            Ok(())
        }
        Err(e) => {
            log::error!("Failed to init vault: {}", e);
            Err(BackendError::GenericError)
        }
    }
}

pub fn close(vault: &Vault) -> Result<(), BackendError> {
    let child = Command::new("flatpak-spawn")
        .stdout(Stdio::piped())
        .arg("--host")
        .arg("fusermount")
        .arg("-u")
        .arg(vault.get_config().unwrap().mount_directory)
        .spawn();

    match child {
        Ok(child) => {
            match child.wait_with_output() {
                Ok(output) => {
                    if output.status.success() {
                        log::debug!("Successfully closed vault");
                    } else {
                        log::error!(
                            "Failed to close vault {}",
                            std::str::from_utf8(&output.stdout).unwrap()
                        );
                        return Err(BackendError::GenericError);
                    }
                }
                Err(e) => {
                    log::error!("Failed to wait for child: {}", e);
                    return Err(BackendError::GenericError);
                }
            }

            Ok(())
        }
        Err(e) => {
            log::error!("Failed to close vault: {}", e);
            Err(BackendError::GenericError)
        }
    }
}
