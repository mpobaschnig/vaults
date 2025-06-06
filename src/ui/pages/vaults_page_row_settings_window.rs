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
use adw::prelude::AlertDialogExt;
use adw::prelude::AlertDialogExtManual;
use adw::prelude::EntryRowExt;
use adw::subclass::dialog::AdwDialogImpl;
use adw::subclass::prelude::*;
use adw::{prelude::ComboRowExt, prelude::PreferencesGroupExt};
use gettextrs::gettext;
use gtk::glib::Properties;
use gtk::glib::subclass::Signal;
use gtk::{
    self, CompositeTemplate, gio,
    glib::{self, GString, clone},
    prelude::*,
};
use once_cell::sync::Lazy;
use std::cell::Cell;

use std::cell::RefCell;
use strum::IntoEnumIterator;

use crate::{backend, backend::Backend, user_config_manager::UserConfigManager, vault::*};

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate, Properties)]
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
        #[property(get, set, name = "vault")]
        pub(super) vault: RefCell<Option<Vault>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VaultsPageRowSettingsWindow {
        const NAME: &'static str = "VaultsPageRowSettingsWindow";
        type ParentType = adw::Dialog;
        type Type = vaults_page_row_settings_window::VaultsPageRowSettingsWindow;

        fn new() -> Self {
            Self {
                name_entry_row: TemplateChild::default(),
                combo_row_backend: TemplateChild::default(),
                encrypted_data_directory_entry_row: TemplateChild::default(),
                encrypted_data_directory_button: TemplateChild::default(),
                mount_directory_entry_row: TemplateChild::default(),
                mount_directory_button: TemplateChild::default(),
                lock_screen_switch_row: TemplateChild::default(),
                vault: RefCell::new(None),
            }
        }

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

            self.obj().connect_vault_notify(clone!(move |obj| {
                obj.emit_by_name::<()>("save", &[]);
            }));
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
        let dialog: Self = glib::Object::builder().build();

        dialog.fill_combo_box_text();
        dialog.set_vault(vault);
        dialog.setup_signals();

        dialog
    }

    fn setup_signals(&self) {
        self.imp().name_entry_row.connect_apply(clone!(
            #[weak(rename_to = obj)]
            self,
            move |_| {
                obj.apply_changes();
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

        self.imp().mount_directory_button.connect_clicked(clone!(
            #[weak(rename_to = obj)]
            self,
            move |_| {
                obj.mount_directory_button_clicked();
            }
        ));
    }

    fn remove_button_clicked(&self) {
        let confirm_dialog = adw::AlertDialog::builder()
            .heading(gettext("Remove Vault?"))
            .default_response(gettext("Cancel"))
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
            self,
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
                            log::info!("Removing {}", obj.vault().unwrap().get_name().unwrap());

                            let vault = obj.vault().unwrap();

                            if sr.is_active() {
                                match vault.delete_encrypted_data() {
                                    Ok(_) => (),
                                    Err(e) => {
                                        log::error!(
                                            "Could not delete encrypted data: {}",
                                            e.kind()
                                        );
                                        let err_dialog = adw::AlertDialog::builder()
                                            .body(gettext("Could not remove encrypted data."))
                                            .build();
                                        err_dialog.add_response("close", &gettext("Close"));
                                        err_dialog.set_default_response(Some("close"));
                                        err_dialog.choose(
                                            &obj,
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

    fn apply_changes(&self) {
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
            Some(false),
            None,
        );

        UserConfigManager::instance().change_vault(self.vault().unwrap(), new_vault.clone());

        //*self.imp().vault.borrow_mut() = Some(new_vault);
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
                            }
                        }
                    ),
                );
            }
        ));
    }

    fn fill_combo_box_text(&self) {
        let list = gtk::StringList::new(&[]);

        for backend in Backend::iter() {
            let backend = &backend::get_ui_string_from_backend(&backend);
            list.append(backend);
        }

        self.imp().combo_row_backend.set_model(Some(&list));
    }

    pub fn create_vault_from_settings(&self) -> Vault {
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
            Some(false),
            None,
        )
    }

    //pub fn vault(&self) -> Option<Vault> {
    //    self.imp().vault.borrow().clone()
    //}

    //pub fn set_vault(&self, vault: Vault) {
    //    match (vault.get_name(), vault.get_config()) {
    //        (Some(name), Some(config)) => {
    //            self.imp().vault.replace(Some(vault.clone()));

    //            self.imp().name_entry_row.set_text(&name);
    //            let model = self.imp().combo_row_backend.model().unwrap();
    //            for (position, item) in model.iter::<glib::Object>().enumerate() {
    //                if let Ok(object) = item {
    //                    let string_object = object.downcast::<gtk::StringObject>().unwrap();
    //                    if string_object
    //                        .string()
    //                        .eq(&backend::get_ui_string_from_backend(&config.backend))
    //                    {
    //                        self.imp().combo_row_backend.set_selected(position as u32);
    //                    }
    //                }
    //            }
    //            self.imp()
    //                .encrypted_data_directory_entry_row
    //                .set_text(&config.encrypted_data_directory.to_string());
    //            self.imp()
    //                .mount_directory_entry_row
    //                .set_text(&config.mount_directory.to_string());
    //            if let Some(session_lock) = config.session_lock {
    //                self.imp().lock_screen_switch_row.set_active(session_lock);
    //            }
    //        }
    //        (_, _) => {
    //            log::error!("Vault not initialised!");
    //        }
    //    }
    //}
}
