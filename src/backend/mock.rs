// mock.rs
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
//

use super::BackendError;
use super::backend::BackendImpl;
use crate::backend::Backend;
use crate::vault_config::VaultConfig;
use gettextrs::gettext;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Prefix of the environment variables that configure the mock backend.
///
/// Every `MockSettings` field can be overridden through an environment
/// variable of the form `VAULTS_MOCK_<FIELD>` (e.g., `VAULTS_MOCK_INIT_FAILS`),
/// which allows test harnesses to steer a running development instance of
/// Vaults without recompiling it.
pub const ENV_PREFIX: &str = "VAULTS_MOCK_";

/// Configurable behavior of the mock backend.
///
/// Each field simulates a specific aspect of a real encryption backend.
/// Settings can be composed through the builder methods, loaded from
/// environment variables (see [`MockSettings::from_env`]) or deserialized,
/// which makes the mock usable from in-process unit tests as well as from
/// external end-to-end test harnesses.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MockSettings {
    /// Simulates a missing backend binary, i.e., `is_available` returns `false`.
    pub unavailable: bool,

    /// Makes `init` fail with an error.
    pub init_fails: bool,

    /// Milliseconds `init` sleeps before returning, simulating a slow init.
    pub init_delay_ms: u64,

    /// Makes `open` fail with an error.
    pub open_fails: bool,

    /// Milliseconds `open` sleeps before returning, simulating a slow open.
    pub open_delay_ms: u64,

    /// Makes `close` fail with an error.
    pub close_fails: bool,

    /// Milliseconds `close` sleeps before returning, simulating a slow close.
    pub close_delay_ms: u64,

    /// When set, `init` and `open` only succeed with exactly this password,
    /// simulating a wrong password error otherwise.
    pub password: Option<String>,
}

// The builder and accessor methods are part of the testing API of the mock
// backend and are exercised by tests and future end-to-end harnesses rather
// than by the application itself.
#[allow(dead_code)]
impl MockSettings {
    /// Sets the mock backend to be unavailable.
    ///
    /// # Returns
    /// The modified settings.
    pub fn unavailable(mut self) -> Self {
        self.unavailable = true;
        self
    }

    /// Makes `init` fail.
    ///
    /// # Returns
    /// The modified settings.
    pub fn init_fails(mut self) -> Self {
        self.init_fails = true;
        self
    }

    /// Makes `init` sleep for the given duration.
    ///
    /// # Arguments
    /// * `ms`: Number of milliseconds `init` sleeps.
    ///
    /// # Returns
    /// The modified settings.
    pub fn with_init_delay(mut self, ms: u64) -> Self {
        self.init_delay_ms = ms;
        self
    }

    /// Makes `open` fail.
    ///
    /// # Returns
    /// The modified settings.
    pub fn open_fails(mut self) -> Self {
        self.open_fails = true;
        self
    }

    /// Makes `open` sleep for the given duration.
    ///
    /// # Arguments
    /// * `ms`: Number of milliseconds `open` sleeps.
    ///
    /// # Returns
    /// The modified settings.
    pub fn with_open_delay(mut self, ms: u64) -> Self {
        self.open_delay_ms = ms;
        self
    }

    /// Makes `close` fail.
    ///
    /// # Returns
    /// The modified settings.
    pub fn close_fails(mut self) -> Self {
        self.close_fails = true;
        self
    }

    /// Makes `close` sleep for the given duration.
    ///
    /// # Arguments
    /// * `ms`: Number of milliseconds `close` sleeps.
    ///
    /// # Returns
    /// The modified settings.
    pub fn with_close_delay(mut self, ms: u64) -> Self {
        self.close_delay_ms = ms;
        self
    }

    /// Restricts `init` and `open` to the given password.
    ///
    /// # Arguments
    /// * `password`: The only password accepted by `init` and `open`.
    ///
    /// # Returns
    /// The modified settings.
    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Reads the mock settings from the `VAULTS_MOCK_*` environment variables.
    ///
    /// Boolean variables are enabled by any of `1`, `true` or `yes`
    /// (case-insensitive), delay variables are parsed as milliseconds and
    /// `VAULTS_MOCK_PASSWORD` is taken verbatim.
    ///
    /// # Returns
    /// The settings assembled from the environment, with defaults for all
    /// unset variables.
    pub fn from_env() -> Self {
        Self::from_lookup(|key| std::env::var(key).ok().filter(|value| !value.is_empty()))
    }

