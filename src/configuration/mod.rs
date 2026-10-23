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

#[cfg(test)]
mod tests;

pub use reader::read_config;
pub use writer::write_config;
