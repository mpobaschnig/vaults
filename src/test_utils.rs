// test_utils.rs
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

/// Initializes GTK for testing, registers all types, and loads the compiled resources.
/// This function should be called once per test process before creating any GTK widgets.
pub fn init_gtk() {
    gtk::init().expect("Failed to initialize GTK");
    gtk::test_register_all_types();

    // Load the compiled GTK resources (same as main.rs)
    let res = gtk::gio::Resource::load(crate::config::RESOURCES_FILE)
        .expect("Could not load gresource file");
    gtk::gio::resources_register(&res);
}
