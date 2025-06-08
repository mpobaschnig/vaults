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

use crate::application::VApplication;
use crate::ui::pages::vaults_page_row_settings_window;
use crate::vault::Vault;
use crate::{backend, backend::Backend, user_config_manager::UserConfigManager};
use adw::{
    prelude::{ComboRowExt, EntryRowExt},
    subclass::{dialog::AdwDialogImpl, prelude::*},
};
use gettextrs::gettext;
use gtk::{
    self, CompositeTemplate, gio,
    glib::{self, Properties, clone, subclass::Signal},
    prelude::*,
};
use once_cell::sync::Lazy;
use std::cell::RefCell;
use strum::IntoEnumIterator;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate, Properties, Default)]
    #[properties(wrapper_type = super::VaultsPageRowSettingsWindow)]
    #[template(resource = "/io/github/mpobaschnig/Vaults/vaults_page_row_settings_window.ui")]
    pub struct VaultsPageRowSettingsWindow {
        #[template_child]
        pub name_entry_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub combo_row_backend: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub encrypted_data_directory_entry_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub encrypted_data_directory_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub mount_directory_entry_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub mount_directory_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub lock_screen_switch_row: TemplateChild<adw::SwitchRow>,
        #[property(get, set, name = "vault", construct)]
        pub vault: RefCell<Option<Vault>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VaultsPageRowSettingsWindow {
        const NAME: &'static str = "VaultsPageRowSettingsWindow";
        type ParentType = adw::Dialog;
        type Type = vaults_page_row_settings_window::VaultsPageRowSettingsWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for VaultsPageRowSettingsWindow {
        fn constructed(&self) {
            self.parent_constructed();

            self.name_entry_row
                .set_text(&self.obj().vault().unwrap().get_name().unwrap());

            let mut model_position = 0;
            let vault_backend = self.obj().vault().unwrap().get_config().unwrap().backend;
            let list = gtk::StringList::new(&[]);
            for (position, backend) in Backend::iter().enumerate() {
                let backend = &backend::get_ui_string_from_backend(&backend);
                list.append(backend);
                if &backend::get_ui_string_from_backend(&vault_backend) == backend {
                    model_position = position;
                }
            }
            self.combo_row_backend.set_model(Some(&list));
            self.combo_row_backend.set_selected(model_position as u32);

            self.encrypted_data_directory_entry_row.set_text(
                &self
                    .obj()
                    .vault()
                    .unwrap()
                    .get_config()
                    .unwrap()
                    .encrypted_data_directory,
            );

            self.mount_directory_entry_row.set_text(
                &self
                    .obj()
                    .vault()
                    .unwrap()
                    .get_config()
                    .unwrap()
                    .mount_directory,
            );

            self.lock_screen_switch_row.set_active(
                self.obj()
                    .vault()
                    .unwrap()
                    .get_config()
                    .unwrap()
                    .session_lock
                    .unwrap_or(false),
            );

            self.obj().connect_vault_notify(clone!(move |obj| {
                obj.emit_by_name::<()>("save", &[]);
            }));

            self.name_entry_row.connect_apply(clone!(
                #[weak(rename_to = s)]
                self,
                move |_| {
                    s.obj().apply_changes();
                }
            ));

            self.encrypted_data_directory_entry_row
                .connect_apply(clone!(
                    #[weak(rename_to = s)]
                    self,
                    move |_| {
                        s.obj().apply_changes();
                    }
                ));

            self.encrypted_data_directory_button.connect_clicked(clone!(
                #[weak(rename_to = s)]
                self,
                move |_| {
                    s.obj().encrypted_data_directory_button_clicked();
                }
            ));

            self.mount_directory_entry_row.connect_apply(clone!(
                #[weak(rename_to = s)]
                self,
                move |_| {
                    s.obj().apply_changes();
                }
            ));

            self.mount_directory_button.connect_clicked(clone!(
                #[weak(rename_to = s)]
                self,
                move |_| {
                    s.obj().mount_directory_button_clicked();
                }
            ));

            self.lock_screen_switch_row.connect_active_notify(clone!(
                #[weak(rename_to = s)]
                self,
                move |_| {
                    s.obj().apply_changes();
                }
            ));
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

    impl AdwDialogImpl for VaultsPageRowSettingsWindow {}
    impl DialogImpl for VaultsPageRowSettingsWindow {}
    impl WidgetImpl for VaultsPageRowSettingsWindow {}
    impl WindowImpl for VaultsPageRowSettingsWindow {}
}

glib::wrapper! {
    pub struct VaultsPageRowSettingsWindow(ObjectSubclass<imp::VaultsPageRowSettingsWindow>)
        @extends gtk::Widget, adw::Dialog, adw::Window, gtk::Window;
}

impl VaultsPageRowSettingsWindow {
    pub fn new(vault: Vault) -> Self {
        glib::Object::builder().property("vault", vault).build()
    }

    fn apply_changes(&self) {
        let new_vault = Vault::new(
            self.vault().unwrap().get_uuid(),
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
            self.vault()
                .unwrap()
                .get_config()
                .unwrap()
                .use_custom_binary,
            self.vault()
                .unwrap()
                .get_config()
                .unwrap()
                .custom_binary_path,
        );

        UserConfigManager::instance().change_vault(
            self.vault().unwrap().get_uuid(),
            new_vault.get_config().unwrap().clone(),
        );
        self.set_vault(new_vault);
        self.notify_vault();
    }

    fn encrypted_data_directory_button_clicked(&self) {
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
                                obj.apply_changes();
                            }
                        }
                    ),
                );
            }
        ));
    }

    fn mount_directory_button_clicked(&self) {
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
                                obj.apply_changes();
                            }
                        }
                    ),
                );
            }
        ));
    }

    pub fn create_vault_from_settings(&self) -> Vault {
        Vault::new(
            self.vault().unwrap().get_uuid(),
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
            Some(false),
            None,
        )
    }
}