    /// Assembles the settings from a lookup function, e.g., a map in tests.
    ///
    /// # Arguments
    /// * `lookup`: Function returning the value for a full environment
    ///   variable name, e.g., `VAULTS_MOCK_INIT_FAILS`.
    ///
    /// # Returns
    /// The settings assembled from the lookup results, with defaults for all
    /// missing keys.
    fn from_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        let mut settings = MockSettings::default();

        if let Some(value) = lookup(&format!("{ENV_PREFIX}UNAVAILABLE")) {
            settings.unavailable = parse_bool(&value);
        }
        if let Some(value) = lookup(&format!("{ENV_PREFIX}INIT_FAILS")) {
            settings.init_fails = parse_bool(&value);
        }
        if let Some(value) = lookup(&format!("{ENV_PREFIX}INIT_DELAY_MS"))
            && let Ok(ms) = value.parse::<u64>()
        {
            settings.init_delay_ms = ms;
        }
        if let Some(value) = lookup(&format!("{ENV_PREFIX}OPEN_FAILS")) {
            settings.open_fails = parse_bool(&value);
        }
        if let Some(value) = lookup(&format!("{ENV_PREFIX}OPEN_DELAY_MS"))
            && let Ok(ms) = value.parse::<u64>()
        {
            settings.open_delay_ms = ms;
        }
        if let Some(value) = lookup(&format!("{ENV_PREFIX}CLOSE_FAILS")) {
            settings.close_fails = parse_bool(&value);
        }
        if let Some(value) = lookup(&format!("{ENV_PREFIX}CLOSE_DELAY_MS"))
            && let Ok(ms) = value.parse::<u64>()
        {
            settings.close_delay_ms = ms;
        }
        if let Some(value) = lookup(&format!("{ENV_PREFIX}PASSWORD")) {
            settings.password = Some(value);
        }

        settings
    }
}

/// Parses an environment variable value as a boolean.
///
/// # Arguments
/// * `value`: The raw environment variable value.
///
/// # Returns
/// `true` for `1`, `true` and `yes` (case-insensitive), `false` otherwise.
fn parse_bool(value: &str) -> bool {
    matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes")
}

/// Internal state shared between all clones of a mock backend instance.
///
/// Which vaults have been initialized is persisted on disk through a
/// marker file inside the encrypted data directory (see
/// [`MARKER_FILE_NAME`]), so it survives restarts of the application.
/// Which vaults are currently open only lives in memory, since a mount
/// cannot outlive the process that created it.
#[derive(Debug, Default)]
struct MockState {
    open: HashSet<String>,
}

/// Name of the marker file the mock backend writes into the encrypted
/// data directory of a vault when it is initialized, mirroring the
/// configuration files real backends create (e.g. `gocryptfs.conf`).
const MARKER_FILE_NAME: &str = "mock.conf";

/// Returns the path of the marker file of the given encrypted data
/// directory.
///
/// # Arguments
/// * `encrypted_data_directory`: The encrypted data directory of the
///   vault.
///
/// # Returns
/// The path of the marker file inside the encrypted data directory.
fn marker_file_path(encrypted_data_directory: &str) -> PathBuf {
    Path::new(encrypted_data_directory).join(MARKER_FILE_NAME)
}

/// Returns whether the vault in the given encrypted data directory is
/// initialized, i.e. whether its marker file exists.
///
/// # Arguments
/// * `encrypted_data_directory`: The encrypted data directory of the
///   vault.
///
/// # Returns
/// `true` if the vault is initialized, `false` otherwise.
fn is_initialized(encrypted_data_directory: &str) -> bool {
    marker_file_path(encrypted_data_directory).exists()
}

