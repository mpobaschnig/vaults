// add_new_vault_dialog.rs
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

use std::str::FromStr;

use adw::{subclass::prelude::*, ActionRowExt};
use gettextrs::gettext;
use gtk::{self, prelude::*};
use gtk::{glib, CompositeTemplate};
use gtk::{glib::clone, subclass::prelude::*};

use crate::{
    backend::{Backend, AVAILABLE_BACKENDS},
    vault::Vault,
};

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/gitlab/mpobaschnig/Vaults/add_new_vault_dialog.ui")]
    pub struct AddNewVaultDialog {
        #[template_child]
        pub cancel_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub add_new_vault_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub vault_name_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub backend_type_combo_box_text: TemplateChild<gtk::ComboBoxText>,
        #[template_child]
        pub password_action_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub password_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub password_confirm_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub encrypted_data_directory_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub encrypted_data_directory_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub mount_directory_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub mount_directory_button: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AddNewVaultDialog {
        const NAME: &'static str = "AddNewVaultDialog";
        type ParentType = gtk::Dialog;
        type Type = super::AddNewVaultDialog;

        fn new() -> Self {
            Self {
                cancel_button: TemplateChild::default(),
                add_new_vault_button: TemplateChild::default(),
                vault_name_entry: TemplateChild::default(),
                backend_type_combo_box_text: TemplateChild::default(),
                password_action_row: TemplateChild::default(),
                password_entry: TemplateChild::default(),
                password_confirm_entry: TemplateChild::default(),
                encrypted_data_directory_entry: TemplateChild::default(),
                encrypted_data_directory_button: TemplateChild::default(),
                mount_directory_entry: TemplateChild::default(),
                mount_directory_button: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AddNewVaultDialog {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.setup_actions();
            obj.setup_signals();

            obj.fill_combo_box_text();
        }
    }

    impl WidgetImpl for AddNewVaultDialog {}
    impl WindowImpl for AddNewVaultDialog {}
    impl DialogImpl for AddNewVaultDialog {}
}

glib::wrapper! {
    pub struct AddNewVaultDialog(ObjectSubclass<imp::AddNewVaultDialog>)
        @extends gtk::Widget, gtk::Window, gtk::Dialog;
}

impl AddNewVaultDialog {
    pub fn new(parent: &gtk::Window) -> Self {
        let dialog: Self = glib::Object::new(&[("use-header-bar", &1)])
            .expect("Failed to create AddNewVaultDialog");

        dialog.set_transient_for(Some(parent));

        dialog
    }

    fn setup_actions(&self) {}

    fn setup_signals(&self) {
        let self_ = imp::AddNewVaultDialog::from_instance(self);

        self_
            .cancel_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.response(gtk::ResponseType::Cancel);
            }));

        self_
            .add_new_vault_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.response(gtk::ResponseType::Ok);
            }));

        self_
            .vault_name_entry
            .connect_property_text_notify(clone!(@weak self as obj => move |_| {
                obj.check_add_button_enable_conditions();
            }));

        self_
            .password_entry
            .connect_property_text_notify(clone!(@weak self as obj => move |_| {
                obj.check_add_button_enable_conditions();
            }));

        self_.password_confirm_entry.connect_property_text_notify(
            clone!(@weak self as obj => move |_| {
                obj.check_add_button_enable_conditions();
            }),
        );

        self_
            .encrypted_data_directory_entry
            .connect_property_text_notify(clone!(@weak self as obj => move |_| {
                obj.check_add_button_enable_conditions();
            }));

        self_.encrypted_data_directory_button.connect_clicked(
            clone!(@weak self as obj => move |_| {
                println!("Encrypted Data Directory button clicked!");
            }),
        );

        self_.mount_directory_entry.connect_property_text_notify(
            clone!(@weak self as obj => move |_| {
                obj.check_add_button_enable_conditions();
            }),
        );

        self_
            .mount_directory_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                println!("Mount Directory button clicked!");
            }));
    }

    fn check_add_button_enable_conditions(&self) {
        let self_ = imp::AddNewVaultDialog::from_instance(self);

        let vault_name = self_.vault_name_entry.get_text();
        let backend = self_.backend_type_combo_box_text.get_active();
        let password = self_.password_entry.get_text();
        let confirm_password = self_.password_confirm_entry.get_text();
        let encrypted_data_directory = self_.encrypted_data_directory_entry.get_text();
        let mount_directory = self_.mount_directory_entry.get_text();

        if !vault_name.is_empty()
            && !password.is_empty()
            && !confirm_password.is_empty()
            && !encrypted_data_directory.is_empty()
            && !mount_directory.is_empty()
            && backend.is_some()
        {
            if password.eq(&confirm_password) {
                self_.add_new_vault_button.set_sensitive(true);
                self_.password_action_row.set_subtitle(Some(""));
            } else {
                self_.add_new_vault_button.set_sensitive(false);
                self_
                    .password_action_row
                    .set_subtitle(Some(&gettext("Passwords are not equal!")));
            }
        } else {
            if password.eq(&confirm_password) {
                self_.password_action_row.set_subtitle(Some(""));
            } else {
                self_
                    .password_action_row
                    .set_subtitle(Some(&gettext("Passwords are not equal!")));
            }
            self_.add_new_vault_button.set_sensitive(false);
        }
    }

    pub fn get_entry_values(&self) -> (String, String, String, String, String) {
        let self_ = imp::AddNewVaultDialog::from_instance(self);

        let vault_name = String::from(self_.vault_name_entry.get_text().as_str());
        let backend_type = String::from(
            self_
                .backend_type_combo_box_text
                .get_active_text()
                .unwrap()
                .as_str(),
        );
        let password = String::from(self_.password_entry.get_text().as_str());
        let encrypted_data_directory =
            String::from(self_.encrypted_data_directory_entry.get_text().as_str());
        let mount_directory = String::from(self_.mount_directory_entry.get_text().as_str());

        (
            vault_name,
            backend_type,
            password,
            encrypted_data_directory,
            mount_directory,
        )
    }

    pub fn get_vault(&self) -> Vault {
        let self_ = imp::AddNewVaultDialog::from_instance(self);

        Vault {
            name: String::from(self_.password_entry.get_text().as_str()),
            backend: Backend::from_str(
                self_
                    .backend_type_combo_box_text
                    .get_active_text()
                    .unwrap()
                    .as_str(),
            )
            .unwrap(),
            encrypted_data_directory: String::from(
                self_.encrypted_data_directory_entry.get_text().as_str(),
            ),
            mount_directory: String::from(self_.mount_directory_entry.get_text().as_str()),
        }
    }

    fn fill_combo_box_text(&self) {
        let self_ = imp::AddNewVaultDialog::from_instance(self);

        let combo_box_text = &self_.backend_type_combo_box_text;

        let backends_res = AVAILABLE_BACKENDS.lock();
        match backends_res {
            Ok(backends) => {
                for backend in backends.iter() {
                    combo_box_text.append_text(backend);
                }

                if !backends.is_empty() {
                    combo_box_text.set_active(Some(0));
                }
            }
            Err(e) => {
                log::error!("Failed to aquire mutex lock of AVAILABLE_BACKENDS: {}", e);
            }
        }
    }
}
