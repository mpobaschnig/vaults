// vaults_page_row_settings_window.rs
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

use adw::prelude::MessageDialogExtManual;
use adw::subclass::prelude::*;
use adw::{prelude::ComboRowExt, prelude::MessageDialogExt, prelude::PreferencesGroupExt};
use gettextrs::gettext;
use gtk::{
    self, gio,
    glib::{self, clone, GString},
    prelude::*,
    CompositeTemplate,
};
use std::cell::RefCell;
use strum::IntoEnumIterator;

use crate::{
    backend, backend::Backend, user_config_manager::UserConfigManager, vault::*, VApplication,
};

mod imp {
    use crate::ui::pages::vaults_page_row_settings_window;
    use gtk::glib::subclass::Signal;
    use once_cell::sync::Lazy;

    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/io/github/mpobaschnig/Vaults/vaults_page_row_settings_window.ui")]
    pub struct VaultsPageRowSettingsWindow {
        #[template_child]
        pub remove_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub apply_changes_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub name_entry_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub name_error_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub combo_row_backend: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub backend_error_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub encrypted_data_directory_entry_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub encrypted_data_directory_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub encrypted_data_directory_error_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub mount_directory_entry_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub mount_directory_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub mount_directory_error_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub lock_screen_switch_row: TemplateChild<adw::SwitchRow>,

        pub current_vault: RefCell<Option<Vault>>,
        pub to_remove: RefCell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VaultsPageRowSettingsWindow {
        const NAME: &'static str = "VaultsPageRowSettingsWindow";
        type ParentType = adw::Window;
        type Type = vaults_page_row_settings_window::VaultsPageRowSettingsWindow;

        fn new() -> Self {
            Self {
                remove_button: TemplateChild::default(),
                apply_changes_button: TemplateChild::default(),
                name_entry_row: TemplateChild::default(),
                name_error_label: TemplateChild::default(),
                combo_row_backend: TemplateChild::default(),
                backend_error_label: TemplateChild::default(),
                encrypted_data_directory_entry_row: TemplateChild::default(),
                encrypted_data_directory_button: TemplateChild::default(),
                encrypted_data_directory_error_label: TemplateChild::default(),
                mount_directory_entry_row: TemplateChild::default(),
                mount_directory_button: TemplateChild::default(),
                mount_directory_error_label: TemplateChild::default(),
                toast_overlay: TemplateChild::default(),
                lock_screen_switch_row: TemplateChild::default(),
                current_vault: RefCell::new(None),
                to_remove: RefCell::new(false),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for VaultsPageRowSettingsWindow {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("save").build(),
                    Signal::builder("remove").build(),
                ]
            });
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for VaultsPageRowSettingsWindow {}
    impl AdwWindowImpl for VaultsPageRowSettingsWindow {}
    impl WindowImpl for VaultsPageRowSettingsWindow {}
    impl DialogImpl for VaultsPageRowSettingsWindow {}
}

glib::wrapper! {
    pub struct VaultsPageRowSettingsWindow(ObjectSubclass<imp::VaultsPageRowSettingsWindow>)
        @extends gtk::Widget, adw::Window, gtk::Window;
}

impl VaultsPageRowSettingsWindow {
    pub fn new(vault: Vault) -> Self {
        let dialog: Self = glib::Object::builder().build();

        let window = gio::Application::default()
            .unwrap()
            .downcast_ref::<VApplication>()
            .unwrap()
            .active_window()
            .unwrap();
        dialog.set_transient_for(Some(&window));

        dialog.fill_combo_box_text();
        dialog.set_vault(vault);
        dialog.setup_signals();

        dialog
    }