/// Mock backend implementation for development and testing.
///
/// The mock simulates a real encryption backend without requiring any
/// external binary. Which operations fail and how long they take is
/// controlled through [`MockSettings`]. Initialization is persisted
/// through a marker file in the encrypted data directory, while the
/// currently open vaults are tracked in memory, so that opening an
/// uninitialized vault or closing a vault that is not open fails, just
/// like with a real backend.
///
/// The mock is only compiled in debug and test builds and never in release
/// builds.
#[derive(Debug, Clone)]
pub struct Mock {
    settings: MockSettings,
    state: Arc<Mutex<MockState>>,
}

impl Mock {
    /// Creates a mock backend with the given settings.
    ///
    /// # Arguments
    /// * `settings`: The behavior the mock backend simulates.
    ///
    /// # Returns
    /// A new mock backend.
    pub fn new(settings: MockSettings) -> Self {
        Self {
            settings,
            state: Arc::new(Mutex::new(MockState::default())),
        }
    }

    /// Returns the settings this mock backend simulates.
    ///
    /// # Returns
    /// A reference to the settings of this mock backend.
    #[allow(dead_code)]
    pub fn settings(&self) -> &MockSettings {
        &self.settings
    }

    /// Returns whether the given vault is currently open according to the
    /// mock state.
    ///
    /// # Arguments
    /// * `vault_config`: The configuration of the vault to check.
    ///
    /// # Returns
    /// `true` if the vault is open, `false` otherwise.
    #[allow(dead_code)]
    pub fn is_open(&self, vault_config: &VaultConfig) -> bool {
        log::trace!("Mock::is_open({:?})", vault_config);

        match self.state.lock() {
            Ok(state) => state.open.contains(&vault_config.mount_directory),
            Err(_) => false,
        }
    }

    /// Checks the password against the configured expected password.
    ///
    /// # Arguments
    /// * `password`: The password to check.
    ///
    /// # Returns
    /// `Ok(())` if the password matches or no password is configured,
    /// otherwise a `BackendError` describing the wrong password.
    fn check_password(&self, password: &str) -> Result<(), BackendError> {
        if let Some(expected) = &self.settings.password
            && password != expected
        {
            return Err(BackendError::ToUser(gettext("The password is wrong.")));
        }

        Ok(())
    }

    /// Locks the shared state and runs the given function with it.
    ///
    /// # Arguments
    /// * `f`: The function to run with the locked state.
    ///
    /// # Returns
    /// The result of `f` with the state locked, or a `BackendError` if the
    /// state lock is poisoned.
    fn with_state<T>(
        &self,
        f: impl FnOnce(&mut MockState) -> Result<T, BackendError>,
    ) -> Result<T, BackendError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| BackendError::ToUser(gettext("Mock backend state is poisoned.")))?;

        f(&mut state)
    }

    /// Simulates the duration of a backend operation.
    ///
    /// # Arguments
    /// * `ms`: Number of milliseconds to sleep, `0` returns immediately.
    fn delay(ms: u64) {
        if ms > 0 {
            std::thread::sleep(Duration::from_millis(ms));
        }
    }
}

impl Default for Mock {
    /// Creates a mock backend configured through the `VAULTS_MOCK_*`
    /// environment variables.
    ///
    /// # Returns
    /// A new mock backend with environment-configured settings.
    fn default() -> Self {
        Self::new(MockSettings::from_env())
    }
}

impl BackendImpl for Mock {
    fn is_available(&self) -> Result<bool, BackendError> {
        log::trace!("Mock::is_available()");

        Ok(!self.settings.unavailable)
    }

    fn init(&self, vault_config: &VaultConfig, password: &str) -> Result<(), BackendError> {
        log::trace!("Mock::init({:?}, password: <redacted>)", vault_config);

        Self::delay(self.settings.init_delay_ms);

        if self.settings.init_fails {
            log::error!("Mock init failed (configured)");
            return Err(BackendError::ToUser(gettext(
                "The mock backend failed to initialize the vault (as configured).",
            )));
        }

        self.check_password(password)?;

        let edd = vault_config.encrypted_data_directory.clone();
        if is_initialized(&edd) {
            return Err(BackendError::ToUser(gettext(
                "The encrypted data directory is not empty.",
            )));
        }

        std::fs::create_dir_all(&edd)?;
        std::fs::write(marker_file_path(&edd), "")?;

        Ok(())
    }

