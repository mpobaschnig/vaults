// import_vault_dialog.rs
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
use gtk::{self, gio, glib, glib::clone, prelude::*, subclass::prelude::*, CompositeTemplate};

use crate::{
    backend::{Backend, AVAILABLE_BACKENDS},
    user_config_manager::UserConfig,
    vault::*,
    VApplication,
};

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/mpobaschnig/Vaults/import_vault_dialog.ui")]
    pub struct ImportVaultDialog {
        #[template_child]
        pub cancel_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub import_vault_button: TemplateChild<gtk::Button>,
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
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ImportVaultDialog {
        const NAME: &'static str = "ImportVaultDialog";
        type ParentType = gtk::Dialog;
        type Type = super::ImportVaultDialog;

        fn new() -> Self {
            Self {
                cancel_button: TemplateChild::default(),
                import_vault_button: TemplateChild::default(),
                vault_name_action_row: TemplateChild::default(),
                vault_name_entry: TemplateChild::default(),
                backend_type_combo_box_text: TemplateChild::default(),
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
        let self_ = imp::ImportVaultDialog::from_instance(self);

        self_
            .cancel_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.response(gtk::ResponseType::Cancel);
            }));

        self_
            .import_vault_button
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
        let file_chooser = gtk::FileChooserDialog::new(
            Some(&gettext("Choose Encrypted Data Directory")),
            Some(self),
            gtk::FileChooserAction::SelectFolder,
            &[
                (&gettext("Cancel"), gtk::ResponseType::Cancel),
                (&gettext("Select"), gtk::ResponseType::Accept),
            ],
        );

        file_chooser.set_transient_for(Some(self));

        file_chooser.connect_response(
            clone!(@weak self as obj, @strong file_chooser => move |s, response| {
                if response == gtk::ResponseType::Accept {
                    let file = file_chooser.get_file().unwrap();
                    let path = String::from(file.get_path().unwrap().as_os_str().to_str().unwrap());
                    let self_ = imp::ImportVaultDialog::from_instance(&obj);
                    self_.encrypted_data_directory_entry.set_text(&path);
                }
                s.destroy();
            }),
        );

        file_chooser.show();
    }

    fn mount_directory_button_clicked(&self) {
        let file_chooser = gtk::FileChooserDialog::new(
            Some(&gettext("Choose Mount Directory")),
            Some(self),
            gtk::FileChooserAction::SelectFolder,
            &[
                (&gettext("Cancel"), gtk::ResponseType::Cancel),
                (&gettext("Select"), gtk::ResponseType::Accept),
            ],
        );

        file_chooser.set_transient_for(Some(self));

        file_chooser.connect_response(
            clone!(@weak self as obj, @strong file_chooser => move |s, response| {
                if response == gtk::ResponseType::Accept {
                    let file = file_chooser.get_file().unwrap();
                    let path = String::from(file.get_path().unwrap().as_os_str().to_str().unwrap());
                    let self_ = imp::ImportVaultDialog::from_instance(&obj);
                    self_.mount_directory_entry.set_text(&path);
                }
                s.destroy();
            }),
        );

        file_chooser.show();
    }

    fn check_add_button_enable_conditions(&self) {
        let self_ = imp::ImportVaultDialog::from_instance(self);

        let vault_name = self_.vault_name_entry.get_text();
        let backend = self_.backend_type_combo_box_text.get_active();
        let encrypted_data_directory = self_.encrypted_data_directory_entry.get_text();
        let mount_directory = self_.mount_directory_entry.get_text();

        let is_duplicate_name = UserConfig::instance()
            .get_map()
            .contains_key(&vault_name.to_string());
        if !vault_name.is_empty() && is_duplicate_name {
            self_
                .vault_name_action_row
                .set_subtitle(Some(&gettext("Name already exists.")));
        } else {
            if vault_name.is_empty() {
                self_
                    .vault_name_action_row
                    .set_subtitle(Some(&gettext("Name is not valid.")));
            } else {
                self_.vault_name_action_row.set_subtitle(Some(""));
            }
        }

        let mut is_mount_directory_empty = false;
        match std::fs::read_dir(mount_directory.to_string()) {
            Ok(dir) => {
                if dir.count() > 0 {
                    self_
                        .mount_directory_action_row
                        .set_subtitle(Some(&gettext("Directory is not empty.")));
                } else {
                    is_mount_directory_empty = true;
                    self_
                        .mount_directory_action_row
                        .set_subtitle(Some(&gettext("")));
                }
            }
            Err(e) => {
                log::debug!("Could not read mount directory: {}", e);
                self_
                    .mount_directory_action_row
                    .set_subtitle(Some(&gettext("Directory is not valid.")));
            }
        }

        if is_mount_directory_empty && encrypted_data_directory.eq(&mount_directory) {
            self_
                .encrypted_data_directory_action_row
                .set_subtitle(Some(&gettext("Directories must not be equal.")));
            self_
                .mount_directory_action_row
                .set_subtitle(Some(&gettext("Directories must not be equal.")));
        }

        if !vault_name.is_empty()
            && !encrypted_data_directory.is_empty()
            && !mount_directory.is_empty()
            && backend.is_some()
            && !is_duplicate_name
            && is_mount_directory_empty
        {
            self_.import_vault_button.set_sensitive(true);
        } else {
            self_.import_vault_button.set_sensitive(false);
        }
    }

    pub fn get_vault(&self) -> Vault {
        let self_ = imp::ImportVaultDialog::from_instance(self);

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

    fn fill_combo_box_text(&self) {
        let self_ = imp::ImportVaultDialog::from_instance(self);

        let combo_box_text = &self_.backend_type_combo_box_text;

        let backends_res = AVAILABLE_BACKENDS.lock();
        match backends_res {
            Ok(backends) => {
                for backend in backends.iter() {
                    combo_box_text.append_text(backend);
                }

                if !backends.is_empty() {
                    combo_box_text.set_active(Some(0));
                }
            }
            Err(e) => {
                log::error!("Failed to aquire mutex lock of AVAILABLE_BACKENDS: {}", e);
            }
        }
    }
}
