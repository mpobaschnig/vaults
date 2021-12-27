// import_vault_dialog.rs
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
use gettextrs::gettext;
use gtk::{
    self, gio, gio::File, glib, glib::clone, glib::GString, prelude::*, subclass::prelude::*,
    CompositeTemplate,
};
use std::cell::RefCell;
use strum::IntoEnumIterator;

use crate::{backend, user_config_manager::UserConfigManager, vault::*, VApplication};

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/io/github/mpobaschnig/Vaults/import_vault_dialog.ui")]
    pub struct ImportVaultDialog {
        #[template_child]
        pub carousel: TemplateChild<adw::Carousel>,
        #[template_child]
        pub cancel_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub next_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub previous_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub import_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub name_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub backend_type_combo_box_text: TemplateChild<gtk::ComboBoxText>,
        #[template_child]
        pub backend_type_error_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub name_error_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub encrypted_data_directory_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub encrypted_data_directory_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub mount_directory_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub mount_directory_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub encrypted_data_directory_error_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub encrypted_data_directory_info_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub mount_directory_error_label: TemplateChild<gtk::Label>,

        pub current_page: RefCell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ImportVaultDialog {
        const NAME: &'static str = "ImportVaultDialog";
        type ParentType = gtk::Dialog;
        type Type = super::ImportVaultDialog;

        fn new() -> Self {
            Self {
                carousel: TemplateChild::default(),
                cancel_button: TemplateChild::default(),
                previous_button: TemplateChild::default(),
                next_button: TemplateChild::default(),
                import_button: TemplateChild::default(),
                name_entry: TemplateChild::default(),
                backend_type_combo_box_text: TemplateChild::default(),
                backend_type_error_label: TemplateChild::default(),
                name_error_label: TemplateChild::default(),
                encrypted_data_directory_entry: TemplateChild::default(),
                encrypted_data_directory_button: TemplateChild::default(),
                mount_directory_entry: TemplateChild::default(),
                mount_directory_button: TemplateChild::default(),
                encrypted_data_directory_error_label: TemplateChild::default(),
                encrypted_data_directory_info_label: TemplateChild::default(),
                mount_directory_error_label: TemplateChild::default(),

                current_page: RefCell::new(0),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ImportVaultDialog {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.setup_actions();
            obj.setup_signals();

            obj.fill_combo_box_text();
        }
    }

    impl WidgetImpl for ImportVaultDialog {}
    impl WindowImpl for ImportVaultDialog {}
    impl DialogImpl for ImportVaultDialog {}
}

glib::wrapper! {
    pub struct ImportVaultDialog(ObjectSubclass<imp::ImportVaultDialog>)
        @extends gtk::Widget, gtk::Window, gtk::Dialog;
}

impl ImportVaultDialog {
    pub fn new() -> Self {
        let dialog: Self = glib::Object::new(&[("use-header-bar", &1)])
            .expect("Failed to create ImportVaultDialog");

        let window = gio::Application::default()
            .unwrap()
            .downcast_ref::<VApplication>()
            .unwrap()
            .active_window()
            .unwrap();
        dialog.set_transient_for(Some(&window));

        dialog
    }

    fn setup_actions(&self) {}

    fn setup_signals(&self) {
        let self_ = imp::ImportVaultDialog::from_instance(self);

        self_
            .cancel_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.response(gtk::ResponseType::Cancel);
            }));

        self_
            .previous_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.previous_button_clicked();
            }));

        self_
            .next_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.next_button_clicked();
            }));

        self_
            .import_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.response(gtk::ResponseType::Ok);
            }));

        self_
            .name_entry
            .connect_text_notify(clone!(@weak self as obj => move |_| {
                obj.validate_name();
            }));

        self_
            .backend_type_combo_box_text
            .connect_changed(clone!(@weak self as obj => move |_| {
                obj.validate_name();
            }));

        self_.encrypted_data_directory_entry.connect_text_notify(
            clone!(@weak self as obj => move |_| {
                obj.validate_directories();
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
                obj.validate_directories();
            }));

        self_
            .mount_directory_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.mount_directory_button_clicked();
            }));
    }

    pub fn next_button_clicked(&self) {
        let self_ = imp::ImportVaultDialog::from_instance(self);

        *self_.current_page.borrow_mut() += 1;

        self_.carousel.scroll_to(
            &self_
                .carousel
                .nth_page(*self_.current_page.borrow())
                .unwrap(),
        );

        self.update_headerbar_buttons();
    }

    pub fn previous_button_clicked(&self) {
        let self_ = &mut imp::ImportVaultDialog::from_instance(self);

        *self_.current_page.borrow_mut() -= 1;

        self_.carousel.scroll_to(
            &self_
                .carousel
                .nth_page(*self_.current_page.borrow())
                .unwrap(),
        );

        self.update_headerbar_buttons();
    }

    fn update_headerbar_buttons(&self) {
        let self_ = &mut imp::ImportVaultDialog::from_instance(self);

        match *self_.current_page.borrow() {
            0 => {
                self_.cancel_button.set_visible(true);
                self_.previous_button.set_visible(false);
                self_.next_button.set_visible(true);
                self_.import_button.set_visible(false);

                self.validate_directories();
            }
            1 => {
                self_.cancel_button.set_visible(false);
                self_.previous_button.set_visible(true);
                self_.next_button.set_visible(false);
                self_.import_button.set_visible(true);

                let combo_box_text = self_.backend_type_combo_box_text.active_text();

                if combo_box_text.is_none() {
                    self_.import_button.set_sensitive(false);

                    self_.backend_type_error_label.set_text(&gettext(
                        "No backend installed. Please install gocryptfs or CryFS.",
                    ));
                    self_.backend_type_error_label.set_visible(true);
                } else {
                    self_.backend_type_error_label.set_visible(false);
                }

                self.validate_name();
            }
            _ => {}
        }
    }

    pub fn validate_name(&self) {
        let self_ = imp::ImportVaultDialog::from_instance(self);

        self_.import_button.set_sensitive(false);

        let vault_name = self_.name_entry.text();

        if vault_name.is_empty() {
            self_.import_button.set_sensitive(false);

            self_.name_entry.remove_css_class("error");
            self_.name_error_label.set_visible(false);
            self_.name_error_label.set_text("");

            return;
        }

        let is_duplicate = UserConfigManager::instance()
            .get_map()
            .contains_key(&vault_name.to_string());

        if is_duplicate {
            self_.import_button.set_sensitive(false);

            self_.name_entry.add_css_class("error");
            self_.name_error_label.set_visible(true);
            self_
                .name_error_label
                .set_text(&gettext("Name is already taken."));

            return;
        } else {
            self_.name_entry.remove_css_class("error");
            self_.name_error_label.set_visible(false);
            self_.name_error_label.set_text("");
        }

        self_.import_button.set_sensitive(true);
    }

    pub fn encrypted_data_directory_button_clicked(&self) {
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
                let file = dialog.file().unwrap();
                let path = String::from(file.path().unwrap().as_os_str().to_str().unwrap());
                let self_ = imp::ImportVaultDialog::from_instance(&obj);
                self_.encrypted_data_directory_entry.set_text(&path);

                obj.validate_directories();
            }

            dialog.destroy();
        }));

        dialog.show();
    }

    pub fn mount_directory_button_clicked(&self) {
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
                let file = dialog.file().unwrap();
                let path = String::from(file.path().unwrap().as_os_str().to_str().unwrap());
                let self_ = imp::ImportVaultDialog::from_instance(&obj);
                self_.mount_directory_entry.set_text(&path);

                obj.guess_name(&file);
                obj.validate_directories();
            }

            dialog.destroy();
        }));

        dialog.show();
    }

    pub fn validate_directories(&self) {
        let self_ = imp::ImportVaultDialog::from_instance(self);

        self_.import_button.set_sensitive(false);

        self_.encrypted_data_directory_info_label.set_visible(false);

        let encrypted_data_directory = self_.encrypted_data_directory_entry.text();
        let mount_directory = self_.mount_directory_entry.text();

        let is_edd_valid = self.is_encrypted_data_directory_valid(&encrypted_data_directory);

        let is_md_valid = self.is_mount_directory_valid(&mount_directory);

        if !is_edd_valid || !is_md_valid {
            return;
        }

        if encrypted_data_directory.eq(&mount_directory) {
            self_.encrypted_data_directory_entry.add_css_class("error");
            self_.mount_directory_entry.add_css_class("error");

            self_
                .mount_directory_error_label
                .set_text(&gettext("Directories must not be equal."));
            self_.mount_directory_error_label.set_visible(true);

            return;
        }

        if !self.is_valid_backend(&encrypted_data_directory.to_string()) {
            return;
        }

        self_.next_button.set_sensitive(true);
    }

    fn is_encrypted_data_directory_valid(&self, encrypted_data_directory: &GString) -> bool {
        let self_ = imp::ImportVaultDialog::from_instance(self);

        if encrypted_data_directory.is_empty() {
            self_
                .encrypted_data_directory_error_label
                .set_visible(false);

            self_
                .encrypted_data_directory_entry
                .remove_css_class("error");

            return false;
        }

        match self.is_path_empty(&encrypted_data_directory) {
            Ok(is_empty) => {
                if is_empty {
                    self_
                        .encrypted_data_directory_error_label
                        .set_text(&gettext("Encrypted data directory is empty."));
                    self_.encrypted_data_directory_error_label.set_visible(true);

                    self_.encrypted_data_directory_entry.add_css_class("error");

                    false
                } else {
                    self_
                        .encrypted_data_directory_error_label
                        .set_visible(false);

                    self_
                        .encrypted_data_directory_entry
                        .remove_css_class("error");

                    self.is_valid_backend(&encrypted_data_directory.to_string());

                    true
                }
            }
            Err(_) => {
                self_
                    .encrypted_data_directory_error_label
                    .set_text(&gettext("Encrypted data directory is not valid."));
                self_.encrypted_data_directory_error_label.set_visible(true);

                self_.encrypted_data_directory_entry.add_css_class("error");

                false
            }
        }
    }

    fn is_mount_directory_valid(&self, mount_directory: &GString) -> bool {
        let self_ = imp::ImportVaultDialog::from_instance(self);

        if mount_directory.is_empty() {
            self_.mount_directory_error_label.set_visible(false);

            self_.mount_directory_entry.remove_css_class("error");

            return false;
        }

        match self.is_path_empty(&mount_directory) {
            Ok(is_empty) => {
                if is_empty {
                    self_.mount_directory_error_label.set_visible(false);

                    self_.mount_directory_entry.remove_css_class("error");

                    true
                } else {
                    self_
                        .mount_directory_error_label
                        .set_text(&gettext("Mount directory is not empty."));
                    self_.mount_directory_error_label.set_visible(true);

                    self_.mount_directory_entry.add_css_class("error");

                    false
                }
            }
            Err(_) => {
                self_
                    .mount_directory_error_label
                    .set_text(&gettext("Mount directory is not valid."));
                self_.mount_directory_error_label.set_visible(true);

                self_.mount_directory_entry.add_css_class("error");

                false
            }
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

    pub fn get_vault(&self) -> Vault {
        let self_ = imp::ImportVaultDialog::from_instance(self);

        Vault::new(
            String::from(self_.name_entry.text().as_str()),
            backend::get_backend_from_ui_string(
                &self_
                    .backend_type_combo_box_text
                    .active_text()
                    .unwrap()
                    .to_string(),
            )
            .unwrap(),
            String::from(self_.encrypted_data_directory_entry.text().as_str()),
            String::from(self_.mount_directory_entry.text().as_str()),
        )
    }

    fn fill_combo_box_text(&self) {
        let self_ = imp::ImportVaultDialog::from_instance(self);

        for backend in backend::Backend::iter() {
            let value = &backend::get_ui_string_from_backend(&backend);

            self_
                .backend_type_combo_box_text
                .append(Some(&value), &value);
        }
    }

    fn guess_name(&self, file: &File) {
        let self_ = imp::ImportVaultDialog::from_instance(self);

        self_
            .name_entry
            .set_text(file.basename().unwrap().as_os_str().to_str().unwrap());
    }

    fn is_valid_backend(&self, path: &String) -> bool {
        let self_ = imp::ImportVaultDialog::from_instance(self);

        match std::fs::read_dir(path.to_string()) {
            Ok(dir) => {
                for file in dir.into_iter() {
                    match file {
                        Ok(f) => {
                            let file_name = f.file_name();

                            if file_name == "gocryptfs.conf" {
                                self_
                                    .encrypted_data_directory_info_label
                                    .set_text(&gettext("Found gocryptfs configuration file."));

                                self_.encrypted_data_directory_info_label.set_visible(true);

                                self_
                                    .backend_type_combo_box_text
                                    .set_active_id(Some("gocryptfs"));

                                return true;
                            }

                            if file_name == "cryfs.config" {
                                self_
                                    .encrypted_data_directory_info_label
                                    .set_text(&gettext("Found CryFS configuration file."));

                                self_.encrypted_data_directory_info_label.set_visible(true);

                                self_
                                    .backend_type_combo_box_text
                                    .set_active_id(Some("CryFS"));

                                return true;
                            }
                        }
                        Err(e) => {
                            log::debug!("Invalid file: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                log::debug!("Could not read path {}: {}", path, e);
            }
        }

        self_
            .encrypted_data_directory_error_label
            .set_text(&gettext("No configuration file found."));

        self_.encrypted_data_directory_error_label.set_visible(true);

        false
    }
}
