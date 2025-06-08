// add_new_vault_window.rs
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

use crate::application::VApplication;
use crate::config::APP_ID;
use crate::{backend, util, vault::*};
use adw::prelude::AdwDialogExt;
use adw::prelude::ComboRowExt;
use gettextrs::gettext;
use gtk::gio;

use adw::subclass::prelude::*;
use gtk::{self, prelude::*};
use gtk::{CompositeTemplate, glib};
use gtk::{gio::Settings, glib::subclass::Signal};
use gtk::{glib::GString, glib::clone};
use std::cell::RefCell;
use strum::IntoEnumIterator;

mod imp {
    use once_cell::sync::Lazy;

    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/io/github/mpobaschnig/Vaults/add_new_vault_window.ui")]
    pub struct AddNewVaultWindow {
        #[template_child]
        pub entry_row_name: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub combo_row_backend: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub carousel: TemplateChild<adw::Carousel>,
        #[template_child]
        pub cancel_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub next_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub previous_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub add_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub info_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub name_error_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub password_entry_row: TemplateChild<adw::PasswordEntryRow>,
        #[template_child]
        pub confirm_password_entry_row: TemplateChild<adw::PasswordEntryRow>,
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
        pub mount_directory_error_label: TemplateChild<gtk::Label>,

        pub current_page: RefCell<u32>,

        pub settings: Settings,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AddNewVaultWindow {
        const NAME: &'static str = "AddNewVaultWindow";
        type ParentType = adw::Dialog;
        type Type = super::AddNewVaultWindow;

        fn new() -> Self {
            Self {
                entry_row_name: TemplateChild::default(),
                combo_row_backend: TemplateChild::default(),
                carousel: TemplateChild::default(),
                cancel_button: TemplateChild::default(),
                previous_button: TemplateChild::default(),
                next_button: TemplateChild::default(),
                add_button: TemplateChild::default(),
                info_label: TemplateChild::default(),
                name_error_label: TemplateChild::default(),
                password_entry_row: TemplateChild::default(),
                confirm_password_entry_row: TemplateChild::default(),
                encrypted_data_directory_entry_row: TemplateChild::default(),
                encrypted_data_directory_button: TemplateChild::default(),
                mount_directory_entry_row: TemplateChild::default(),
                mount_directory_button: TemplateChild::default(),
                encrypted_data_directory_error_label: TemplateChild::default(),
                mount_directory_error_label: TemplateChild::default(),

                current_page: RefCell::new(0),

                settings: Settings::new(APP_ID),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AddNewVaultWindow {
        fn constructed(&self) {
            let obj = self.obj();
            self.parent_constructed();

            obj.setup_combo_box();
            obj.setup_actions();
            obj.setup_signals();
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("add").build(),
                    Signal::builder("close").build(),
                ]
            });
            SIGNALS.as_ref()
        }
    }

    impl AdwDialogImpl for AddNewVaultWindow {}
    impl DialogImpl for AddNewVaultWindow {}
    impl WidgetImpl for AddNewVaultWindow {}
    impl WindowImpl for AddNewVaultWindow {}
}

glib::wrapper! {
    pub struct AddNewVaultWindow(ObjectSubclass<imp::AddNewVaultWindow>)
        @extends gtk::Widget, adw::Dialog, adw::Window, gtk::Window;
}

impl Default for AddNewVaultWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl AddNewVaultWindow {
    pub fn new() -> Self {
        let dialog: Self = glib::Object::builder().build();

        dialog
    }

    fn setup_actions(&self) {}

