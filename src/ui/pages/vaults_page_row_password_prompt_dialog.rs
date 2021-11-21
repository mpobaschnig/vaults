// vaults_page_row_settings_dialog.rs
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
use gtk::{self, gio, glib, glib::clone, prelude::*, subclass::prelude::*, CompositeTemplate};

use crate::VApplication;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(
        resource = "/io/github/mpobaschnig/Vaults/vaults_page_row_password_prompt_dialog.ui"
    )]
    pub struct VaultsPageRowPasswordPromptDialog {
        #[template_child]
        pub cancel_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub unlock_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub password_entry: TemplateChild<gtk::Entry>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VaultsPageRowPasswordPromptDialog {
        const NAME: &'static str = "VaultsPageRowPasswordPromptDialog";
        type ParentType = gtk::Dialog;
        type Type = super::VaultsPageRowPasswordPromptDialog;

        fn new() -> Self {
            Self {
                cancel_button: TemplateChild::default(),
                unlock_button: TemplateChild::default(),
                password_entry: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for VaultsPageRowPasswordPromptDialog {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.setup_signals();

            self.password_entry.grab_focus_without_selecting();
        }
    }

    impl WidgetImpl for VaultsPageRowPasswordPromptDialog {}
    impl WindowImpl for VaultsPageRowPasswordPromptDialog {}
    impl DialogImpl for VaultsPageRowPasswordPromptDialog {}
}

glib::wrapper! {
    pub struct VaultsPageRowPasswordPromptDialog(ObjectSubclass<imp::VaultsPageRowPasswordPromptDialog>)
        @extends gtk::Widget, gtk::Window, gtk::Dialog;
}

impl VaultsPageRowPasswordPromptDialog {
    pub fn new() -> Self {
        let dialog: Self = glib::Object::new(&[("use-header-bar", &1)])
            .expect("Failed to create VaultsPageRowPasswordPromptDialog");

        let window = gio::Application::default()
            .unwrap()
            .downcast_ref::<VApplication>()
            .unwrap()
            .active_window()
            .unwrap();
        dialog.set_transient_for(Some(&window));

        dialog
    }

    fn setup_signals(&self) {
        let self_ = imp::VaultsPageRowPasswordPromptDialog::from_instance(self);

        self_
            .cancel_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.cancel_button_clicked();
            }));

        self_
            .unlock_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.unlock_button_clicked();
            }));

        self_
            .password_entry
            .connect_text_notify(clone!(@weak self as obj => move |_| {
                obj.check_unlock_button_enable_conditions();
            }));

        self_
            .password_entry
            .connect_activate(clone!(@weak self as obj => move |_| {
                obj.connect_activate();
            }));
    }

    fn cancel_button_clicked(&self) {
        self.response(gtk::ResponseType::Cancel);
    }

    fn unlock_button_clicked(&self) {
        self.response(gtk::ResponseType::Ok);
    }

    fn check_unlock_button_enable_conditions(&self) {
        let self_ = imp::VaultsPageRowPasswordPromptDialog::from_instance(self);

        let vault_name = self_.password_entry.text();

        if !vault_name.is_empty() {
            self_.unlock_button.set_sensitive(true);
        } else {
            self_.unlock_button.set_sensitive(false);
        }
    }

    fn connect_activate(&self) {
        let self_ = imp::VaultsPageRowPasswordPromptDialog::from_instance(self);

        if !self_.password_entry.text().is_empty() {
            self.unlock_button_clicked();
        }
    }

    pub fn get_password(&self) -> String {
        let self_ = imp::VaultsPageRowPasswordPromptDialog::from_instance(self);

        self_.password_entry.text().to_string()
    }
}