    fn open(&self, vault_config: &VaultConfig, password: &str) -> Result<(), BackendError> {
        log::trace!("Mock::open({:?}, password: <redacted>)", vault_config);

        Self::delay(self.settings.open_delay_ms);

        if self.settings.open_fails {
            log::error!("Mock open failed (configured)");
            return Err(BackendError::ToUser(gettext(
                "The mock backend failed to open the vault (as configured).",
            )));
        }

        self.check_password(password)?;

        let edd = vault_config.encrypted_data_directory.clone();
        if !is_initialized(&edd) {
            return Err(BackendError::ToUser(gettext(
                "The encrypted data directory is not an initialized vault.",
            )));
        }

        let md = vault_config.mount_directory.clone();
        self.with_state(|state| {
            if state.open.contains(&md) {
                return Err(BackendError::ToUser(gettext(
                    "The mount directory is already in use.",
                )));
            }

            state.open.insert(md);
            Ok(())
        })
    }

    fn close(&self, vault_config: &VaultConfig) -> Result<(), BackendError> {
        log::trace!("Mock::close({:?})", vault_config);

        Self::delay(self.settings.close_delay_ms);

        if self.settings.close_fails {
            log::error!("Mock close failed (configured)");
            return Err(BackendError::ToUser(gettext(
                "The mock backend failed to close the vault (as configured).",
            )));
        }

        let md = vault_config.mount_directory.clone();
        self.with_state(|state| {
            if !state.open.remove(&md) {
                return Err(BackendError::ToUser(gettext("The vault is not open.")));
            }

            Ok(())
        })
    }

    fn name(&self) -> String {
        String::from("mock")
    }

    fn kind(&self) -> Backend {
        Backend::Mock
    }

    fn clone_box(&self) -> Box<dyn BackendImpl> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::time::Instant;

    const TEST_ROOT: &str = "/tmp/vaults-mock-test";

    /// Returns a vault configuration with directories unique to the given
    /// test, removing any leftovers from previous runs so tests can be
    /// run repeatedly and in parallel.
    fn test_vault_config(name: &str) -> VaultConfig {
        let root = Path::new(TEST_ROOT).join(name);
        let _ = std::fs::remove_dir_all(&root);

        VaultConfig {
            name: String::from(name),
            backend: Backend::Mock.into(),
            encrypted_data_directory: root.join("encrypted-data").to_string_lossy().into_owned(),
            mount_directory: root.join("mount").to_string_lossy().into_owned(),
            session_lock: false,
        }
    }

    #[test]
    fn default_lifecycle_succeeds() {
        let mock = Mock::new(MockSettings::default());
        let vault_config = test_vault_config("default_lifecycle_succeeds");

        assert!(mock.is_available().unwrap());
        mock.init(&vault_config, "pw").unwrap();
        mock.open(&vault_config, "pw").unwrap();
        assert!(mock.is_open(&vault_config));
        mock.close(&vault_config).unwrap();
        assert!(!mock.is_open(&vault_config));
    }

    #[test]
    fn unavailable() {
        let mock = Mock::new(MockSettings::default().unavailable());

        assert!(!mock.is_available().unwrap());
    }

    #[test]
    fn init_fails() {
        let mock = Mock::new(MockSettings::default().init_fails());
        let vault_config = test_vault_config("init_fails");

        assert!(mock.init(&vault_config, "pw").is_err());
    }

    #[test]
    fn open_fails() {
        let mock = Mock::new(MockSettings::default().open_fails());
        let vault_config = test_vault_config("open_fails");

        mock.init(&vault_config, "pw").unwrap();
        assert!(mock.open(&vault_config, "pw").is_err());
    }

    #[test]
    fn close_fails() {
        let mock = Mock::new(MockSettings::default().close_fails());
        let vault_config = test_vault_config("close_fails");

        mock.init(&vault_config, "pw").unwrap();
        mock.open(&vault_config, "pw").unwrap();
        assert!(mock.close(&vault_config).is_err());
    }

