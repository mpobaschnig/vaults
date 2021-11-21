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

use gio::ApplicationFlags;
use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib};
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
        type ParentType = gtk::Application;
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
            app.setup_css();

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
}

glib::wrapper! {
    pub struct VApplication(ObjectSubclass<imp::VApplication>)
        @extends gio::Application, gtk::Application, @implements gio::ActionMap, gio::ActionGroup;
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
            "about",
            clone!(@weak self as obj => move |_, _| {
                obj.show_about_dialog();
            })
        );
    }

    fn setup_css(&self) {
        let provider = gtk::CssProvider::new();
        provider.load_from_resource("/io/github/mpobaschnig/Vaults/style.css");
        if let Some(display) = gdk::Display::get_default() {
            gtk::StyleContext::add_provider_for_display(
                &display,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    }

    fn show_about_dialog(&self) {
        let dialog = gtk::AboutDialogBuilder::new()
            .program_name("Vaults")
            .logo_icon_name(config::APP_ID)
            .license_type(gtk::License::Gpl30)
            .website("https://github.com/mpobaschnig/Vaults")
            .version(config::VERSION)
            .transient_for(&self.get_active_window().unwrap())
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
