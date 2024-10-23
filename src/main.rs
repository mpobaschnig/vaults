// main.rs
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

mod application;
#[rustfmt::skip]
mod config;
mod global_config_manager;
mod user_config_manager;
mod vault;

mod backend;
mod ui;

mod nova;

#[macro_use]
extern crate quick_error;
extern crate proc_mounts;
extern crate serde;
extern crate toml;

use application::VApplication;
use config::{GETTEXT_PACKAGE, LOCALEDIR, RESOURCES_FILE};
use gettextrs::*;
use global_config_manager::GlobalConfigManager;
use gtk::gio;
use user_config_manager::UserConfigManager;

fn main() {
    pretty_env_logger::init();

    setlocale(LocaleCategory::LcAll, "");

    if let Err(e) = bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR) {
        log::error!("Could not bind text domain: {}", e);
    }

    if let Err(e) = textdomain(GETTEXT_PACKAGE) {
        log::error!("Could not set text domain: {}", e);
    }

    GlobalConfigManager::instance().read_config();

    UserConfigManager::instance().read_config();

    gtk::glib::set_application_name("Vaults");
    gtk::glib::set_prgname(Some("vaults"));

    gtk::init().expect("Unable to start GTK4");

    let res = gio::Resource::load(RESOURCES_FILE).expect("Could not load gresource file");
    gio::resources_register(&res);

    let app = VApplication::new();
    app.run();
}