    fn setup_signals(&self) {
        self.imp().remove_button.connect_clicked(clone!(
            #[weak(rename_to = obj)]
            self,
            move |_| {
                obj.remove_button_clicked();
            }
        ));

        self.imp().apply_changes_button.connect_clicked(clone!(
            #[weak(rename_to = obj)]
            self,
            move |_| {
                obj.apply_changes_button_clicked();
            }
        ));

        self.imp().name_entry_row.connect_text_notify(clone!(
            #[weak(rename_to = obj)]
            self,
            move |_| {
                obj.check_add_button_enable_conditions();
            }
        ));

        self.imp().combo_row_backend.connect_selected_notify(clone!(
            #[weak(rename_to = obj)]
            self,
            move |_| {
                obj.check_add_button_enable_conditions();
            }
        ));

        self.imp()
            .encrypted_data_directory_entry_row
            .connect_text_notify(clone!(
                #[weak(rename_to = obj)]
                self,
                move |_| {
                    obj.check_add_button_enable_conditions();
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
                    obj.check_add_button_enable_conditions();
                }
            ));

        self.imp().mount_directory_button.connect_clicked(clone!(
            #[weak(rename_to = obj)]
            self,
            move |_| {
                obj.mount_directory_button_clicked();
            }
        ));

        self.imp()
            .lock_screen_switch_row
            .connect_active_notify(clone!(
                #[weak(rename_to = obj)]
                self,
                move |_| {
                    obj.check_add_button_enable_conditions();
                }
            ));
    }

    fn remove_button_clicked(&self) {
        let confirm_dialog = adw::MessageDialog::builder()
            .heading(gettext("Remove Vault?"))
            .default_response(gettext("Cancel"))
            .transient_for(self)
            .build();

        confirm_dialog.add_responses(&[
            ("cancel", &gettext("Cancel")),
            ("remove", &gettext("Remove")),
        ]);
        confirm_dialog.set_response_appearance("remove", adw::ResponseAppearance::Destructive);
        confirm_dialog.set_default_response(Some("cancel"));
        confirm_dialog.set_close_response("cancel");

        let switch_row = adw::SwitchRow::builder()
            .title(gettext("Delete encrypted data"))
            .build();
        let preference_group = adw::PreferencesGroup::builder().build();
        preference_group.add(&switch_row);
        confirm_dialog.set_extra_child(Some(&preference_group));

        confirm_dialog.choose(
            None::<&gio::Cancellable>,
            clone!(
                #[strong(rename_to = obj)]
                self,
                #[strong(rename_to = sr)]
                switch_row,
                move |s| {
                    match s.as_str() {
                        "cancel" => (),
                        "remove" => {
                            log::info!("Removing {}", obj.get_vault().get_name().unwrap());

                            let vault = obj.get_vault();

                            if sr.is_active() {
                                match vault.delete_encrypted_data() {
                                    Ok(_) => (),
                                    Err(e) => {
                                        log::error!(
                                            "Could not delete encrypted data: {}",
                                            e.kind()
                                        );
                                        let err_dialog = adw::MessageDialog::builder()
                                            .transient_for(&obj)
                                            .body(gettext("Could not remove encrypted data."))
                                            .build();
                                        err_dialog.add_response("close", &gettext("Close"));
                                        err_dialog.set_default_response(Some("close"));
                                        err_dialog.choose(
                                            None::<&gio::Cancellable>,
                                            clone!(
                                                #[strong(rename_to = _o)]
                                                obj,
                                                move |_o| {}
                                            ),
                                        );
                                    }
                                }
                            }

                            UserConfigManager::instance().remove_vault(vault);
                            obj.emit_by_name::<()>("remove", &[])
                        }
                        _ => todo!(),
                    }
                }
            ),
        );
    }

    fn apply_changes_button_clicked(&self) {
        let new_vault = Vault::new(
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
            Some(self.imp().lock_screen_switch_row.is_active()),
        );

        UserConfigManager::instance()
            .change_vault(self.get_current_vault().unwrap(), new_vault.clone());

        *self.imp().current_vault.borrow_mut() = Some(new_vault);

        let toast = adw::Toast::new(&gettext("Saved settings successfully!"));
        self.imp().toast_overlay.add_toast(toast);

        self.emit_by_name::<()>("save", &[]);
    }

    fn encrypted_data_directory_button_clicked(&self) {
        let dialog = gtk::FileDialog::builder()
            .title(gettext("Choose Encrypted Data Directory"))
            .modal(true)
            .accept_label(gettext("Select"))
            .build();

        dialog.select_folder(
            Some(self),
            gio::Cancellable::NONE,
            clone!(
                #[weak(rename_to = obj)]
                self,
                move |directory| {
                    if let Ok(directory) = directory {
                        let path =
                            String::from(directory.path().unwrap().as_os_str().to_str().unwrap());
                        obj.imp().encrypted_data_directory_entry_row.set_text(&path);
                    }
                }
            ),
        );
    }

