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

use adw::subclass::prelude::*;
use gtk::{self, prelude::*};
use gtk::{glib, CompositeTemplate};
use gtk::{glib::clone, subclass::prelude::*};

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
            obj.connect_handlers();
            obj.setup_actions();
            obj.setup_signals();
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

    fn connect_handlers(&self) {
        self.connect_response(Self::handle_response);
    }

    fn handle_response(&self, id: gtk::ResponseType) {
        match id {
            gtk::ResponseType::Ok => {}
            _ => {
                self.destroy();
            }
        }
    }

    fn setup_actions(&self) {}

    fn setup_signals(&self) {
        let self_ = imp::AddNewVaultDialog::from_instance(self);

        self_
            .cancel_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.destroy();
            }));

        self_
            .add_new_vault_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                println!("Add New Vault button clicked!");
            }));

        self_
            .vault_name_entry
            .connect_property_text_notify(clone!(@weak self as obj => move |_| {
                obj.enable_add_button_if_entries_not_empty();
            }));

        self_
            .password_entry
            .connect_property_text_notify(clone!(@weak self as obj => move |_| {
                obj.enable_add_button_if_entries_not_empty();
            }));

        self_.password_confirm_entry.connect_property_text_notify(
            clone!(@weak self as obj => move |_| {
                obj.enable_add_button_if_entries_not_empty();
            }),
        );

        self_
            .encrypted_data_directory_entry
            .connect_property_text_notify(clone!(@weak self as obj => move |_| {
                obj.enable_add_button_if_entries_not_empty();
            }));

        self_.encrypted_data_directory_button.connect_clicked(
            clone!(@weak self as obj => move |_| {
                println!("Encrypted Data Directory button clicked!");
            }),
        );

        self_.mount_directory_entry.connect_property_text_notify(
            clone!(@weak self as obj => move |_| {
                obj.enable_add_button_if_entries_not_empty();
            }),
        );

        self_
            .mount_directory_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                println!("Mount Directory button clicked!");
            }));
    }

    fn enable_add_button_if_entries_not_empty(&self) {
        let self_ = imp::AddNewVaultDialog::from_instance(self);

        let vault_name = self_.vault_name_entry.get_text();
        let password = self_.password_entry.get_text();
        let confirm_password = self_.password_confirm_entry.get_text();
        let encrypted_data_directory = self_.encrypted_data_directory_entry.get_text();
        let mount_directory = self_.mount_directory_entry.get_text();

        if !vault_name.is_empty()
            && !password.is_empty()
            && !confirm_password.is_empty()
            && !encrypted_data_directory.is_empty()
            && !mount_directory.is_empty()
        {
            self_.add_new_vault_button.set_sensitive(true);
        } else {
            self_.add_new_vault_button.set_sensitive(false);
        }
    }
}
