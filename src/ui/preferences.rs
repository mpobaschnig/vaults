// preferences.rs
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

use adw::prelude::*;
use adw::subclass::dialog::AdwDialogImpl;
use gettextrs::gettext;
use glib::clone;
use gtk::gio;
use gtk::glib::GString;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

use crate::GlobalConfigManager;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/io/github/mpobaschnig/Vaults/preferences.ui")]
    pub struct VaultsSettingsWindow {
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
        pub general_apply_changes_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VaultsSettingsWindow {
        const NAME: &'static str = "VaultSettingsWindow";
        type ParentType = adw::Dialog;
        type Type = super::VaultsSettingsWindow;

        fn new() -> Self {
            Self {
                general_apply_changes_button: TemplateChild::default(),
                encrypted_data_directory_entry_row: TemplateChild::default(),
                encrypted_data_directory_button: TemplateChild::default(),
                encrypted_data_directory_error_label: TemplateChild::default(),
                mount_directory_entry_row: TemplateChild::default(),
                mount_directory_button: TemplateChild::default(),
                mount_directory_error_label: TemplateChild::default(),
                toast_overlay: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for VaultsSettingsWindow {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for VaultsSettingsWindow {}
    impl WindowImpl for VaultsSettingsWindow {}
    impl AdwDialogImpl for VaultsSettingsWindow {}
}

glib::wrapper! {
    pub struct VaultsSettingsWindow(ObjectSubclass<imp::VaultsSettingsWindow>)
        @extends gtk::Widget, adw::Dialog, adw::Window, gtk::Window;
}

impl Default for VaultsSettingsWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl VaultsSettingsWindow {
    pub fn new() -> Self {
        let o: Self = glib::Object::builder().build();
        o.init();
        o.setup_signals();
        o
    }

    fn init(&self) {
        let global_config = GlobalConfigManager::instance().get_global_config();

        self.imp()
            .encrypted_data_directory_entry_row
            .set_text(&global_config.encrypted_data_directory.borrow());

        self.imp()
            .mount_directory_entry_row
            .set_text(&global_config.mount_directory.borrow());
    }

    fn setup_signals(&self) {
        self.imp()
            .encrypted_data_directory_entry_row
            .connect_text_notify(clone!(
                #[weak(rename_to = obj)]
                self,
                move |_| {
                    obj.check_apply_changes_button_enable_conditions();
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
                    obj.check_apply_changes_button_enable_conditions();
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
            .general_apply_changes_button
            .connect_clicked(clone!(
                #[weak(rename_to = obj)]
                self,
                move |_| {
                    obj.general_apply_changes_button_clicked();
                }
            ));
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

    fn general_apply_changes_button_clicked(&self) {
        let encrypted_data_directory = self
            .imp()
            .encrypted_data_directory_entry_row
            .text()
            .to_string();
        let mount_directory = self.imp().mount_directory_entry_row.text().to_string();

        GlobalConfigManager::instance().set_encrypted_data_directory(encrypted_data_directory);
        GlobalConfigManager::instance().set_mount_directory(mount_directory);

        GlobalConfigManager::instance().write_config();

        let toast = adw::Toast::new(&gettext("Saved preferences successfully!"));
        self.imp().toast_overlay.add_toast(toast);
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

    fn check_apply_changes_button_enable_conditions(&self) {
        let encrypted_data_directory = self.imp().encrypted_data_directory_entry_row.text();
        let mount_directory = self.imp().mount_directory_entry_row.text();

        let are_directories_different =
            self.are_directories_different(&encrypted_data_directory, &mount_directory);

        if are_directories_different {
            self.imp().general_apply_changes_button.set_sensitive(true);
        } else {
            self.imp().general_apply_changes_button.set_sensitive(false);
        }
    }
}