    fn mount_directory_button_clicked(&self) {
        let dialog = gtk::FileDialog::builder()
            .title(gettext("Choose Mount Directory"))
            .modal(true)
            .accept_label(gettext("Select"))
            .build();

        dialog.select_folder(
            Some(self),
            gio::Cancellable::NONE,
            clone!(
                #[weak(rename_to = obj)]
                self,
                move |directory| {
                    if let Ok(directory) = directory {
                        let path =
                            String::from(directory.path().unwrap().as_os_str().to_str().unwrap());
                        obj.imp().mount_directory_entry_row.set_text(&path);
                    }
                }
            ),
        );
    }

    fn is_valid_vault_name(&self, vault_name: GString) -> bool {
        if vault_name.is_empty() {
            self.imp().name_entry_row.add_css_class("error");

            self.imp()
                .name_error_label
                .set_text(&gettext("Name is not valid."));

            self.imp().name_error_label.set_visible(true);

            false
        } else {
            self.imp().name_entry_row.remove_css_class("error");

            self.imp().name_error_label.set_visible(false);

            true
        }
    }

    fn is_different_vault_name(&self, vault_name: GString) -> bool {
        let is_same_name = vault_name.eq(&self.get_current_vault().unwrap().get_name().unwrap());
        let is_duplicate_name = UserConfigManager::instance()
            .get_map()
            .contains_key(&vault_name.to_string());
        if !vault_name.is_empty() && !is_same_name && is_duplicate_name {
            self.imp().name_entry_row.add_css_class("error");

            self.imp()
                .name_error_label
                .set_text(&gettext("Name already exists."));

            self.imp().name_error_label.set_visible(true);

            false
        } else {
            self.imp().name_entry_row.remove_css_class("error");

            self.imp().name_error_label.set_visible(false);

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
        match self.is_path_empty(encrypted_data_directory) {
            Ok(is_empty) => {
                if is_empty {
                    self.imp()
                        .encrypted_data_directory_error_label
                        .set_text(&gettext("Directory is empty."));

                    self.imp()
                        .encrypted_data_directory_error_label
                        .set_visible(true);

                    false
                } else {
                    self.imp()
                        .encrypted_data_directory_error_label
                        .set_visible(false);

                    true
                }
            }
            Err(_) => {
                self.imp()
                    .encrypted_data_directory_error_label
                    .set_text(&gettext("Directory is not valid."));

                false
            }
        }
    }

    fn is_mount_directory_valid(&self, mount_directory: &GString) -> bool {
        match self.is_path_empty(mount_directory) {
            Ok(is_empty) => {
                if is_empty {
                    self.imp().mount_directory_error_label.set_text("");
                    self.imp().mount_directory_error_label.set_visible(true);

                    true
                } else {
                    self.imp()
                        .mount_directory_error_label
                        .set_text(&gettext("Directory is not empty."));

                    self.imp().mount_directory_error_label.set_visible(true);

                    false
                }
            }
            Err(_) => {
                self.imp()
                    .mount_directory_error_label
                    .set_text(&gettext("Directory is not valid."));

                self.imp().mount_directory_error_label.set_visible(true);

                false
            }
        }
    }

    fn are_directories_different(
        &self,
        encrypted_data_directory: &GString,
        mount_directory: &GString,
    ) -> bool {
        if encrypted_data_directory.eq(mount_directory) {
            self.imp()
                .mount_directory_error_label
                .set_text(&gettext("Directories must not be equal."));

            self.imp().mount_directory_error_label.set_visible(true);

            false
        } else {
            self.imp().mount_directory_error_label.set_visible(false);

            true
        }
    }

    fn has_something_changed(
        &self,
        curr_vault_name: &GString,
        curr_backend: &GString,
        curr_encrypted_data_directory: &GString,
        curr_mount_directory: &GString,
        curr_session_locking: &bool,
    ) -> bool {
        let prev_vault = self.get_current_vault().unwrap();
        let prev_config = &prev_vault.get_config().unwrap();

        let prev_vault_name = &prev_vault.get_name().unwrap();
        let prev_backend = backend::get_ui_string_from_backend(&prev_config.backend);

        let prev_encrypted_data_directory = &prev_config.encrypted_data_directory;
        let prev_mount_directory = &prev_config.mount_directory;

        if !curr_vault_name.eq(prev_vault_name) {
            return true;
        }

        if !curr_backend.eq(&prev_backend) {
            return true;
        }

        if !curr_encrypted_data_directory.eq(prev_encrypted_data_directory) {
            return true;
        }

        if !curr_mount_directory.eq(prev_mount_directory) {
            return true;
        }

        if let Some(prev_session_locking) = prev_config.session_lock {
            if prev_session_locking != *curr_session_locking {
                return true;
            }
        } else if *curr_session_locking {
            return true;
        }

        false
    }

    fn exists_config_file(&self, backend: Backend, encrypted_data_directory: &GString) -> bool {
        if !self.is_encrypted_data_directory_valid(encrypted_data_directory) {
            self.imp().backend_error_label.set_visible(false);

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
            self.imp().backend_error_label.set_visible(false);

            true
        } else {
            self.imp()
                .backend_error_label
                .set_text(&gettext("No configuration file found."));
            self.imp().backend_error_label.set_visible(true);

            false
        }
    }

    fn check_add_button_enable_conditions(&self) {
        let vault_name = self.imp().name_entry_row.text();
        let backend_str = &self
            .imp()
            .combo_row_backend
            .selected_item()
            .unwrap()
            .downcast::<gtk::StringObject>()
            .unwrap()
            .string();
        let backend = backend::get_backend_from_ui_string(&backend_str.to_string()).unwrap();
        let encrypted_data_directory = self.imp().encrypted_data_directory_entry_row.text();
        let mount_directory = self.imp().mount_directory_entry_row.text();

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
        let is_session_locking = self.imp().lock_screen_switch_row.is_active();
        let has_something_changed = self.has_something_changed(
            &vault_name,
            backend_str,
            &encrypted_data_directory,
            &mount_directory,
            &is_session_locking,
        );
        let exists_config_file = self.exists_config_file(backend, &encrypted_data_directory);

        if is_valid_vault_name
            && is_different_vault_name
            && is_encrypted_data_directory_valid
            && is_mount_directory_valid
            && are_directories_different
            && has_something_changed
            && exists_config_file
        {
            self.imp().apply_changes_button.set_sensitive(true);
        } else {
            self.imp().apply_changes_button.set_sensitive(false);
        }
    }

    fn fill_combo_box_text(&self) {
        let list = gtk::StringList::new(&[]);

        for backend in Backend::iter() {
            let backend = &backend::get_ui_string_from_backend(&backend);
            list.append(backend);
        }

        self.imp().combo_row_backend.set_model(Some(&list));
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
            Some(self.imp().lock_screen_switch_row.is_active()),
        )
    }