    #[test]
    fn wrong_password() {
        let mock = Mock::new(MockSettings::default().with_password("secret"));
        let vault_config = test_vault_config("wrong_password");

        assert!(mock.init(&vault_config, "wrong").is_err());
        mock.init(&vault_config, "secret").unwrap();

        assert!(mock.open(&vault_config, "wrong").is_err());
        mock.open(&vault_config, "secret").unwrap();

        mock.close(&vault_config).unwrap();
    }

    #[test]
    fn delays() {
        let mock = Mock::new(
            MockSettings::default()
                .with_init_delay(100)
                .with_open_delay(100)
                .with_close_delay(100),
        );
        let vault_config = test_vault_config("delays");

        let start = Instant::now();
        mock.init(&vault_config, "pw").unwrap();
        assert!(start.elapsed() >= Duration::from_millis(100));

        let start = Instant::now();
        mock.open(&vault_config, "pw").unwrap();
        assert!(start.elapsed() >= Duration::from_millis(100));

        let start = Instant::now();
        mock.close(&vault_config).unwrap();
        assert!(start.elapsed() >= Duration::from_millis(100));
    }

    #[test]
    fn state_errors() {
        let mock = Mock::new(MockSettings::default());
        let vault_config = test_vault_config("state_errors");

        assert!(mock.open(&vault_config, "pw").is_err());
        assert!(mock.close(&vault_config).is_err());

        mock.init(&vault_config, "pw").unwrap();
        assert!(mock.init(&vault_config, "pw").is_err());

        mock.open(&vault_config, "pw").unwrap();
        assert!(mock.open(&vault_config, "pw").is_err());

        mock.close(&vault_config).unwrap();
        assert!(mock.close(&vault_config).is_err());
    }

    #[test]
    fn clones_share_state() {
        let mock = Mock::new(MockSettings::default());
        let clone = mock.clone_box();
        let vault_config = test_vault_config("clones_share_state");

        clone.init(&vault_config, "pw").unwrap();
        mock.open(&vault_config, "pw").unwrap();
        assert!(mock.clone().is_open(&vault_config));

        assert_eq!(clone.kind(), Backend::Mock);
        assert_eq!(clone.name(), String::from("mock"));
    }

    #[test]
    fn initialization_persists_across_instances() {
        let first = Mock::new(MockSettings::default());
        let second = Mock::new(MockSettings::default());
        let vault_config = test_vault_config("initialization_persists_across_instances");

        first.init(&vault_config, "pw").unwrap();
        assert!(!first.is_open(&vault_config));

        second.open(&vault_config, "pw").unwrap();
        second.close(&vault_config).unwrap();
    }

    #[test]
    fn settings_from_lookup() {
        let vars: HashMap<String, String> = [
            (format!("{ENV_PREFIX}UNAVAILABLE"), String::from("yes")),
            (format!("{ENV_PREFIX}INIT_FAILS"), String::from("1")),
            (format!("{ENV_PREFIX}INIT_DELAY_MS"), String::from("500")),
            (format!("{ENV_PREFIX}OPEN_FAILS"), String::from("true")),
            (format!("{ENV_PREFIX}OPEN_DELAY_MS"), String::from("150")),
            (format!("{ENV_PREFIX}CLOSE_FAILS"), String::from("no")),
            (format!("{ENV_PREFIX}PASSWORD"), String::from("secret")),
        ]
        .into_iter()
        .collect();

        let settings = MockSettings::from_lookup(|key| vars.get(key).cloned());

        assert_eq!(
            settings,
            MockSettings::default()
                .unavailable()
                .init_fails()
                .with_init_delay(500)
                .open_fails()
                .with_open_delay(150)
                .with_password("secret")
        );
        assert!(!settings.close_fails);
    }

    #[test]
    fn settings_from_lookup_defaults() {
        let settings = MockSettings::from_lookup(|_| None);

        assert_eq!(settings, MockSettings::default());
    }

    #[test]
    fn settings_serde_roundtrip() {
        let settings = MockSettings::default()
            .init_fails()
            .with_init_delay(42)
            .open_fails()
            .with_open_delay(43)
            .close_fails()
            .with_close_delay(44)
            .with_password("secret");

        let json = serde_json::to_string(&settings).unwrap();
        assert_eq!(
            serde_json::from_str::<MockSettings>(&json).unwrap(),
            settings
        );
    }
}