    fn setup_signals(&self) {
        self.imp().add_button.connect_clicked(clone!(
            #[weak(rename_to = obj)]
            self,
            move |_| {
                obj.emit_by_name::<()>("add", &[]);
                AdwDialogExt::close(&obj);
            }
        ));

        self.imp().cancel_button.connect_clicked(clone!(
            #[weak(rename_to = obj)]
            self,
            move |_| {
                obj.emit_by_name::<()>("close", &[]);
                AdwDialogExt::close(&obj);
            }
        ));

        self.imp().previous_button.connect_clicked(clone!(
            #[weak(rename_to = obj)]
            self,
            move |_| {
                obj.previous_button_clicked();
            }
        ));

        self.imp().next_button.connect_clicked(clone!(
            #[weak(rename_to = obj)]
            self,
            move |_| {
                obj.next_button_clicked();
            }
        ));

        self.imp().entry_row_name.connect_text_notify(clone!(
            #[weak(rename_to = obj)]
            self,
            move |_| {
                obj.validate_name();
            }
        ));

        self.imp().combo_row_backend.connect_notify_local(
            Some("selected-item"),
            clone!(
                #[weak(rename_to = obj)]
                self,
                move |_, __| {
                    obj.combo_box_changed();
                }
            ),
        );

        self.imp().password_entry_row.connect_text_notify(clone!(
            #[weak(rename_to = obj)]
            self,
            move |_| {
                obj.validate_passwords();
            }
        ));

        self.imp()
            .confirm_password_entry_row
            .connect_text_notify(clone!(
                #[weak(rename_to = obj)]
                self,
                move |_| {
                    obj.validate_passwords();
                }
            ));

        self.imp()
            .encrypted_data_directory_entry_row
            .connect_text_notify(clone!(
                #[weak(rename_to = obj)]
                self,
                move |_| {
                    obj.validate_directories();
                }
            ));

        self.imp()
            .encrypted_data_directory_button
            .connect_clicked(clone!(
                #[weak(rename_to = obj)]
                self,
                move |_| {
                    obj.encrypted_data_directory_button_clicked();
                }
            ));

        self.imp()
            .mount_directory_entry_row
            .connect_text_notify(clone!(
                #[weak(rename_to = obj)]
                self,
                move |_| {
                    obj.validate_directories();
                }
            ));

        self.imp().mount_directory_button.connect_clicked(clone!(
            #[weak(rename_to = obj)]
            self,
            move |_| {
                obj.mount_directory_button_clicked();
            }
        ));
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

                self.validate_name();
            }
            1 => {
                self.imp().cancel_button.set_visible(false);
                self.imp().previous_button.set_visible(true);
                self.imp().next_button.set_visible(true);
                self.imp().add_button.set_visible(false);

                self.validate_passwords();
            }
            2 => {
                self.imp().next_button.set_visible(false);
                self.imp().add_button.set_visible(true);

                self.fill_directories();
            }
            _ => {}
        }
    }

    pub fn validate_name(&self) {
        let combo_box_text = self.imp().combo_row_backend.selected_item();

        if combo_box_text.is_none() {
            self.imp().next_button.set_sensitive(false);

            self.imp().name_error_label.set_visible(true);
            self.imp().name_error_label.set_text(&gettext(
                "No backend installed. Please install gocryptfs or CryFS.",
            ));

            return;
        }

        let vault_name = self.imp().entry_row_name.text();

        if vault_name.is_empty() {
            self.imp().next_button.set_sensitive(false);

            self.imp().entry_row_name.remove_css_class("error");
            self.imp().name_error_label.set_visible(false);
            self.imp().name_error_label.set_text("");

            return;
        }
    }

    pub fn combo_box_changed(&self) {
        let backend = backend::get_backend_from_ui_string(
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
        .unwrap();

        match backend {
            backend::Backend::Cryfs => {
                self.imp().info_label.set_text(&gettext("CryFS works well together with cloud services like Dropbox, iCloud, OneDrive and others. It does not expose directory structure, number of files or file sizes in the encrypted data directory. While being considered safe, there is no independent audit of CryFS."))
            },
            backend::Backend::Gocryptfs => {
                self.imp().info_label.set_text(&gettext("Fast and robust, gocryptfs works well in general cases where third-parties do not always have access to the encrypted data directory (e.g. file hosting services). It exposes directory structure, number of files and file sizes. A security audit in 2017 verified gocryptfs is safe against third-parties that can read or write to encrypted data."));
            }
        }
    }

    pub fn validate_passwords(&self) {
        let password = self.imp().password_entry_row.text();
        let confirm_password = self.imp().confirm_password_entry_row.text();

        if password.is_empty() && confirm_password.is_empty() {
            self.imp().next_button.set_sensitive(false);

            self.imp().password_entry_row.remove_css_class("error");
            self.imp()
                .confirm_password_entry_row
                .remove_css_class("error");

            return;
        }

        if password.eq(&confirm_password) {
            self.imp().next_button.set_sensitive(true);

            self.imp().password_entry_row.remove_css_class("error");
            self.imp()
                .confirm_password_entry_row
                .remove_css_class("error");
        } else {
            self.imp().next_button.set_sensitive(false);

            self.imp().password_entry_row.add_css_class("error");
            self.imp().confirm_password_entry_row.add_css_class("error");
        }
    }

    pub fn encrypted_data_directory_button_clicked(&self) {
        let window = gtk::gio::Application::default()
            .unwrap()
            .downcast_ref::<VApplication>()
            .unwrap()
            .active_window()
            .unwrap()
            .clone();

        glib::spawn_future_local(clone!(
            #[strong]
            window,
            #[strong(rename_to = obj)]
            self,
            async move {
                let dialog = gtk::FileDialog::builder()
                    .title(gettext("Choose Encrypted Data Directory"))
                    .modal(true)
                    .accept_label(gettext("Select"))
                    .build();

                dialog.select_folder(
                    Some(&window),
                    gio::Cancellable::NONE,
                    clone!(
                        #[strong]
                        obj,
                        move |directory| {
                            if let Ok(directory) = directory {
                                let path = String::from(
                                    directory.path().unwrap().as_os_str().to_str().unwrap(),
                                );
                                obj.imp().encrypted_data_directory_entry_row.set_text(&path);
                            }
                        }
                    ),
                );
            }
        ));
    }

    pub fn mount_directory_button_clicked(&self) {
        let window = gtk::gio::Application::default()
            .unwrap()
            .downcast_ref::<VApplication>()
            .unwrap()
            .active_window()
            .unwrap()
            .clone();

        glib::spawn_future_local(clone!(
            #[strong]
            window,
            #[strong(rename_to = obj)]
            self,
            async move {
                let dialog = gtk::FileDialog::builder()
                    .title(gettext("Choose Mount Directory"))
                    .modal(true)
                    .accept_label(gettext("Select"))
                    .build();

                dialog.select_folder(
                    Some(&window),
                    gio::Cancellable::NONE,
                    clone!(
                        #[strong]
                        obj,
                        move |directory| {
                            if let Ok(directory) = directory {
                                let path = String::from(
                                    directory.path().unwrap().as_os_str().to_str().unwrap(),
                                );
                                obj.imp().mount_directory_entry_row.set_text(&path);
                            }
                        }
                    ),
                );
            }
        ));
    }

    pub fn validate_directories(&self) {
        self.imp().add_button.set_sensitive(false);

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

        self.imp().add_button.set_sensitive(true);
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

        match self.is_path_empty(encrypted_data_directory) {
            Ok(is_empty) => {
                if is_empty {
                    self.imp()
                        .encrypted_data_directory_error_label
                        .set_visible(false);

                    self.imp()
                        .encrypted_data_directory_entry_row
                        .remove_css_class("error");

                    true
                } else {
                    self.imp()
                        .encrypted_data_directory_error_label
                        .set_text(&gettext("Encrypted data directory is not empty."));
                    self.imp()
                        .encrypted_data_directory_error_label
                        .set_visible(true);

                    self.imp()
                        .encrypted_data_directory_entry_row
                        .add_css_class("error");

                    false
                }
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => {
                    self.imp()
                        .encrypted_data_directory_error_label
                        .set_visible(false);

                    self.imp()
                        .encrypted_data_directory_entry_row
                        .remove_css_class("error");

                    true
                }
                _ => {
                    log::debug!("Encrypted data directory is not valid: {}", e);

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
            },
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

        match self.is_path_empty(mount_directory) {
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
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => {
                    self.imp().mount_directory_error_label.set_visible(false);

                    self.imp()
                        .mount_directory_entry_row
                        .remove_css_class("error");

                    true
                }
                _ => {
                    log::debug!("Mount directory is not valid: {}", e);

                    self.imp()
                        .mount_directory_error_label
                        .set_text(&gettext("Mount directory is not valid."));
                    self.imp().mount_directory_error_label.set_visible(true);

                    self.imp().mount_directory_entry_row.add_css_class("error");

                    false
                }
            },
        }
    }

    fn is_path_empty(&self, path: &GString) -> Result<bool, std::io::Error> {
        match std::fs::read_dir(path) {
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

    pub fn get_password(&self) -> String {
        String::from(self.imp().password_entry_row.text().as_str())
    }

    pub fn get_vault(&self) -> Vault {
        let backend = backend::get_backend_from_ui_string(
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
        .unwrap();

        Vault::new(
            util::generate_uuid(),
            String::from(self.imp().entry_row_name.text().as_str()),
            backend,
            String::from(
                self.imp()
                    .encrypted_data_directory_entry_row
                    .text()
                    .as_str(),
            ),
            String::from(self.imp().mount_directory_entry_row.text().as_str()),
            None,
            None,
            None,
        )
    }

    fn setup_combo_box(&self) {
        let list = gtk::StringList::new(&[]);

        for backend in backend::Backend::iter() {
            list.append(&backend.to_string());
        }

        self.combo_box_changed();
    }

    fn fill_directories(&self) {
        let vault_name = self.imp().entry_row_name.text().to_string();

        let mut path = self
            .imp()
            .settings
            .string("encrypted-data-directory")
            .to_string();
        if !path.ends_with("/") {
            path.push('/');
        }
        let encrypted_data_directory = path + &vault_name;

        path = self.imp().settings.string("mount-directory").to_string();
        if !path.ends_with("/") {
            path.push('/');
        }
        let mount_directory = path + &vault_name;

        self.imp()
            .encrypted_data_directory_entry_row
            .set_text(&encrypted_data_directory);
        self.imp()
            .mount_directory_entry_row
            .set_text(&mount_directory);

        self.validate_directories();
    }
}
