// import_vault_window.rs
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

use adw::prelude::ComboRowExt;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::{self, gio, gio::File, glib, glib::clone, glib::GString, prelude::*, CompositeTemplate};
use std::cell::RefCell;
use strum::IntoEnumIterator;

use crate::{backend, user_config_manager::UserConfigManager, vault::*, VApplication};

mod imp {
    use gtk::glib::subclass::Signal;
    use once_cell::sync::Lazy;

    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/io/github/mpobaschnig/Vaults/import_vault_window.ui")]
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
        pub name_entry_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub combo_row_backend: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub backend_type_error_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub name_error_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub encrypted_data_directory_entry_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub encrypted_data_directory_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub mount_directory_entry_row: TemplateChild<adw::EntryRow>,
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
        type ParentType = adw::Window;
        type Type = super::ImportVaultDialog;

        fn new() -> Self {
            Self {
                carousel: TemplateChild::default(),
                cancel_button: TemplateChild::default(),
                previous_button: TemplateChild::default(),
                next_button: TemplateChild::default(),
                import_button: TemplateChild::default(),
                name_entry_row: TemplateChild::default(),
                combo_row_backend: TemplateChild::default(),
                backend_type_error_label: TemplateChild::default(),
                name_error_label: TemplateChild::default(),
                encrypted_data_directory_entry_row: TemplateChild::default(),
                encrypted_data_directory_button: TemplateChild::default(),
                mount_directory_entry_row: TemplateChild::default(),
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
        fn constructed(&self) {
            let obj = self.obj();
            self.parent_constructed();

            obj.fill_combo_box_text();
            obj.setup_actions();
            obj.setup_signals();
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("import").build(),
                    Signal::builder("close").build(),
                ]
            });
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for ImportVaultDialog {}
    impl AdwWindowImpl for ImportVaultDialog {}
    impl WindowImpl for ImportVaultDialog {}
    impl DialogImpl for ImportVaultDialog {}
}

glib::wrapper! {
    pub struct ImportVaultDialog(ObjectSubclass<imp::ImportVaultDialog>)
        @extends gtk::Widget, adw::Window, gtk::Window;
}

