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
use gtk::gio::{Settings, SettingsBindFlags};
use gtk::glib::subclass::Signal;
use gtk::subclass::prelude::*;
use gtk::{CompositeTemplate, glib};
use once_cell::sync::Lazy;

use crate::application::VApplication;
use crate::config::APP_ID;

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

        pub settings: Settings,
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
        o.setup_signals();
        o
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
            .settings
            .bind(
                "encrypted-data-directory",
                &self.imp().encrypted_data_directory_entry_row.get(),
                "text",
            )
            .build();

        self.imp()
            .settings
            .bind(
                "mount-directory",
                &self.imp().mount_directory_entry_row.get(),
                "text",
            )
            .build();

        self.imp()
            .settings
            .bind(
                "use-custom-cryfs-binary",
                &self.imp().cryfs_custom_binary_expander_row.get(),
                "expanded",
            )
            .build();

        self.imp()
            .settings
            .bind(
                "use-custom-cryfs-binary",
                &self.imp().cryfs_custom_binary_expander_row.get(),
                "enable-expansion",
            )
            .flags(SettingsBindFlags::GET)
            .build();

        self.imp()
            .settings
            .bind(
                "custom-cryfs-binary-path",
                &self.imp().cryfs_custom_binary_entry_row.get(),
                "text",
            )
            .build();

        self.imp()
            .settings
            .bind(
                "use-custom-gocryptfs-binary",
                &self.imp().gocryptfs_custom_binary_expander_row.get(),
                "expanded",
            )
            .build();

        self.imp()
            .settings
            .bind(
                "use-custom-gocryptfs-binary",
                &self.imp().gocryptfs_custom_binary_expander_row.get(),
                "enable-expansion",
            )
            .flags(SettingsBindFlags::GET)
            .build();

        self.imp()
            .settings
            .bind(
                "custom-gocryptfs-binary-path",
                &self.imp().gocryptfs_custom_binary_entry_row.get(),
                "text",
            )
            .build();
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
