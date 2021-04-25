// vaults_page_row_settings_dialog.rs
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
use gtk::{
    self, gio, glib, glib::clone, glib::GString, prelude::*, subclass::prelude::*,
    CompositeTemplate,
};
use std::cell::RefCell;
use strum::IntoEnumIterator;

use crate::{backend::Backend, user_config_manager::UserConnfigManager, vault::*, VApplication};

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/mpobaschnig/Vaults/vaults_page_row_settings_dialog.ui")]
    pub struct VaultsPageRowSettingsDialog {
        #[template_child]
        pub cancel_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub delete_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub save_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub vault_name_action_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub vault_name_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub backend_type_combo_box_text: TemplateChild<gtk::ComboBoxText>,
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

        pub current_vault: RefCell<Option<Vault>>,
        pub to_delete: RefCell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VaultsPageRowSettingsDialog {
        const NAME: &'static str = "VaultsPageRowSettingsDialog";
        type ParentType = gtk::Dialog;
        type Type = super::VaultsPageRowSettingsDialog;

        fn new() -> Self {
            Self {
                cancel_button: TemplateChild::default(),
                delete_button: TemplateChild::default(),
                save_button: TemplateChild::default(),
                vault_name_action_row: TemplateChild::default(),
                vault_name_entry: TemplateChild::default(),
                backend_type_combo_box_text: TemplateChild::default(),
                encrypted_data_directory_action_row: TemplateChild::default(),
                encrypted_data_directory_entry: TemplateChild::default(),
                encrypted_data_directory_button: TemplateChild::default(),
                mount_directory_action_row: TemplateChild::default(),
                mount_directory_entry: TemplateChild::default(),
                mount_directory_button: TemplateChild::default(),
                current_vault: RefCell::new(None),
                to_delete: RefCell::new(false),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for VaultsPageRowSettingsDialog {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for VaultsPageRowSettingsDialog {}
    impl WindowImpl for VaultsPageRowSettingsDialog {}
    impl DialogImpl for VaultsPageRowSettingsDialog {}
}

glib::wrapper! {
    pub struct VaultsPageRowSettingsDialog(ObjectSubclass<imp::VaultsPageRowSettingsDialog>)
        @extends gtk::Widget, gtk::Window, gtk::Dialog;
}

impl VaultsPageRowSettingsDialog {
    pub fn new(vault: Vault) -> Self {
        let dialog: Self = glib::Object::new(&[("use-header-bar", &1)])
            .expect("Failed to create VaultsPageRowSettingsDialog");

        let window = gio::Application::get_default()
            .unwrap()
            .downcast_ref::<VApplication>()
            .unwrap()
            .get_active_window()
            .unwrap();
        dialog.set_transient_for(Some(&window));

        dialog.set_vault(vault);

        dialog.setup_signals();

        dialog.fill_combo_box_text();

        dialog
    }

    fn setup_signals(&self) {
        let self_ = imp::VaultsPageRowSettingsDialog::from_instance(self);

        self_
            .cancel_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.response(gtk::ResponseType::Cancel);
            }));

        self_
            .delete_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.delete_button_clicked();
            }));

        self_
            .save_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.save_button_clicked();
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

    fn delete_button_clicked(&self) {
        UserConnfigManager::instance().remove_vault(self.get_vault());
        self.response(gtk::ResponseType::Other(0));
    }

    fn save_button_clicked(&self) {
        let self_ = imp::VaultsPageRowSettingsDialog::from_instance(self);

        let new_vault = Vault::new(
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
        );

        UserConnfigManager::instance().change_vault(self.get_current_vault().unwrap(), new_vault);

        self.response(gtk::ResponseType::Other(1));
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
                let self_ = imp::VaultsPageRowSettingsDialog::from_instance(&obj);
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
                let self_ = imp::VaultsPageRowSettingsDialog::from_instance(&obj);
                self_.mount_directory_entry.set_text(&path);
            }

            dialog.destroy();
        }));

        dialog.show();
    }

    fn is_valid_vault_name(&self, vault_name: GString) -> bool {
        let self_ = imp::VaultsPageRowSettingsDialog::from_instance(self);

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
        let self_ = imp::VaultsPageRowSettingsDialog::from_instance(self);

        let is_same_name = vault_name.eq(&self.get_current_vault().unwrap().get_name().unwrap());
        let is_duplicate_name = UserConnfigManager::instance()
            .get_map()
            .contains_key(&vault_name.to_string());
        if !vault_name.is_empty() && !is_same_name && is_duplicate_name {
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
        let self_ = imp::VaultsPageRowSettingsDialog::from_instance(self);

        match self.is_path_empty(encrypted_data_directory) {
            Ok(is_empty) => {
                if is_empty {
                    self_
                        .encrypted_data_directory_action_row
                        .set_subtitle(Some(&gettext("Directory is empty.")));
                    false
                } else {
                    self_
                        .encrypted_data_directory_action_row
                        .set_subtitle(Some(&gettext("")));
                    true
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
        let self_ = imp::VaultsPageRowSettingsDialog::from_instance(self);

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
        let self_ = imp::VaultsPageRowSettingsDialog::from_instance(self);

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

    fn has_something_changed(
        &self,
        curr_vault_name: &GString,
        curr_backend: &GString,
        curr_encrypted_data_directory: &GString,
        curr_mount_directory: &GString,
    ) -> bool {
        let prev_vault = self.get_current_vault().unwrap();
        let prev_config = &prev_vault.get_config().unwrap();

        let prev_vault_name = &prev_vault.get_name().unwrap();
        let prev_backend = &prev_config.backend.to_string();
        let prev_encrypted_data_directory = &prev_config.encrypted_data_directory;
        let prev_mount_directory = &prev_config.mount_directory;

        if !curr_vault_name.eq(prev_vault_name) {
            return true;
        }

        if !curr_backend.eq(prev_backend) {
            return true;
        }

        if !curr_encrypted_data_directory.eq(prev_encrypted_data_directory) {
            return true;
        }

        if !curr_mount_directory.eq(prev_mount_directory) {
            return true;
        }

        false
    }

    fn check_add_button_enable_conditions(&self) {
        let self_ = imp::VaultsPageRowSettingsDialog::from_instance(self);

        let vault_name = self_.vault_name_entry.get_text();
        let backend = self_.backend_type_combo_box_text.get_active_text().unwrap();
        let encrypted_data_directory = self_.encrypted_data_directory_entry.get_text();
        let mount_directory = self_.mount_directory_entry.get_text();

        let is_valid_vault_name = self.is_valid_vault_name(vault_name.clone());
        let is_different_vault_name = self.is_different_vault_name(vault_name.clone());
        let is_encrypted_data_directory_valid =
            self.is_encrypted_data_directory_valid(&encrypted_data_directory);
        let is_mount_directory_valid = self.is_mount_directory_valid(&mount_directory);
        let are_directories_different =
            if is_encrypted_data_directory_valid && is_mount_directory_valid {
                self.are_directories_different(&encrypted_data_directory, &mount_directory)
            } else {
                false
            };
        let has_something_changed = self.has_something_changed(
            &vault_name,
            &backend,
            &encrypted_data_directory,
            &mount_directory,
        );

        if is_valid_vault_name
            && is_different_vault_name
            && is_encrypted_data_directory_valid
            && is_mount_directory_valid
            && are_directories_different
            && has_something_changed
        {
            self_.save_button.set_sensitive(true);
        } else {
            self_.save_button.set_sensitive(false);
        }
    }

    fn fill_combo_box_text(&self) {
        let self_ = imp::VaultsPageRowSettingsDialog::from_instance(self);

        let curr_backend = self
            .get_current_vault()
            .unwrap()
            .get_config()
            .unwrap()
            .backend
            .to_string();

        let combo_box_text = &self_.backend_type_combo_box_text;

        for (i, backend) in Backend::iter().enumerate() {
            let backend = backend.to_string();

            combo_box_text.append_text(&backend);

            if backend.eq(&curr_backend) {
                combo_box_text.set_active(Some(i as u32));
            }
        }
    }

    pub fn get_vault(&self) -> Vault {
        let self_ = imp::VaultsPageRowSettingsDialog::from_instance(self);

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

    pub fn get_current_vault(&self) -> Option<Vault> {
        let self_ = imp::VaultsPageRowSettingsDialog::from_instance(self);

        self_.current_vault.borrow().clone()
    }

    pub fn set_vault(&self, vault: Vault) {
        let self_ = imp::VaultsPageRowSettingsDialog::from_instance(&self);

        match (vault.get_name(), vault.get_config()) {
            (Some(name), Some(config)) => {
                self_.current_vault.replace(Some(vault.clone()));

                self_.vault_name_entry.set_text(&name);
                self_
                    .backend_type_combo_box_text
                    .set_active_id(Some(&config.backend.to_string()));
                self_
                    .encrypted_data_directory_entry
                    .set_text(&config.encrypted_data_directory.to_string());
                self_
                    .mount_directory_entry
                    .set_text(&config.mount_directory.to_string());
            }
            (_, _) => {
                log::error!("Vault not initalised!");
                return;
            }
        }
    }
}