impl ImportVaultDialog {
    pub fn new() -> Self {
        let dialog: Self = glib::Object::builder().build();

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
        self.imp()
            .cancel_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.emit_by_name::<()>("close", &[]);
                obj.close();
            }));

        self.imp()
            .previous_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.previous_button_clicked();
            }));

        self.imp()
            .next_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.next_button_clicked();
            }));

        self.imp()
            .import_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.emit_by_name::<()>("import", &[]);
                obj.close();
            }));

        self.imp()
            .name_entry_row
            .connect_text_notify(clone!(@weak self as obj => move |_| {
                obj.validate_name();
            }));

        self.imp().combo_row_backend.connect_selected_notify(
            clone!(@weak self as obj => move |_| {
                obj.validate_name();
            }),
        );

        self.imp()
            .encrypted_data_directory_entry_row
            .connect_text_notify(clone!(@weak self as obj => move |_| {
                obj.validate_directories();
            }));

        self.imp().encrypted_data_directory_button.connect_clicked(
            clone!(@weak self as obj => move |_| {
                obj.encrypted_data_directory_button_clicked();
            }),
        );

        self.imp().mount_directory_entry_row.connect_text_notify(
            clone!(@weak self as obj => move |_| {
                obj.validate_directories();
            }),
        );

        self.imp()
            .mount_directory_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.mount_directory_button_clicked();
            }));
    }

    pub fn next_button_clicked(&self) {
        *self.imp().current_page.borrow_mut() += 1;

        self.imp().carousel.scroll_to(
            &self
                .imp()
                .carousel
                .nth_page(*self.imp().current_page.borrow()),
            false,
        );

        self.update_headerbar_buttons();
    }

    pub fn previous_button_clicked(&self) {
        *self.imp().current_page.borrow_mut() -= 1;

        self.imp().carousel.scroll_to(
            &self
                .imp()
                .carousel
                .nth_page(*self.imp().current_page.borrow()),
            false,
        );

        self.update_headerbar_buttons();
    }

    fn update_headerbar_buttons(&self) {
        match *self.imp().current_page.borrow() {
            0 => {
                self.imp().cancel_button.set_visible(true);
                self.imp().previous_button.set_visible(false);
                self.imp().next_button.set_visible(true);
                self.imp().import_button.set_visible(false);

                self.validate_directories();
            }
            1 => {
                self.imp().cancel_button.set_visible(false);
                self.imp().previous_button.set_visible(true);
                self.imp().next_button.set_visible(false);
                self.imp().import_button.set_visible(true);

                let drop_down_text = self.imp().combo_row_backend.selected_item();

                if drop_down_text.is_none() {
                    self.imp().import_button.set_sensitive(false);

                    self.imp().backend_type_error_label.set_text(&gettext(
                        "No backend installed. Please install gocryptfs or CryFS.",
                    ));
                    self.imp().backend_type_error_label.set_visible(true);
                } else {
                    self.imp().backend_type_error_label.set_visible(false);
                }

                self.validate_name();
            }
            _ => {}
        }
    }

    pub fn validate_name(&self) {
        self.imp().import_button.set_sensitive(false);

        let vault_name = self.imp().name_entry_row.text();

        if vault_name.is_empty() {
            self.imp().import_button.set_sensitive(false);

            self.imp().name_entry_row.remove_css_class("error");
            self.imp().name_error_label.set_visible(false);
            self.imp().name_error_label.set_text("");

            return;
        }

        let is_duplicate = UserConfigManager::instance()
            .get_map()
            .contains_key(&vault_name.to_string());

        if is_duplicate {
            self.imp().import_button.set_sensitive(false);

            self.imp().name_entry_row.add_css_class("error");
            self.imp().name_error_label.set_visible(true);
            self.imp()
                .name_error_label
                .set_text(&gettext("Name is already taken."));

            return;
        } else {
            self.imp().name_entry_row.remove_css_class("error");
            self.imp().name_error_label.set_visible(false);
            self.imp().name_error_label.set_text("");
        }

        self.imp().import_button.set_sensitive(true);
    }

    pub fn encrypted_data_directory_button_clicked(&self) {
        let dialog = gtk::FileDialog::builder()
            .title(&gettext("Choose Encrypted Data Directory"))
            .modal(true)
            .accept_label(&gettext("Select"))
            .build();

        dialog.select_folder(Some(self), gio::Cancellable::NONE, clone!(@weak self as obj => move |directory| {
            if let Ok(directory) = directory {
                let path = String::from(directory.path().unwrap().as_os_str().to_str().unwrap());
                obj.imp().encrypted_data_directory_entry_row.set_text(&path);

                obj.validate_directories();
            }
        }));
    }

    pub fn mount_directory_button_clicked(&self) {
        let dialog = gtk::FileDialog::builder()
            .title(&gettext("Choose Mount Directory"))
            .modal(true)
            .accept_label(&gettext("Select"))
            .build();

        dialog.select_folder(Some(self), gio::Cancellable::NONE, clone!(@weak self as obj => move |directory| {
            if let Ok(directory) = directory {
                let path = String::from(directory.path().unwrap().as_os_str().to_str().unwrap());
                obj.imp().mount_directory_entry_row.set_text(&path);

                obj.guess_name(&directory);
                obj.validate_directories();
            }
        }));
    }

    pub fn validate_directories(&self) {
        self.imp().import_button.set_sensitive(false);

        self.imp()
            .encrypted_data_directory_info_label
            .set_visible(false);

        let encrypted_data_directory = self.imp().encrypted_data_directory_entry_row.text();
        let mount_directory = self.imp().mount_directory_entry_row.text();

        let is_edd_valid = self.is_encrypted_data_directory_valid(&encrypted_data_directory);

        let is_md_valid = self.is_mount_directory_valid(&mount_directory);

        if !is_edd_valid || !is_md_valid {
            return;
        }

        if encrypted_data_directory.eq(&mount_directory) {
            self.imp()
                .encrypted_data_directory_entry_row
                .add_css_class("error");
            self.imp().mount_directory_entry_row.add_css_class("error");

            self.imp()
                .mount_directory_error_label
                .set_text(&gettext("Directories must not be equal."));
            self.imp().mount_directory_error_label.set_visible(true);

            return;
        }

        if !self.is_valid_backend(&encrypted_data_directory.to_string()) {
            return;
        }

        self.imp().next_button.set_sensitive(true);
    }

    fn is_encrypted_data_directory_valid(&self, encrypted_data_directory: &GString) -> bool {
        if encrypted_data_directory.is_empty() {
            self.imp()
                .encrypted_data_directory_error_label
                .set_visible(false);

            self.imp()
                .encrypted_data_directory_entry_row
                .remove_css_class("error");

            return false;
        }

        match self.is_path_empty(&encrypted_data_directory) {
            Ok(is_empty) => {
                if is_empty {
                    self.imp()
                        .encrypted_data_directory_error_label
                        .set_text(&gettext("Encrypted data directory is empty."));
                    self.imp()
                        .encrypted_data_directory_error_label
                        .set_visible(true);

                    self.imp()
                        .encrypted_data_directory_entry_row
                        .add_css_class("error");

                    false
                } else {
                    self.imp()
                        .encrypted_data_directory_error_label
                        .set_visible(false);

                    self.imp()
                        .encrypted_data_directory_entry_row
                        .remove_css_class("error");

                    self.is_valid_backend(&encrypted_data_directory.to_string());

                    true
                }
            }
            Err(_) => {
                self.imp()
                    .encrypted_data_directory_error_label
                    .set_text(&gettext("Encrypted data directory is not valid."));
                self.imp()
                    .encrypted_data_directory_error_label
                    .set_visible(true);

                self.imp()
                    .encrypted_data_directory_entry_row
                    .add_css_class("error");

                false
            }
        }
    }

    fn is_mount_directory_valid(&self, mount_directory: &GString) -> bool {
        if mount_directory.is_empty() {
            self.imp().mount_directory_error_label.set_visible(false);

            self.imp()
                .mount_directory_entry_row
                .remove_css_class("error");

            return false;
        }

        match self.is_path_empty(&mount_directory) {
            Ok(is_empty) => {
                if is_empty {
                    self.imp().mount_directory_error_label.set_visible(false);

                    self.imp()
                        .mount_directory_entry_row
                        .remove_css_class("error");

                    true
                } else {
                    self.imp()
                        .mount_directory_error_label
                        .set_text(&gettext("Mount directory is not empty."));
                    self.imp().mount_directory_error_label.set_visible(true);

                    self.imp().mount_directory_entry_row.add_css_class("error");

                    false
                }
            }
            Err(_) => {
                self.imp()
                    .mount_directory_error_label
                    .set_text(&gettext("Mount directory is not valid."));
                self.imp().mount_directory_error_label.set_visible(true);

                self.imp().mount_directory_entry_row.add_css_class("error");

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
        Vault::new(
            String::from(self.imp().name_entry_row.text().as_str()),
            backend::get_backend_from_ui_string(
                &self
                    .imp()
                    .combo_row_backend
                    .selected_item()
                    .unwrap()
                    .downcast::<gtk::StringObject>()
                    .unwrap()
                    .string()
                    .to_string(),
            )
            .unwrap(),
            String::from(
                self.imp()
                    .encrypted_data_directory_entry_row
                    .text()
                    .as_str(),
            ),
            String::from(self.imp().mount_directory_entry_row.text().as_str()),
            None,
        )
    }

    fn fill_combo_box_text(&self) {
        let list = gtk::StringList::new(&[]);

        for backend in backend::Backend::iter() {
            let value = &backend::get_ui_string_from_backend(&backend);
            list.append(value);
        }

        self.imp().combo_row_backend.set_model(Some(&list));
        self.imp().combo_row_backend.set_sensitive(false);
    }

    fn guess_name(&self, file: &File) {
        self.imp()
            .name_entry_row
            .set_text(file.basename().unwrap().as_os_str().to_str().unwrap());
    }

    fn is_valid_backend(&self, path: &String) -> bool {
        match std::fs::read_dir(path.to_string()) {
            Ok(dir) => {
                for file in dir.into_iter() {
                    match file {
                        Ok(f) => {
                            let file_name = f.file_name();

                            if file_name == "gocryptfs.conf" {
                                self.imp()
                                    .encrypted_data_directory_info_label
                                    .set_text(&gettext("Found gocryptfs configuration file."));

                                self.imp()
                                    .encrypted_data_directory_info_label
                                    .set_visible(true);

                                let model = self.imp().combo_row_backend.model().unwrap();
                                for (position, item) in model.iter::<glib::Object>().enumerate() {
                                    if let Ok(object) = item {
                                        let string_object =
                                            object.downcast::<gtk::StringObject>().unwrap();
                                        if string_object.string().eq("gocryptfs") {
                                            self.imp()
                                                .combo_row_backend
                                                .set_selected(position as u32);
                                        }
                                    }
                                }
                                return true;
                            }

                            if file_name == "cryfs.config" {
                                self.imp()
                                    .encrypted_data_directory_info_label
                                    .set_text(&gettext("Found CryFS configuration file."));

                                self.imp()
                                    .encrypted_data_directory_info_label
                                    .set_visible(true);

                                let model = self.imp().combo_row_backend.model().unwrap();
                                for (position, item) in model.iter::<glib::Object>().enumerate() {
                                    if let Ok(object) = item {
                                        let string_object =
                                            object.downcast::<gtk::StringObject>().unwrap();
                                        if string_object.string().eq("CryFS") {
                                            self.imp()
                                                .combo_row_backend
                                                .set_selected(position as u32);
                                        }
                                    }
                                }
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

        self.imp()
            .encrypted_data_directory_error_label
            .set_text(&gettext("No configuration file found."));

        self.imp()
            .encrypted_data_directory_error_label
            .set_visible(true);

        false
    }
}
