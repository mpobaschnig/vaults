// add_page.rs
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

use crate::backend::*;
use crate::password_manager::PasswordManager;
use crate::user_config_manager::UserConfigManager;
use crate::vault::*;
use crate::VApplication;
use adw::{subclass::prelude::*, ActionRowExt};
use gettextrs::gettext;
use glib::clone;
use glib::once_cell::sync::Lazy;
use gtk::gio::{self};
use gtk::glib;
use gtk::glib::subclass::Signal;
use gtk::glib::GString;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::cell::RefCell;
use std::str::FromStr;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/mpobaschnig/Vaults/add_page.ui")]
    pub struct AddPage {
        #[template_child]
        pub carousel: TemplateChild<adw::Carousel>,
        #[template_child]
        pub add_new_vault_action_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub import_new_vault_action_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub vault_name_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub backend_type_combo_box_text: TemplateChild<gtk::ComboBoxText>,
        #[template_child]
        pub backend_type_info_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub info_revealer: TemplateChild<gtk::Revealer>,
        #[template_child]
        pub info_revealer_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub password_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub password_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub confirm_password_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub confirm_password_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub previous_button_p_2: TemplateChild<gtk::Button>,
        #[template_child]
        pub next_button_p_2: TemplateChild<gtk::Button>,
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
        #[template_child]
        pub previous_button_p_3: TemplateChild<gtk::Button>,
        #[template_child]
        pub add_import_button: TemplateChild<gtk::Button>,

        pub is_add_new_vault: RefCell<Option<bool>>,
        pub name: RefCell<Option<String>>,
        pub backend_type: RefCell<Option<Backend>>,
        pub password: RefCell<Option<String>>,
        pub encrypted_data_directory: RefCell<Option<String>>,
        pub mount_directory: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AddPage {
        const NAME: &'static str = "AddPage";
        type ParentType = adw::Bin;
        type Type = super::AddPage;

        fn new() -> Self {
            Self {
                carousel: TemplateChild::default(),
                add_new_vault_action_row: TemplateChild::default(),
                import_new_vault_action_row: TemplateChild::default(),
                vault_name_entry: TemplateChild::default(),
                backend_type_combo_box_text: TemplateChild::default(),
                backend_type_info_button: TemplateChild::default(),
                info_revealer: TemplateChild::default(),
                info_revealer_label: TemplateChild::default(),
                password_label: TemplateChild::default(),
                password_entry: TemplateChild::default(),
                confirm_password_label: TemplateChild::default(),
                confirm_password_entry: TemplateChild::default(),
                previous_button_p_2: TemplateChild::default(),
                next_button_p_2: TemplateChild::default(),
                encrypted_data_directory_entry: TemplateChild::default(),
                encrypted_data_directory_button: TemplateChild::default(),
                mount_directory_entry: TemplateChild::default(),
                mount_directory_button: TemplateChild::default(),
                info_label: TemplateChild::default(),
                previous_button_p_3: TemplateChild::default(),
                add_import_button: TemplateChild::default(),
                is_add_new_vault: RefCell::new(None),
                name: RefCell::new(None),
                backend_type: RefCell::new(None),
                password: RefCell::new(None),
                encrypted_data_directory: RefCell::new(None),
                mount_directory: RefCell::new(None),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AddPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let obj_ = imp::AddPage::from_instance(&obj);

            obj.setup_combo_box();

            obj_.add_new_vault_action_row
                .connect_activated(clone!(@weak obj => move |_| {
                    obj.add_new_vault_action_row_clicked();
                }));

            obj_.import_new_vault_action_row
                .connect_activated(clone!(@weak obj => move |_| {
                    obj.import_new_vault_action_row_clicked();
                }));

            obj_.previous_button_p_2
                .connect_clicked(clone!(@weak obj => move |_| {
                    obj.previous_button_p_2_clicked();
                }));

            obj_.next_button_p_2
                .connect_clicked(clone!(@weak obj => move |_| {
                    obj.next_button_p_2_clicked();
                }));

            obj_.vault_name_entry
                .connect_property_text_notify(clone!(@weak obj => move |_| {
                    obj.check_next_button_p_2_enable();
                }));

            obj_.backend_type_combo_box_text
                .connect_changed(clone!(@weak obj => move |_| {
                    obj.set_info_label_text();
                    obj.check_next_button_p_2_enable();
                }));

            obj_.backend_type_info_button
                .connect_clicked(clone!(@weak obj => move |_| {
                    obj.backend_type_info_button_clicked();
                }));

            obj_.password_entry
                .connect_property_text_notify(clone!(@weak obj => move |_| {
                    obj.check_next_button_p_2_enable();
                }));

            obj_.confirm_password_entry.connect_property_text_notify(
                clone!(@weak obj => move |_| {
                    obj.check_next_button_p_2_enable();
                }),
            );

            obj_.previous_button_p_3
                .connect_clicked(clone!(@weak obj => move |_| {
                    obj.previous_button_p_3_clicked();
                }));

            obj_.encrypted_data_directory_entry
                .connect_property_text_notify(clone!(@weak obj => move |_| {
                    obj.check_add_import_button_enable();
                }));

            obj_.mount_directory_entry
                .connect_property_text_notify(clone!(@weak obj => move |_| {
                    obj.check_add_import_button_enable();
                }));

            obj_.add_import_button
                .connect_clicked(clone!(@weak obj => move |_| {
                    obj.add_import_button_clicked();
                }));

            obj_.encrypted_data_directory_button
                .connect_clicked(clone!(@weak obj => move |_| {
                    obj.encrypted_data_directory_button_clicked();
                }));

            obj_.mount_directory_button
                .connect_clicked(clone!(@weak obj => move |_| {
                    obj.mount_directory_button_clicked();
                }));
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("add", &[], glib::Type::UNIT.into()).build(),
                    Signal::builder("import", &[], glib::Type::UNIT.into()).build(),
                ]
            });
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for AddPage {}

    impl BinImpl for AddPage {}
}

