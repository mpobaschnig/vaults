// vaults_page_row_settings_window.rs
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

use adw::subclass::prelude::*;
use gtk::{self, gio, glib, glib::clone, prelude::*, CompositeTemplate};

use crate::VApplication;

mod imp {
    use gtk::glib::subclass::Signal;
    use once_cell::sync::Lazy;

    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(
        resource = "/io/github/mpobaschnig/Vaults/vaults_page_row_password_prompt_window.ui"
    )]
    pub struct VaultsPageRowPasswordPromptWindow {
        #[template_child]
        pub unlock_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub password_entry_row: TemplateChild<adw::PasswordEntryRow>,
        #[template_child]
        pub status_page: TemplateChild<adw::StatusPage>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VaultsPageRowPasswordPromptWindow {
        const NAME: &'static str = "VaultsPageRowPasswordPromptWindow";
        type ParentType = adw::Window;
        type Type = super::VaultsPageRowPasswordPromptWindow;

        fn new() -> Self {
            Self {
                unlock_button: TemplateChild::default(),
                password_entry_row: TemplateChild::default(),
                status_page: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for VaultsPageRowPasswordPromptWindow {
        fn constructed(&self) {
            let obj = self.obj();
            self.parent_constructed();

            obj.setup_signals();
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("unlock").build()]);
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for VaultsPageRowPasswordPromptWindow {}
    impl AdwWindowImpl for VaultsPageRowPasswordPromptWindow {}
    impl WindowImpl for VaultsPageRowPasswordPromptWindow {}
    impl DialogImpl for VaultsPageRowPasswordPromptWindow {}
}

glib::wrapper! {
    pub struct VaultsPageRowPasswordPromptWindow(ObjectSubclass<imp::VaultsPageRowPasswordPromptWindow>)
        @extends gtk::Widget, adw::Window, gtk::Window;
}

impl VaultsPageRowPasswordPromptWindow {
    pub fn new() -> Self {
        let dialog: Self = glib::Object::builder().build();

        dialog.add_css_class("flat");

        if let Some(window) = gio::Application::default()
            .unwrap()
            .downcast_ref::<VApplication>()
            .unwrap()
            .active_window()
        {
            dialog.set_transient_for(Some(&window));
        }

        dialog
    }

    fn setup_signals(&self) {
        self.imp()
            .unlock_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.unlock_button_clicked();
            }));

        self.imp()
            .password_entry_row
            .connect_text_notify(clone!(@weak self as obj => move |_| {
                obj.check_unlock_button_enable_conditions();
            }));

        self.imp()
            .password_entry_row
            .connect_activate(clone!(@weak self as obj => move |_| {
                obj.connect_activate();
            }));
    }

    pub fn set_name(&self, name: &String) {
        self.imp().status_page.set_title(name);
    }

    fn unlock_button_clicked(&self) {
        self.emit_by_name::<()>("unlock", &[]);
        //self.close();
    }

    fn check_unlock_button_enable_conditions(&self) {
        let vault_name = self.imp().password_entry_row.text();

        if !vault_name.is_empty() {
            self.imp().unlock_button.set_sensitive(true);
        } else {
            self.imp().unlock_button.set_sensitive(false);
        }
    }

    fn connect_activate(&self) {
        if !self.imp().password_entry_row.text().is_empty() {
            self.unlock_button_clicked();
        }
    }

    pub fn get_password(&self) -> String {
        self.imp().password_entry_row.text().to_string()
    }
}
