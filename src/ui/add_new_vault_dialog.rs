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

use crate::{
    backend::{Backend, AVAILABLE_BACKENDS},
    user_config_manager::UserConnfigManager,
    vault::*,
    VApplication,
};
use adw::{subclass::prelude::*, ActionRowExt};
use gettextrs::gettext;
use gtk::gio;
use gtk::{self, prelude::*};
use gtk::{glib, CompositeTemplate};
use gtk::{glib::clone, glib::GString, subclass::prelude::*};

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/mpobaschnig/Vaults/add_new_vault_dialog.ui")]
    pub struct AddNewVaultDialog {
        #[template_child]
        pub cancel_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub add_new_vault_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub vault_name_action_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub vault_name_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub backend_type_action_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub backend_type_combo_box_text: TemplateChild<gtk::ComboBoxText>,
        #[template_child]
        pub password_action_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub password_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub password_confirm_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub encrypted_data_directory_action_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub encrypted_data_directory_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub encrypted_data_directory_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub mount_directory_action_row: TemplateChild<adw::ActionRow>,
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
                vault_name_action_row: TemplateChild::default(),
                vault_name_entry: TemplateChild::default(),
                backend_type_action_row: TemplateChild::default(),
                backend_type_combo_box_text: TemplateChild::default(),
                password_action_row: TemplateChild::default(),
                password_entry: TemplateChild::default(),
                password_confirm_entry: TemplateChild::default(),
                encrypted_data_directory_action_row: TemplateChild::default(),
                encrypted_data_directory_entry: TemplateChild::default(),
                encrypted_data_directory_button: TemplateChild::default(),
                mount_directory_action_row: TemplateChild::default(),
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

            obj.setup_combo_box();
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
    pub fn new() -> Self {
        let dialog: Self = glib::Object::new(&[("use-header-bar", &1)])
            .expect("Failed to create AddNewVaultDialog");

        let window = gio::Application::get_default()
            .unwrap()
            .downcast_ref::<VApplication>()
            .unwrap()
            .get_active_window()
            .unwrap();
        dialog.set_transient_for(Some(&window));

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
            .backend_type_combo_box_text
            .connect_changed(clone!(@weak self as obj => move |_| {
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
                obj.encrypted_data_directory_button_clicked();
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
                obj.mount_directory_button_clicked();
            }));
    }

    fn encrypted_data_directory_button_clicked(&self) {
        let dialog = gtk::FileChooserDialog::new(
            Some(&gettext("Choose Encrypted Data Directory")),
            Some(self),
            gtk::FileChooserAction::SelectFolder,
            &[
                (&gettext("Cancel"), gtk::ResponseType::Cancel),
                (&gettext("Select"), gtk::ResponseType::Accept),
            ],
        );

        dialog.set_transient_for(Some(self));

        dialog.connect_response(clone!(@weak self as obj => move |dialog, response| {
            if response == gtk::ResponseType::Accept {
                let file = dialog.get_file().unwrap();
                let path = String::from(file.get_path().unwrap().as_os_str().to_str().unwrap());
                let self_ = imp::AddNewVaultDialog::from_instance(&obj);
                self_.encrypted_data_directory_entry.set_text(&path);
            }

            dialog.destroy();
        }));

        dialog.show();
    }

    fn mount_directory_button_clicked(&self) {
        let dialog = gtk::FileChooserDialog::new(
            Some(&gettext("Choose Mount Directory")),
            Some(self),
            gtk::FileChooserAction::SelectFolder,
            &[
                (&gettext("Cancel"), gtk::ResponseType::Cancel),
                (&gettext("Select"), gtk::ResponseType::Accept),
            ],
        );

        dialog.set_transient_for(Some(self));

        dialog.connect_response(clone!(@weak self as obj => move |dialog, response| {
            if response == gtk::ResponseType::Accept {
                let file = dialog.get_file().unwrap();
                let path = String::from(file.get_path().unwrap().as_os_str().to_str().unwrap());
                let self_ = imp::AddNewVaultDialog::from_instance(&obj);
                self_.mount_directory_entry.set_text(&path);
            }

            dialog.destroy();
        }));

        dialog.show();
    }

    fn is_valid_vault_name(&self, vault_name: GString) -> bool {
        let self_ = imp::AddNewVaultDialog::from_instance(self);

        if vault_name.is_empty() {
            self_
                .vault_name_action_row
                .set_subtitle(Some(&gettext("Name is not valid.")));
            false
        } else {
            self_.vault_name_action_row.set_subtitle(Some(""));
            true
        }
    }

    fn is_different_vault_name(&self, vault_name: GString) -> bool {
        let self_ = imp::AddNewVaultDialog::from_instance(self);

        let is_duplicate_name = UserConnfigManager::instance()
            .get_map()
            .contains_key(&vault_name.to_string());
        if !vault_name.is_empty() && is_duplicate_name {
            self_
                .vault_name_action_row
                .set_subtitle(Some(&gettext("Name already exists.")));
            false
        } else {
            true
        }
    }

    fn is_path_empty(&self, path: &GString) -> Result<bool, std::io::Error> {
        match std::fs::read_dir(path.to_string()) {
            Ok(dir) => {
                if dir.count() > 0 {
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            Err(e) => {
                log::debug!("Could not read path {}: {}", path, e);
                Err(e)
            }
        }
    }

    fn is_encrypted_data_directory_valid(&self, encrypted_data_directory: &GString) -> bool {
        let self_ = imp::AddNewVaultDialog::from_instance(self);

        match self.is_path_empty(encrypted_data_directory) {
            Ok(is_empty) => {
                if is_empty {
                    self_
                        .encrypted_data_directory_action_row
                        .set_subtitle(Some(&gettext("")));
                    true
                } else {
                    self_
                        .encrypted_data_directory_action_row
                        .set_subtitle(Some(&gettext("Directory is not empty.")));
                    false
                }
            }
            Err(_) => {
                self_
                    .encrypted_data_directory_action_row
                    .set_subtitle(Some(&gettext("Directory is not valid.")));
                false
            }
        }
    }

    fn is_mount_directory_valid(&self, mount_directory: &GString) -> bool {
        let self_ = imp::AddNewVaultDialog::from_instance(self);

        match self.is_path_empty(mount_directory) {
            Ok(is_empty) => {
                if is_empty {
                    self_
                        .mount_directory_action_row
                        .set_subtitle(Some(&gettext("")));
                    true
                } else {
                    self_
                        .mount_directory_action_row
                        .set_subtitle(Some(&gettext("Directory is not empty.")));
                    false
                }
            }
            Err(_) => {
                self_
                    .mount_directory_action_row
                    .set_subtitle(Some(&gettext("Directory is not valid.")));
                false
            }
        }
    }

    fn are_directories_different(
        &self,
        encrypted_data_directory: &GString,
        mount_directory: &GString,
    ) -> bool {
        let self_ = imp::AddNewVaultDialog::from_instance(self);

        if encrypted_data_directory.eq(mount_directory) {
            self_
                .encrypted_data_directory_action_row
                .set_subtitle(Some(&gettext("Directories must not be equal.")));
            self_
                .mount_directory_action_row
                .set_subtitle(Some(&gettext("Directories must not be equal.")));
            false
        } else {
            true
        }
    }

    fn are_passwords_empty(&self, password: &GString, confirm_password: &GString) -> bool {
        let self_ = imp::AddNewVaultDialog::from_instance(self);

        if password.is_empty() && confirm_password.is_empty() {
            self_
                .password_action_row
                .set_subtitle(Some(&gettext("Password is empty")));
            true
        } else {
            self_.password_action_row.set_subtitle(Some(&gettext("")));
            false
        }
    }

    fn are_passwords_equal(&self, password: &GString, confirm_password: &GString) -> bool {
        let self_ = imp::AddNewVaultDialog::from_instance(self);

        if password.eq(confirm_password) {
            self_.password_action_row.set_subtitle(Some(""));
            true
        } else {
            self_
                .password_action_row
                .set_subtitle(Some(&gettext("Passwords are not equal!")));
            false
        }
    }

    fn check_add_button_enable_conditions(&self) {
        let self_ = imp::AddNewVaultDialog::from_instance(self);

        let vault_name = self_.vault_name_entry.get_text();
        let password = self_.password_entry.get_text();
        let confirm_password = self_.password_confirm_entry.get_text();
        let encrypted_data_directory = self_.encrypted_data_directory_entry.get_text();
        let mount_directory = self_.mount_directory_entry.get_text();

        let is_valid_vault_name = self.is_valid_vault_name(vault_name.clone());
        let is_different_vault_name = self.is_different_vault_name(vault_name);
        let is_encrypted_data_directory_valid =
            self.is_encrypted_data_directory_valid(&encrypted_data_directory);
        let is_mount_directory_valid = self.is_mount_directory_valid(&mount_directory);
        let are_directories_different =
            if is_encrypted_data_directory_valid && is_mount_directory_valid {
                self.are_directories_different(&encrypted_data_directory, &mount_directory)
            } else {
                false
            };
        let are_passwords_empty = self.are_passwords_empty(&password, &confirm_password);
        let are_passwords_equal = if !are_passwords_empty {
            self.are_passwords_equal(&password, &confirm_password)
        } else {
            false
        };

        if is_valid_vault_name
            && is_different_vault_name
            && is_encrypted_data_directory_valid
            && is_mount_directory_valid
            && are_directories_different
            && are_passwords_equal
        {
            self_.add_new_vault_button.set_sensitive(true);
        } else {
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

        Vault::new(
            String::from(self_.vault_name_entry.get_text().as_str()),
            Backend::from_str(
                self_
                    .backend_type_combo_box_text
                    .get_active_text()
                    .unwrap()
                    .as_str(),
            )
            .unwrap(),
            String::from(self_.encrypted_data_directory_entry.get_text().as_str()),
            String::from(self_.mount_directory_entry.get_text().as_str()),
        )
    }

    pub fn get_password(&self) -> String {
        let self_ = imp::AddNewVaultDialog::from_instance(self);
        String::from(self_.password_entry.get_text().as_str())
    }

    fn setup_combo_box(&self) {
        let self_ = imp::AddNewVaultDialog::from_instance(self);

        let combo_box_text = &self_.backend_type_combo_box_text;

        if let Ok(available_backends) = AVAILABLE_BACKENDS.lock() {
            let mut gocryptfs_index: Option<u32> = None;

            for (i, backend) in available_backends.iter().enumerate() {
                if backend.eq("Gocryptfs") {
                    gocryptfs_index = Some(i as u32);
                }

                combo_box_text.append_text(backend);
            }

            if !available_backends.is_empty() {
                if let Some(index) = gocryptfs_index {
                    combo_box_text.set_active(Some(index));
                } else {
                    combo_box_text.set_active(Some(0));
                }
            } else {
                self_.backend_type_action_row.set_subtitle(Some(&gettext(
                    "No backend is installed. Please install gocryptfs or cryfs.",
                )));
            }
        }
    }
}
