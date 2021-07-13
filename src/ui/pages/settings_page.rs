// settings_page.rs
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

use crate::backend::Backend;
use crate::ui::pages::VaultsPageRow;
use crate::ui::ApplicationWindow;
use crate::user_config_manager::UserConfigManager;
use crate::vault::Vault;
use crate::VApplication;
use adw::subclass::prelude::BinImpl;
use gettextrs::gettext;
use glib::once_cell::sync::Lazy;
use glib::subclass;
use glib::GString;
use gtk::gio;
use gtk::glib;
use gtk::glib::clone;
use gtk::glib::subclass::Signal;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::cell::RefCell;
use std::str::FromStr;
use strum::IntoEnumIterator;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/mpobaschnig/Vaults/settings_page.ui")]
    pub struct SettingsPage {
        #[template_child]
        pub remove_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub save_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub vault_name_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub backend_type_combo_box_text: TemplateChild<gtk::ComboBoxText>,
        #[template_child]
        pub encrypted_data_directory_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub encrypted_data_directory_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub mount_directory_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub mount_directory_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub info_label: TemplateChild<gtk::Label>,

        pub current_vault: RefCell<Option<Vault>>,

        pub current_row: RefCell<Option<VaultsPageRow>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SettingsPage {
        const NAME: &'static str = "SettingsPage";
        type ParentType = adw::Bin;
        type Type = super::SettingsPage;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SettingsPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            for backend in Backend::iter() {
                let backend = backend.to_string();

                self.backend_type_combo_box_text.append_text(&backend);
            }
            self.backend_type_combo_box_text.set_active(Some(0));

            obj.setup_signals();
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("save", &[], glib::Type::UNIT.into()).build(),
                    Signal::builder("remove", &[], glib::Type::UNIT.into()).build(),
                ]
            });

            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for SettingsPage {}

    impl BinImpl for SettingsPage {}
}

glib::wrapper! {
    pub struct SettingsPage(ObjectSubclass<imp::SettingsPage>)
        @extends gtk::Widget, adw::Bin;
}