    pub fn get_current_vault(&self) -> Option<Vault> {
        self.imp().current_vault.borrow().clone()
    }

    pub fn set_vault(&self, vault: Vault) {
        match (vault.get_name(), vault.get_config()) {
            (Some(name), Some(config)) => {
                self.imp().current_vault.replace(Some(vault.clone()));

                self.imp().name_entry_row.set_text(&name);
                let model = self.imp().combo_row_backend.model().unwrap();
                for (position, item) in model.iter::<glib::Object>().enumerate() {
                    if let Ok(object) = item {
                        let string_object = object.downcast::<gtk::StringObject>().unwrap();
                        if string_object
                            .string()
                            .eq(&backend::get_ui_string_from_backend(&config.backend))
                        {
                            self.imp().combo_row_backend.set_selected(position as u32);
                        }
                    }
                }
                self.imp()
                    .encrypted_data_directory_entry_row
                    .set_text(&config.encrypted_data_directory.to_string());
                self.imp()
                    .mount_directory_entry_row
                    .set_text(&config.mount_directory.to_string());
                if let Some(session_lock) = config.session_lock {
                    self.imp().lock_screen_switch_row.set_active(session_lock);
                }
            }
            (_, _) => {
                log::error!("Vault not initialised!");
                return;
            }
        }
    }
}
