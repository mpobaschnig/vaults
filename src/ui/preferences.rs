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
use gtk::glib::subclass::Signal;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use once_cell::sync::Lazy;

use crate::application::VApplication;
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
        pub mount_directory_entry_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub mount_directory_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        // cryfs
        #[template_child]
        pub cryfs_custom_binary_expander_row: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub cryfs_custom_binary_entry_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub cryfs_custom_binary_button: TemplateChild<gtk::Button>,
        // gocryptfs
        #[template_child]
        pub gocryptfs_custom_binary_expander_row: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub gocryptfs_custom_binary_entry_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub gocryptfs_custom_binary_button: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VaultsSettingsWindow {
        const NAME: &'static str = "VaultSettingsWindow";
        type ParentType = adw::Dialog;
        type Type = super::VaultsSettingsWindow;

        fn new() -> Self {
            Self {
                encrypted_data_directory_entry_row: TemplateChild::default(),
                encrypted_data_directory_button: TemplateChild::default(),
                mount_directory_entry_row: TemplateChild::default(),
                mount_directory_button: TemplateChild::default(),
                toast_overlay: TemplateChild::default(),
                cryfs_custom_binary_expander_row: TemplateChild::default(),
                cryfs_custom_binary_entry_row: TemplateChild::default(),
                cryfs_custom_binary_button: TemplateChild::default(),
                gocryptfs_custom_binary_expander_row: TemplateChild::default(),
                gocryptfs_custom_binary_entry_row: TemplateChild::default(),
                gocryptfs_custom_binary_button: TemplateChild::default(),
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

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("refresh").build()]);
            SIGNALS.as_ref()
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
        log::trace!("init()");

        let global_config = GlobalConfigManager::instance().get_global_config();

        if let Some(encrypted_data_directory) =
            global_config.encrypted_data_directory.borrow().as_ref()
        {
            self.imp()
                .encrypted_data_directory_entry_row
                .set_text(encrypted_data_directory);
        } else {
            log::error!("Didn't find encrypted data directory");
        };

        if let Some(mount_directory) = global_config.mount_directory.borrow().as_ref() {
            self.imp()
                .mount_directory_entry_row
                .set_text(mount_directory);
        } else {
            log::error!("Didn't find mount directory");
        };

        if let Some(cryfs_custom_binary) = global_config.cryfs_custom_binary.borrow().as_ref() {
            self.imp()
                .cryfs_custom_binary_expander_row
                .set_enable_expansion(*cryfs_custom_binary);
            self.imp()
                .cryfs_custom_binary_expander_row
                .set_expanded(*cryfs_custom_binary);
        } else {
            log::error!("Didn't find cryfs custom binary");
        };

        if let Some(cryfs_custom_binary_path) =
            global_config.cryfs_custom_binary_path.borrow().as_ref()
        {
            self.imp()
                .cryfs_custom_binary_entry_row
                .set_text(cryfs_custom_binary_path);
        } else {
            log::error!("Didn't find cryfs custom binary path");
        };

        if let Some(gocryptfs_custom_binary) =
            global_config.gocryptfs_custom_binary.borrow().as_ref()
        {
            self.imp()
                .gocryptfs_custom_binary_expander_row
                .set_enable_expansion(*gocryptfs_custom_binary);
            self.imp()
                .gocryptfs_custom_binary_expander_row
                .set_expanded(*gocryptfs_custom_binary);
        } else {
            log::error!("Didn't find gocryptfs custom binary");
        };

        if let Some(gocryptfs_custom_binary_path) =
            global_config.gocryptfs_custom_binary_path.borrow().as_ref()
        {
            self.imp()
                .gocryptfs_custom_binary_entry_row
                .set_text(gocryptfs_custom_binary_path);
        } else {
            log::error!("Didn't find gocryptfs custom binary path");
        };
    }

    fn setup_signals(&self) {
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

        self.imp()
            .cryfs_custom_binary_expander_row
            .connect_enable_expansion_notify(clone!(
                #[weak(rename_to = obj)]
                self,
                move |expander_row| {
                    GlobalConfigManager::instance()
                        .set_cryfs_custom_binary(expander_row.is_expanded());
                    GlobalConfigManager::instance().write_config();
                    let toast = adw::Toast::new(&gettext("Saved preferences successfully!"));
                    obj.imp().toast_overlay.add_toast(toast);
                    obj.emit_by_name::<()>("refresh", &[]);
                }
            ));

        self.imp()
            .cryfs_custom_binary_entry_row
            .connect_apply(clone!(
                #[weak(rename_to = obj)]
                self,
                move |entry_row| {
                    let text = entry_row.text();
                    GlobalConfigManager::instance().set_cryfs_custom_binary_path(text.to_string());
                    GlobalConfigManager::instance().write_config();
                    let toast = adw::Toast::new(&gettext("Saved preferences successfully!"));
                    obj.imp().toast_overlay.add_toast(toast);
                    obj.emit_by_name::<()>("refresh", &[]);
                }
            ));

        self.imp()
            .gocryptfs_custom_binary_expander_row
            .connect_enable_expansion_notify(clone!(
                #[weak(rename_to = obj)]
                self,
                move |expander_row| {
                    GlobalConfigManager::instance()
                        .set_gocryptfs_custom_binary(expander_row.is_expanded());
                    GlobalConfigManager::instance().write_config();
                    let toast = adw::Toast::new(&gettext("Saved preferences successfully!"));
                    obj.imp().toast_overlay.add_toast(toast);
                    obj.emit_by_name::<()>("refresh", &[]);
                }
            ));

        self.imp()
            .gocryptfs_custom_binary_entry_row
            .connect_apply(clone!(
                #[weak(rename_to = obj)]
                self,
                move |entry_row| {
                    let text = entry_row.text();
                    GlobalConfigManager::instance()
                        .set_gocryptfs_custom_binary_path(text.to_string());
                    GlobalConfigManager::instance().write_config();
                    let toast = adw::Toast::new(&gettext("Saved preferences successfully!"));
                    obj.imp().toast_overlay.add_toast(toast);
                    obj.emit_by_name::<()>("refresh", &[]);
                }
            ));
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
}