glib::wrapper! {
    pub struct AddPage(ObjectSubclass<imp::AddPage>)
        @extends gtk::Widget, adw::Bin;
}

impl AddPage {
    pub fn connect_add<F: Fn() + 'static>(&self, callback: F) -> glib::SignalHandlerId {
        self.connect_local("add", false, move |_| {
            callback();
            None
        })
        .unwrap()
    }

    pub fn connect_import<F: Fn() + 'static>(&self, callback: F) -> glib::SignalHandlerId {
        self.connect_local("import", false, move |_| {
            callback();
            None
        })
        .unwrap()
    }

    pub fn new() -> Self {
        let page: Self = glib::Object::new(&[]).expect("Failed to create AddPage");

        page
    }

    pub fn init(&self) {
        let self_ = imp::AddPage::from_instance(self);

        self_.vault_name_entry.set_text("");
        self_.password_entry.set_text("");
        self_.confirm_password_entry.set_text("");
        self_.encrypted_data_directory_entry.set_text("");
        self_.mount_directory_entry.set_text("");
        self_.info_label.set_text("");

        self_.next_button_p_2.set_sensitive(false);
        self_.previous_button_p_3.set_sensitive(true);
        self_.encrypted_data_directory_entry.set_sensitive(true);
        self_.encrypted_data_directory_button.set_sensitive(true);
        self_.mount_directory_entry.set_sensitive(true);
        self_.mount_directory_button.set_sensitive(true);
        self_.add_import_button.set_sensitive(false);

        self_
            .carousel
            .scroll_to(&self_.carousel.get_nth_page(0).unwrap());
    }

    fn setup_combo_box(&self) {
        let self_ = imp::AddPage::from_instance(self);

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
            }

            self.set_info_label_text();
        }
    }

    fn add_new_vault_action_row_clicked(&self) {
        let self_ = imp::AddPage::from_instance(&self);

        self_.is_add_new_vault.borrow_mut().replace(true);
        self_.password_label.set_visible(true);
        self_.password_entry.set_visible(true);
        self_.confirm_password_label.set_visible(true);
        self_.confirm_password_entry.set_visible(true);
        self_.add_import_button.set_label(&gettext("Add"));
        self_.vault_name_entry.grab_focus_without_selecting();
        self_
            .carousel
            .scroll_to(&self_.carousel.get_nth_page(1).unwrap());
    }

    fn import_new_vault_action_row_clicked(&self) {
        let self_ = imp::AddPage::from_instance(&self);

        self_.is_add_new_vault.borrow_mut().replace(false);
        self_.password_label.set_visible(false);
        self_.password_entry.set_visible(false);
        self_.confirm_password_label.set_visible(false);
        self_.confirm_password_entry.set_visible(false);
        self_.add_import_button.set_label(&gettext("Import"));
        self_.vault_name_entry.grab_focus_without_selecting();
        self_
            .carousel
            .scroll_to(&self_.carousel.get_nth_page(1).unwrap());
    }

    fn previous_button_p_2_clicked(&self) {
        let self_ = imp::AddPage::from_instance(&self);
        self_
            .carousel
            .scroll_to(&self_.carousel.get_nth_page(0).unwrap());
    }

    fn backend_type_info_button_clicked(&self) {
        let self_ = imp::AddPage::from_instance(&self);

        self.set_info_label_text();

        if self_.info_revealer.get_reveal_child() {
            self_.info_revealer.set_reveal_child(false);
        } else {
            self_.info_revealer.set_reveal_child(true);
        }
    }

    fn next_button_p_2_clicked(&self) {
        let self_ = imp::AddPage::from_instance(&self);

        self_
            .name
            .borrow_mut()
            .replace(self_.vault_name_entry.get_text().to_string());

        if let Some(is_add_new_vault) = *self_.is_add_new_vault.borrow() {
            if is_add_new_vault {
                if let Some(user_data_directory) = UserConfigManager::instance().get_user_data_dir()
                {
                    let mut path = String::from(user_data_directory);
                    path.push_str(&self_.name.borrow().as_ref().unwrap());
                    self_.encrypted_data_directory_entry.set_text(&path);
                }

                if let Some(vaults_home) = UserConfigManager::instance().get_vaults_home() {
                    let mut path = String::from(vaults_home);
                    path.push_str(&self_.name.borrow().as_ref().unwrap());
                    self_.mount_directory_entry.set_text(&path);
                }
            }
        }

        self_
            .carousel
            .scroll_to(&self_.carousel.get_nth_page(2).unwrap());
    }

    fn previous_button_p_3_clicked(&self) {
        let self_ = imp::AddPage::from_instance(&self);
        self_
            .carousel
            .scroll_to(&self_.carousel.get_nth_page(1).unwrap());
    }

    fn add_import_button_clicked(&self) {
        let self_ = imp::AddPage::from_instance(&self);

        self.set_last_page_sensitive(false);

        let vault = Vault::new(
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
        let vault_config = vault.get_config().clone().unwrap();

        if let Err(e) =
            std::fs::create_dir_all(std::path::Path::new(&vault_config.encrypted_data_directory))
        {
            log::error!("Could not create directories: {}", e);
        };

        UserConfigManager::instance().set_current_vault(vault);

        if let Some(is_add_new_vault) = *self_.is_add_new_vault.borrow() {
            if is_add_new_vault {
                if let Err(e) =
                    std::fs::create_dir_all(std::path::Path::new(&vault_config.mount_directory))
                {
                    log::error!("Could not create directories: {}", e);
                };

                let password = String::from(self_.password_entry.get_text().as_str());
                PasswordManager::instance().set_current_password(password);

                self.emit_by_name("add", &[]).unwrap();
            } else {
                self.emit_by_name("import", &[]).unwrap();
            }
        }
    }

    fn is_valid_vault_name(&self, vault_name: GString) -> bool {
        let self_ = imp::AddPage::from_instance(self);

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
        let self_ = imp::AddPage::from_instance(self);

        let is_duplicate_name = UserConfigManager::instance()
            .get_map()
            .contains_key(&vault_name.to_string());
        if !vault_name.is_empty() && is_duplicate_name {
            self_
                .info_label
                .set_text(&gettext("Vault name already exists."));
            false
        } else {
            true
        }
    }

    fn are_passwords_empty(&self, password: &GString, confirm_password: &GString) -> bool {
        let self_ = imp::AddPage::from_instance(self);

        if password.is_empty() && confirm_password.is_empty() {
            self_.info_label.set_text(&gettext("Password is empty."));
            true
        } else {
            false
        }
    }

    fn are_passwords_equal(&self, password: &GString, confirm_password: &GString) -> bool {
        let self_ = imp::AddPage::from_instance(self);

        if password.eq(confirm_password) {
            true
        } else {
            self_
                .info_label
                .set_text(&gettext("Passwords are not equal."));
            false
        }
    }

    fn check_next_button_p_2_enable(&self) {
        let self_ = imp::AddPage::from_instance(&self);

        self_.info_label.set_text("");
        self_.next_button_p_2.set_sensitive(false);

        let vault_name = self_.vault_name_entry.get_text();
        let password = self_.password_entry.get_text();
        let confirm_password = self_.confirm_password_entry.get_text();

        let is_valid_vault_name = self.is_valid_vault_name(vault_name.clone());
        if !is_valid_vault_name {
            return;
        }

        let is_different_vault_name = self.is_different_vault_name(vault_name);
        if !is_different_vault_name {
            return;
        }

        if let Some(is_add_new_vault) = *self_.is_add_new_vault.borrow() {
            if is_add_new_vault {
                let are_passwords_empty = self.are_passwords_empty(&password, &confirm_password);
                if are_passwords_empty {
                    return;
                }

                let are_passwords_equal = if !are_passwords_empty {
                    self.are_passwords_equal(&password, &confirm_password)
                } else {
                    false
                };
                if !are_passwords_equal {
                    return;
                }
            }
        }

        self_.next_button_p_2.set_sensitive(true);
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
        let self_ = imp::AddPage::from_instance(self);

        if encrypted_data_directory.is_empty() {
            return false;
        }

        if let Some(is_add_new_vault) = *self_.is_add_new_vault.borrow() {
            if is_add_new_vault {
                if !std::path::Path::exists(std::path::Path::new(encrypted_data_directory)) {
                    return true;
                }

                match self.is_path_empty(encrypted_data_directory) {
                    Ok(is_empty) => {
                        if is_empty {
                            return true;
                        } else {
                            self_
                                .info_label
                                .set_text(&gettext("Encrypted data directory is not empty."));
                            return false;
                        }
                    }
                    Err(_) => {
                        self_
                            .info_label
                            .set_text(&gettext("Encrypted data directory is not valid."));
                        return false;
                    }
                }
            } else {
                match self.is_path_empty(encrypted_data_directory) {
                    Ok(is_empty) => {
                        if is_empty {
                            self_
                                .info_label
                                .set_text(&gettext("Encrypted data directory is empty."));
                            return false;
                        } else {
                            return true;
                        }
                    }
                    Err(_) => {
                        self_
                            .info_label
                            .set_text(&gettext("Encrypted data directory is not valid."));
                        return false;
                    }
                }
            }
        }

        false
    }

    fn is_mount_directory_valid(&self, mount_directory: &GString) -> bool {
        let self_ = imp::AddPage::from_instance(self);

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
        let self_ = imp::AddPage::from_instance(self);

        if encrypted_data_directory.eq(mount_directory) {
            self_
                .info_label
                .set_text(&gettext("Directories must be different."));
            false
        } else {
            true
        }
    }

    fn check_add_import_button_enable(&self) {
        let self_ = imp::AddPage::from_instance(&self);

        self_.info_label.set_text("");
        self_.add_import_button.set_sensitive(false);

        let encrypted_data_directory = self_.encrypted_data_directory_entry.get_text();
        let mount_directory = self_.mount_directory_entry.get_text();

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

        self_.add_import_button.set_sensitive(true);
    }

    fn set_info_label_text(&self) {
        let self_ = imp::AddPage::from_instance(&self);

        let backend = self_.backend_type_combo_box_text.get_active_text().unwrap();

        if backend.eq("Gocryptfs") {
            self_.info_revealer_label.set_text(&gettext("Fast and robust, gocryptfs works well in general cases where third-parties do not always have access to the encrypted data directory (e.g. file hosting services). It exposes directory structure, number of files and file sizes. Security audit in 2017 verified gocryptfs being safe against third-parties that can read or write to encrypted data."));
        } else {
            self_.info_revealer_label.set_text(&gettext("CryFS works well together with cloud services like Dropbox, iCloud, OneDrive and others. It does not expose directory structure, number of files or file sizes in the encrypted data directory. While being considered safe, there is no independent audit of CryFS."));
        }
    }

    fn encrypted_data_directory_button_clicked(&self) {
        let window = gio::Application::get_default()
            .unwrap()
            .downcast_ref::<VApplication>()
            .unwrap()
            .get_active_window()
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
                let file = dialog.get_file().unwrap();
                let path = String::from(file.get_path().unwrap().as_os_str().to_str().unwrap());
                let self_ = imp::AddPage::from_instance(&obj);
                self_.encrypted_data_directory_entry.set_text(&path);
            }

            dialog.destroy();
        }));

        dialog.show();
    }

    fn mount_directory_button_clicked(&self) {
        let window = gio::Application::get_default()
            .unwrap()
            .downcast_ref::<VApplication>()
            .unwrap()
            .get_active_window()
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
                let file = dialog.get_file().unwrap();
                let path = String::from(file.get_path().unwrap().as_os_str().to_str().unwrap());
                let self_ = imp::AddPage::from_instance(&obj);
                self_.mount_directory_entry.set_text(&path);
            }

            dialog.destroy();
        }));

        dialog.show();
    }

    pub fn set_last_page_sensitive(&self, sensitive: bool) {
        let self_ = imp::AddPage::from_instance(&self);

        self_.previous_button_p_3.set_sensitive(sensitive);
        self_
            .encrypted_data_directory_entry
            .set_sensitive(sensitive);
        self_
            .encrypted_data_directory_button
            .set_sensitive(sensitive);
        self_.mount_directory_entry.set_sensitive(sensitive);
        self_.mount_directory_button.set_sensitive(sensitive);
        self_.add_import_button.set_sensitive(sensitive);
    }
}
