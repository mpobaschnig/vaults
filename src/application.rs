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

use gio::ApplicationFlags;
use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use gtk_macros::action;
use log::{debug, info};
use std::cell::RefCell;
use adw::subclass::prelude::*;

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
        fn activate(&self, app: &Self::Type) {
            debug!("GtkApplication<VApplication>::activate");

            if let Some(ref window) = *self.window.borrow() {
                window.present();
                return;
            }

            app.set_resource_base_path(Some("/io/github/mpobaschnig/Vaults/"));

            let window = ApplicationWindow::new(app);

            window.present();

            self.window.replace(Some(window));

            app.setup_gactions();
        }

        fn startup(&self, app: &Self::Type) {
            debug!("GtkApplication<VApplication>::startup");
            self.parent_startup(app);
        }
    }

    impl GtkApplicationImpl for VApplication {}
    impl AdwApplicationImpl for VApplication {}
}

glib::wrapper! {
    pub struct VApplication(ObjectSubclass<imp::VApplication>)
    @extends gio::Application, gtk::Application, adw::Application, @implements gio::ActionMap, gio::ActionGroup;
}

impl VApplication {
    pub fn new() -> Self {
        glib::Object::new(&[
            ("application-id", &Some(config::APP_ID)),
            ("flags", &ApplicationFlags::empty()),
        ])
        .expect("Application initialization failed...")
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
    }

    fn show_preferences(&self) {
        let preferences = PreferencesWindow::new();

        preferences.show();
    }

    fn show_about_dialog(&self) {
        let dialog = gtk::AboutDialogBuilder::new()
            .program_name("Vaults")
            .logo_icon_name(config::APP_ID)
            .license_type(gtk::License::Gpl30)
            .website("https://github.com/mpobaschnig/Vaults")
            .version(config::VERSION)
            .transient_for(&self.active_window().unwrap())
            .modal(true)
            .authors(vec!["Martin Pobaschnig".into()])
            .artists(vec!["Martin Pobaschnig".into()])
            .build();

        dialog.show();
    }

    pub fn run(&self) {
        info!("Vaults ({})", config::APP_ID);
        info!("Version: {} ({})", config::VERSION, config::PROFILE);
        info!("Datadir: {}", config::PKGDATADIR);

        ApplicationExtManual::run(self);
    }
}
