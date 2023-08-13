// application.rs
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

use crate::config;
use crate::ui::ApplicationWindow;
use crate::ui::PreferencesWindow;

use adw::subclass::prelude::*;
use gio::ApplicationFlags;
use glib::clone;
use gtk::prelude::*;
use gtk::{gio, glib};
use gtk_macros::action;
use log::{debug, info};
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct VApplication {
        pub window: RefCell<Option<ApplicationWindow>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VApplication {
        const NAME: &'static str = "VApplication";
        type Type = super::VApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for VApplication {}

    impl gio::subclass::prelude::ApplicationImpl for VApplication {
        fn activate(&self) {
            debug!("GtkApplication<VApplication>::activate");

            if let Some(ref window) = *self.window.borrow() {
                window.present();
                return;
            }

            let app = self.obj();

            app.setup_accels();

            app.set_resource_base_path(Some("/io/github/mpobaschnig/Vaults/"));

            let window = ApplicationWindow::new(&app);

            window.present();

            self.window.replace(Some(window));

            app.setup_gactions();
        }

        fn startup(&self) {
            debug!("GtkApplication<VApplication>::startup");
            self.parent_startup();
        }
    }

    impl GtkApplicationImpl for VApplication {}
    impl AdwApplicationImpl for VApplication {}
}

glib::wrapper! {
    pub struct VApplication(ObjectSubclass<imp::VApplication>)
    @extends gio::Application, gtk::ApplicationWindow, gtk::Application, adw::Application, @implements gio::ActionMap, gio::ActionGroup;
}

impl VApplication {
    pub fn new() -> Self {
        let object: Self = glib::Object::new();
        object.set_property("application-id", config::APP_ID);
        object.set_property("flags", ApplicationFlags::empty());
        object
    }

    fn setup_gactions(&self) {
        action!(
            self,
            "preferences",
            clone!(@weak self as obj => move |_, _| {
                obj.show_preferences();
            })
        );

        action!(
            self,
            "about",
            clone!(@weak self as obj => move |_, _| {
                obj.show_about_dialog();
            })
        );

        action!(
            self,
            "quit",
            clone!(@weak self as obj => move |_, _| {
                obj.quit();
            })
        );
    }

    fn setup_accels(&self) {
        self.set_accels_for_action("win.add_new_vault", &["<primary>a"]);
        self.set_accels_for_action("win.import_vault", &["<primary>i"]);
        self.set_accels_for_action("win.search", &["<primary>f"]);
        self.set_accels_for_action("win.escape", &["Escape"]);
        self.set_accels_for_action("win.refresh", &["<primary>r"]);

        self.set_accels_for_action("app.preferences", &["<primary>p"]);
        self.set_accels_for_action("win.show-help-overlay", &["<primary>question"]);
        self.set_accels_for_action("app.quit", &["<primary>q"]);
    }

    fn show_preferences(&self) {
        let preferences = PreferencesWindow::new();

        preferences.set_visible(true);
    }

    fn show_about_dialog(&self) {
        let about_window = adw::AboutWindow::new();

        about_window.set_application_icon(config::APP_ID);
        about_window.set_application_name("Vaults");
        about_window.set_artists(&["Martin Pobaschnig", "Jacson Hilgert"]);
        about_window.set_copyright("© 2022 Martin Pobaschnig");
        about_window.set_developer_name("Martin Pobschnig");
        about_window.set_issue_url("https://github.com/mpobaschnig/Vaults/issues");
        about_window.set_license_type(gtk::License::Gpl30);
        about_window.set_modal(true);
        about_window.set_support_url("https://github.com/mpobaschnig/Vaults/discussions");
        about_window.set_transient_for(Some(&self.active_window().unwrap()));
        about_window.set_version(config::VERSION);
        about_window.set_website("https://github.com/mpobaschnig/Vaults");

        about_window.set_visible(true);
    }

    pub fn run(&self) {
        info!("Vaults ({})", config::APP_ID);
        info!("Version: {} ({})", config::VERSION, config::PROFILE);
        info!("Datadir: {}", config::PKGDATADIR);

        ApplicationExtManual::run(self);
    }
}