impl SettingsPage {
    pub fn connect_save<F: Fn() + 'static>(&self, callback: F) -> glib::SignalHandlerId {
        self.connect_local("save", false, move |_| {
            callback();
            None
        })
        .unwrap()
    }

    pub fn connect_remove<F: Fn() + 'static>(&self, callback: F) -> glib::SignalHandlerId {
        self.connect_local("remove", false, move |_| {
            callback();
            None
        })
        .unwrap()
    }

    fn setup_signals(&self) {
        let self_ = imp::SettingsPage::from_instance(self);

        self_
            .save_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.save_button_clicked();
            }));

        self_
            .remove_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.remove_button_clicked();
            }));

        self_
            .vault_name_entry
            .connect_text_notify(clone!(@weak self as obj => move |_| {
                obj.check_add_button_enable_conditions();
            }));

        self_
            .backend_type_combo_box_text
            .connect_changed(clone!(@weak self as obj => move |_| {
                obj.check_add_button_enable_conditions();
            }));

        self_.encrypted_data_directory_entry.connect_text_notify(
            clone!(@weak self as obj => move |_| {
                obj.check_add_button_enable_conditions();
            }),
        );

        self_.encrypted_data_directory_button.connect_clicked(
            clone!(@weak self as obj => move |_| {
                obj.encrypted_data_directory_button_clicked();
            }),
        );

        self_
            .mount_directory_entry
            .connect_text_notify(clone!(@weak self as obj => move |_| {
                obj.check_add_button_enable_conditions();
            }));

        self_
            .mount_directory_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.mount_directory_button_clicked();
            }));
    }

    pub fn init(&self) {
        let self_ = imp::SettingsPage::from_instance(&self);

        self_.save_button.set_sensitive(false);
    }

    fn save_button_clicked(&self) {
        println!("Save!");
        self.emit_by_name("save", &[]).unwrap();
    }

    fn remove_button_clicked(&self) {
        self.emit_by_name("remove", &[]).unwrap();
    }

    pub fn call_settings(&self, row: &VaultsPageRow) {
        let self_ = imp::SettingsPage::from_instance(&self);

        self_.current_row.borrow_mut().replace(row.clone());

        let vault = row.get_vault();
        let vault_config = vault.get_config().unwrap();

        self_.current_vault.replace(Some(vault.clone()));

        self_.vault_name_entry.set_text(&vault.get_name().unwrap());
        for (i, backend) in Backend::iter().enumerate() {
            let backend = backend.to_string();

            if backend.eq(&vault_config.backend.to_string()) {
                self_.backend_type_combo_box_text.set_active(Some(i as u32));
            }
        }
        self_
            .encrypted_data_directory_entry
            .set_text(&vault_config.encrypted_data_directory);
        self_
            .mount_directory_entry
            .set_text(&vault_config.mount_directory);

        UserConfigManager::instance().set_current_vault(vault);

        let ancestor = self.ancestor(ApplicationWindow::static_type()).unwrap();
        let window = ancestor.downcast_ref::<ApplicationWindow>().unwrap();
        window.set_settings_page();

        row.set_save_handler_id(self.connect_save(
            clone!(@weak self as obj, @weak row, @weak window => move || {
                let obj_ = imp::SettingsPage::from_instance(&obj);

                obj.disconnect_all_signals();

                let new_vault = Vault::new(
                    String::from(obj_.vault_name_entry.text().as_str()),
                    Backend::from_str(
                        obj_
                            .backend_type_combo_box_text
                            .active_text()
                            .unwrap()
                            .as_str(),
                    )
                    .unwrap(),
                    String::from(obj_.encrypted_data_directory_entry.text().as_str()),
                    String::from(obj_.mount_directory_entry.text().as_str()),
                );

                let vault = &row.get_vault();
                if !vault.is_backend_available() {
                    row.set_vault_row_state_backend_unavailable();
                } else {
                    row.set_vault_row_state_backend_available();
                }

                UserConfigManager::instance().change_vault(row.get_vault(), new_vault);

                row.emit_by_name("save", &[]).unwrap();

                let ancestor = obj.ancestor(ApplicationWindow::static_type()).unwrap();
                let window = ancestor.downcast_ref::<ApplicationWindow>().unwrap();
                window.set_standard_window_view();
            }),
        ));

        row.set_remove_handler_id(self.connect_remove(
            clone!(@weak self as obj, @weak row => move || {
                obj.disconnect_all_signals();

                let vault = row.get_vault();

                UserConfigManager::instance().remove_vault(vault);

                row.emit_by_name("remove", &[]).unwrap();

                let ancestor = obj.ancestor(ApplicationWindow::static_type()).unwrap();
                let window = ancestor.downcast_ref::<ApplicationWindow>().unwrap();
                window.set_standard_window_view();
            }),
        ));
    }

    fn is_valid_vault_name(&self, vault_name: GString) -> bool {
        let self_ = imp::SettingsPage::from_instance(self);

        if vault_name.is_empty() {
            self_
                .info_label
                .set_text(&gettext("Vault name is not valid."));
            false
        } else {
            true
        }
    }

    fn is_different_vault_name(&self, vault_name: GString) -> bool {
        let self_ = imp::SettingsPage::from_instance(self);

        let is_same_name = vault_name.eq(&self_
            .current_vault
            .borrow()
            .clone()
            .unwrap()
            .get_name()
            .unwrap());
        let is_duplicate_name = UserConfigManager::instance()
            .get_map()
            .contains_key(&vault_name.to_string());
        if !vault_name.is_empty() && !is_same_name && is_duplicate_name {
            self_
                .info_label
                .set_text(&gettext("Vault name already exists."));
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
        let self_ = imp::SettingsPage::from_instance(self);

        if encrypted_data_directory.is_empty() {
            return false;
        }

        match self.is_path_empty(encrypted_data_directory) {
            Ok(is_empty) => {
                if is_empty {
                    self_
                        .info_label
                        .set_text(&gettext("Encrypted data directory is empty."));
                    false
                } else {
                    true
                }
            }
            Err(_) => {
                self_
                    .info_label
                    .set_text(&gettext("Encrypted data directory is not valid."));
                false
            }
        }
    }

    fn is_mount_directory_valid(&self, mount_directory: &GString) -> bool {
        let self_ = imp::SettingsPage::from_instance(self);

        if mount_directory.is_empty() {
            return false;
        }

        if !std::path::Path::exists(std::path::Path::new(mount_directory)) {
            return true;
        }

        match self.is_path_empty(mount_directory) {
            Ok(is_empty) => {
                if is_empty {
                    true
                } else {
                    self_
                        .info_label
                        .set_text(&gettext("Mount directory is not empty."));
                    false
                }
            }
            Err(_) => {
                self_
                    .info_label
                    .set_text(&gettext("Mount directory is not valid."));
                false
            }
        }
    }

    fn are_directories_different(
        &self,
        encrypted_data_directory: &GString,
        mount_directory: &GString,
    ) -> bool {
        let self_ = imp::SettingsPage::from_instance(self);

        if encrypted_data_directory.eq(mount_directory) {
            self_
                .info_label
                .set_text(&gettext("Directories must be different."));
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
        let self_ = imp::SettingsPage::from_instance(self);

        let prev_vault = self_.current_vault.borrow().clone().unwrap();
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

    fn exists_config_file(&self, backend: Backend, encrypted_data_directory: &GString) -> bool {
        let self_ = imp::SettingsPage::from_instance(self);

        if !self.is_encrypted_data_directory_valid(&encrypted_data_directory) {
            return false;
        }

        let mut path_str = encrypted_data_directory.to_string();

        match backend {
            Backend::Cryfs => {
                path_str.push_str("/cryfs.config");
            }
            Backend::Gocryptfs => {
                path_str.push_str("/gocryptfs.conf");
            }
        }

        let path = std::path::Path::new(&path_str);
        if path.exists() {
            true
        } else {
            self_
                .info_label
                .set_text(&gettext("No configuration file found."));
            false
        }
    }

    fn check_add_button_enable_conditions(&self) {
        let self_ = imp::SettingsPage::from_instance(&self);

        self_.info_label.set_text("");
        self_.save_button.set_sensitive(false);

        let vault_name = self_.vault_name_entry.text();
        let backend_str = self_.backend_type_combo_box_text.active_text().unwrap();
        let backend = Backend::from_str(&backend_str.as_str()).unwrap();
        let encrypted_data_directory = self_.encrypted_data_directory_entry.text();
        let mount_directory = self_.mount_directory_entry.text();

        let is_valid_vault_name = self.is_valid_vault_name(vault_name.clone());
        if !is_valid_vault_name {
            return;
        }

        let is_different_vault_name = self.is_different_vault_name(vault_name.clone());
        if !is_different_vault_name {
            return;
        }

        let is_encrypted_data_directory_valid =
            self.is_encrypted_data_directory_valid(&encrypted_data_directory);
        if !is_encrypted_data_directory_valid {
            return;
        }

        let is_mount_directory_valid = self.is_mount_directory_valid(&mount_directory);
        if !is_mount_directory_valid {
            return;
        }

        let are_directories_different =
            if is_encrypted_data_directory_valid && is_mount_directory_valid {
                self.are_directories_different(&encrypted_data_directory, &mount_directory)
            } else {
                false
            };
        if !are_directories_different {
            return;
        }

        let exists_config_file = self.exists_config_file(backend, &encrypted_data_directory);
        if !exists_config_file {
            return;
        }

        let has_something_changed = self.has_something_changed(
            &vault_name,
            &backend_str,
            &encrypted_data_directory,
            &mount_directory,
        );
        if !has_something_changed {
            return;
        }

        self_.save_button.set_sensitive(true);
    }

    fn encrypted_data_directory_button_clicked(&self) {
        let window = gio::Application::default()
            .unwrap()
            .downcast_ref::<VApplication>()
            .unwrap()
            .active_window()
            .unwrap();

        let dialog = gtk::FileChooserDialog::new(
            Some(&gettext("Choose Encrypted Data Directory")),
            Some(&window),
            gtk::FileChooserAction::SelectFolder,
            &[
                (&gettext("Cancel"), gtk::ResponseType::Cancel),
                (&gettext("Select"), gtk::ResponseType::Accept),
            ],
        );

        dialog.set_transient_for(Some(&window));

        dialog.connect_response(clone!(@weak self as obj => move |dialog, response| {
            if response == gtk::ResponseType::Accept {
                let file = dialog.file().unwrap();
                let path = String::from(file.path().unwrap().as_os_str().to_str().unwrap());
                let self_ = imp::SettingsPage::from_instance(&obj);
                self_.encrypted_data_directory_entry.set_text(&path);
            }

            dialog.destroy();
        }));

        dialog.show();
    }

    fn mount_directory_button_clicked(&self) {
        let window = gio::Application::default()
            .unwrap()
            .downcast_ref::<VApplication>()
            .unwrap()
            .active_window()
            .unwrap();

        let dialog = gtk::FileChooserDialog::new(
            Some(&gettext("Choose Mount Directory")),
            Some(&window),
            gtk::FileChooserAction::SelectFolder,
            &[
                (&gettext("Cancel"), gtk::ResponseType::Cancel),
                (&gettext("Select"), gtk::ResponseType::Accept),
            ],
        );

        dialog.set_transient_for(Some(&window));

        dialog.connect_response(clone!(@weak self as obj => move |dialog, response| {
            if response == gtk::ResponseType::Accept {
                let file = dialog.file().unwrap();
                let path = String::from(file.path().unwrap().as_os_str().to_str().unwrap());
                let self_ = imp::SettingsPage::from_instance(&obj);
                self_.mount_directory_entry.set_text(&path);
            }

            dialog.destroy();
        }));

        dialog.show();
    }

    pub fn disconnect_all_signals(&self) {
        let self_ = imp::SettingsPage::from_instance(&self);

        if let Some(row) = self_.current_row.borrow().as_ref() {
            self.disconnect(row.get_save_handler_id());
            self.disconnect(row.get_remove_handler_id());
        }
    }
}
