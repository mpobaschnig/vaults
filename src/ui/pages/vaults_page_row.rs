// vaults_page_row.rs
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

use adw::{prelude::ActionRowExt, prelude::PreferencesRowExt, subclass::prelude::*};
use gettextrs::gettext;
use glib::once_cell::sync::Lazy;
use glib::{clone, subclass};
use gtk::gio::Mount;
use gtk::gio::VolumeMonitor;
use gtk::glib::subclass::Signal;
use gtk::glib::{self, closure_local};
use gtk::prelude::*;
use gtk::CompositeTemplate;
use std::cell::RefCell;
use std::process::Command;

use super::{VaultsPageRowPasswordPromptWindow, VaultsPageRowSettingsWindow};
use crate::{
    backend::{Backend, BackendError},
    vault::*,
    VApplication,
};

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/io/github/mpobaschnig/Vaults/vaults_page_row.ui")]
    pub struct VaultsPageRow {
        #[template_child]
        pub vaults_page_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub open_folder_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub locker_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub settings_button: TemplateChild<gtk::Button>,

        pub spinner: RefCell<gtk::Spinner>,

        pub config: RefCell<Option<VaultConfig>>,

        pub volume_monitor: RefCell<VolumeMonitor>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VaultsPageRow {
        const NAME: &'static str = "VaultsPageRow";
        type ParentType = gtk::ListBoxRow;
        type Type = super::VaultsPageRow;

        fn new() -> Self {
            Self {
                vaults_page_row: TemplateChild::default(),
                open_folder_button: TemplateChild::default(),
                locker_button: TemplateChild::default(),
                settings_button: TemplateChild::default(),
                config: RefCell::new(None),
                spinner: RefCell::new(gtk::Spinner::new()),
                volume_monitor: RefCell::new(VolumeMonitor::get()),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for VaultsPageRow {
        fn constructed(&self) {
            let obj = self.obj();
            self.parent_constructed();

            obj.setup_connect_handlers();

            self.open_folder_button.set_visible(false);
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("remove").build(),
                    Signal::builder("save").build(),
                ]
            });
            SIGNALS.as_ref()
        }
    }
    impl WidgetImpl for VaultsPageRow {}
    impl ListBoxRowImpl for VaultsPageRow {}
}

glib::wrapper! {
    pub struct VaultsPageRow(ObjectSubclass<imp::VaultsPageRow>)
        @extends gtk::Widget, gtk::ListBoxRow;
}

impl VaultsPageRow {
    pub fn connect_remove<F: Fn() + 'static>(&self, callback: F) -> glib::SignalHandlerId {
        self.connect_local("remove", false, move |_| {
            callback();
            None
        })
    }

    pub fn connect_save<F: Fn() + 'static>(&self, callback: F) -> glib::SignalHandlerId {
        self.connect_local("save", false, move |_| {
            callback();
            None
        })
    }

    pub fn new(vault: Vault) -> Self {
        let object: Self = glib::Object::new();

        match (vault.get_name(), vault.get_config()) {
            (Some(name), Some(config)) => {
                object.imp().vaults_page_row.set_title(&name);
                object.imp().config.replace(Some(config));
            }
            (_, _) => {
                log::error!("Vault(s) not initialised!");
            }
        }

        if vault.is_mounted() {
            object.set_vault_row_state_opened();
        }

        if !vault.is_backend_available() {
            object.set_vault_row_state_backend_unavailable();
        }

        object
    }

    pub fn setup_connect_handlers(&self) {
        self.imp()
            .open_folder_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.open_folder_button_clicked();
            }));

        self.imp()
            .locker_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.locker_button_clicked();
            }));

        self.imp()
            .settings_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.settings_button_clicked();
            }));

        self.imp().volume_monitor.borrow().connect_mount_added(
            clone!(@weak self as obj=> move |_, m| {
                obj.mount_added_triggered(&m);
            }),
        );

        self.imp().volume_monitor.borrow().connect_mount_removed(
            clone!(@weak self as obj=> move |_, m| {
                obj.mount_removed_triggered(&m);
            }),
        );
    }

    fn open_folder_button_clicked(&self) {
        let output_res = Command::new("xdg-open")
            .arg(&self.imp().config.borrow().as_ref().unwrap().mount_directory)
            .output();

        if let Err(e) = output_res {
            log::error!("Failed to open folder: {}", e);
        }
    }

    fn locker_button_clicked_is_mounted(&self, vault: Vault) {
        if !self.imp().open_folder_button.is_visible() {
            self.set_vault_row_state_opened();
            return;
        }

        if self.imp().spinner.borrow().is_spinning() {
            return;
        }

        self.imp().open_folder_button.set_sensitive(false);

        *self.imp().spinner.borrow_mut() = gtk::Spinner::new();
        let spinner = self.imp().spinner.borrow().clone();
        self.imp().locker_button.set_child(Some(&spinner));

        spinner.start();

        enum Message {
            Finished,
            Error(BackendError),
        }

        let (sender, receiver) = glib::MainContext::channel(glib::Priority::DEFAULT);
        let vault_config = vault.get_config().clone().unwrap();
        std::thread::spawn(move || match Backend::close(&vault_config) {
            Ok(_) => {
                let _ = sender.send(Message::Finished);
            }
            Err(e) => {
                let _ = sender.send(Message::Error(e));
            }
        });

        let locker_button = self.imp().locker_button.clone();
        let open_folder_button = self.imp().open_folder_button.clone();
        let settings_button = self.imp().settings_button.clone();
        let vaults_page_row = self.imp().vaults_page_row.clone();
        receiver.attach(None, move |message| {
            match message {
                Message::Finished => {
                    locker_button.set_icon_name(&"changes-prevent-symbolic");
                    locker_button.set_tooltip_text(Some(&gettext("Open Vault")));
                    open_folder_button.set_visible(false);
                    open_folder_button.set_sensitive(false);
                    settings_button.set_sensitive(true);
                }
                Message::Error(e) => {
                    log::error!("Error closing vault: {}", &e);

                    locker_button.set_icon_name(&"changes-allow-symbolic");
                    locker_button.set_tooltip_text(Some(&gettext("Close Vault")));
                    open_folder_button.set_visible(true);
                    open_folder_button.set_sensitive(true);
                    settings_button.set_sensitive(false);

                    let vault_name = vaults_page_row.title().to_string();
                    gtk::glib::MainContext::default().spawn_local(async move {
                        let window = gtk::gio::Application::default()
                            .unwrap()
                            .downcast_ref::<VApplication>()
                            .unwrap()
                            .active_window()
                            .unwrap()
                            .clone();
                        let info_dialog = gtk::AlertDialog::builder()
                            .message(&vault_name)
                            .detail(&format!("{}", e))
                            .modal(true)
                            .build();

                        info_dialog.show(Some(&window));
                    });
                }
            }

            spinner.stop();

            glib::ControlFlow::Continue
        });
    }

    fn locker_button_clicked_is_not_mounted(&self, vault: Vault) {
        if self.imp().open_folder_button.is_visible() {
            self.set_vault_row_state_closed();
            return;
        }

        if self.imp().spinner.borrow().is_spinning() {
            return;
        }

        let dialog = VaultsPageRowPasswordPromptWindow::new();
        dialog.set_name(&vault.get_name().unwrap());
        dialog.connect_closure(
            "unlock",
            false,
            closure_local!(@strong self as obj => move |dialog: VaultsPageRowPasswordPromptWindow| {
                let password = dialog.get_password();

                dialog.close();

                obj.imp().settings_button.set_sensitive(false);
                obj.imp().open_folder_button.set_sensitive(false);

                *obj.imp().spinner.borrow_mut() = gtk::Spinner::new();
                let spinner = obj.imp().spinner.borrow().clone();
                obj.imp().locker_button.set_child(Some(&spinner));

                spinner.start();

                enum Message {
                    Finished,
                    Error(BackendError),
                }

                let (sender, receiver) = glib::MainContext::channel(glib::Priority::DEFAULT);
                let vault_config = vault.get_config().clone().unwrap();
                std::thread::spawn(move || match Backend::open(&vault_config, password) {
                    Ok(_) => {
                        let _ = sender.send(Message::Finished);
                    }
                    Err(e) => {
                        let _ = sender.send(Message::Error(e));
                    }
                });

                let locker_button = obj.imp().locker_button.clone();
                let open_folder_button = obj.imp().open_folder_button.clone();
                let settings_button = obj.imp().settings_button.clone();
                let vaults_page_row = obj.imp().vaults_page_row.clone();
                receiver.attach(None, move |message| {
                    match message {
                        Message::Finished => {
                            locker_button.set_icon_name(&"changes-allow-symbolic");
                            locker_button.set_tooltip_text(Some(&gettext("Close Vault")));
                            open_folder_button.set_visible(true);
                            open_folder_button.set_sensitive(true);
                            settings_button.set_sensitive(false);
                        }
                        Message::Error(e) => {
                            log::error!("Error opening vault: {}", &e);

                            locker_button.set_icon_name(&"changes-prevent-symbolic");
                            locker_button.set_tooltip_text(Some(&gettext("Open Vault")));
                            open_folder_button.set_visible(false);
                            open_folder_button.set_sensitive(false);
                            settings_button.set_sensitive(true);

                            let vault_name = vaults_page_row.title().to_string();
                            gtk::glib::MainContext::default().spawn_local(async move {
                                let window = gtk::gio::Application::default()
                                    .unwrap()
                                    .downcast_ref::<VApplication>()
                                    .unwrap()
                                    .active_window()
                                    .unwrap()
                                    .clone();

                                let info_dialog = gtk::AlertDialog::builder()
                                    .modal(true)
                                    .message(&vault_name)
                                    .detail(&format!("{}", e))
                                    .build();

                                info_dialog.show(Some(&window));
                            });
                        }
                    }

                    spinner.stop();

                    glib::ControlFlow::Continue
                });
            }),
        );

        dialog.set_visible(true);
    }

    fn locker_button_clicked(&self) {
        let vault = self.get_vault();

        if !vault.is_backend_available() {
            self.set_vault_row_state_backend_unavailable();
            return;
        } else {
            self.set_vault_row_state_backend_available();
        }

        if self.is_mounted() {
            self.locker_button_clicked_is_mounted(vault);
        } else {
            self.locker_button_clicked_is_not_mounted(vault);
        }
    }

    fn settings_button_clicked(&self) {
        let dialog = VaultsPageRowSettingsWindow::new(self.get_vault());

        dialog.connect_closure(
            "save",
            false,
            closure_local!(@strong self as obj => move |dialog: VaultsPageRowSettingsWindow| {
                obj.emit_by_name::<()>("save", &[]);

                let vault = &dialog.get_vault();
                if !vault.is_backend_available() {
                    obj.set_vault_row_state_backend_unavailable();
                } else {
                    obj.set_vault_row_state_backend_available();
                }

                obj.set_vault(dialog.get_vault());
            }),
        );

        dialog.connect_closure(
            "remove",
            false,
            closure_local!(@strong self as obj => move |dialog: VaultsPageRowSettingsWindow| {
                obj.emit_by_name::<()>("remove", &[]);
                dialog.close();
            }),
        );

        dialog.present();
    }

    pub fn get_vault(&self) -> Vault {
        let name = self.imp().vaults_page_row.title();
        let config = self.imp().config.borrow().clone();
        match config {
            Some(config) => Vault::new(
                name.to_string(),
                config.backend,
                config.encrypted_data_directory,
                config.mount_directory,
            ),
            _ => {
                log::error!("Vault not initialised!");
                Vault::new_none()
            }
        }
    }

    pub fn set_vault(&self, vault: Vault) {
        let name = vault.get_name();
        let config = vault.get_config();
        match (name, config) {
            (Some(name), Some(config)) => {
                self.imp().vaults_page_row.set_title(&name);
                self.imp().config.replace(Some(config));
            }
            (_, _) => {
                log::error!("Vault not initialised!");
            }
        }
    }

    pub fn get_name(&self) -> String {
        self.imp().vaults_page_row.title().to_string()
    }

    fn is_mounted(&self) -> bool {
        if self.get_vault().is_mounted() {
            true
        } else {
            false
        }
    }

    fn set_vault_row_state_opened(&self) {
        self.imp()
            .locker_button
            .set_icon_name(&"changes-allow-symbolic");
        self.imp()
            .locker_button
            .set_tooltip_text(Some(&gettext("Close Vault")));
        self.imp().open_folder_button.set_visible(true);
        self.imp().open_folder_button.set_sensitive(true);
        self.imp().settings_button.set_sensitive(false);
    }

    fn set_vault_row_state_closed(&self) {
        self.imp()
            .locker_button
            .set_icon_name(&"changes-prevent-symbolic");
        self.imp()
            .locker_button
            .set_tooltip_text(Some(&gettext("Open Vault")));
        self.imp().open_folder_button.set_visible(false);
        self.imp().open_folder_button.set_sensitive(true);
        self.imp().settings_button.set_sensitive(true);
    }

    fn set_vault_row_state_backend_unavailable(&self) {
        self.imp()
            .vaults_page_row
            .set_subtitle(&gettext("Backend is not installed."));
        self.imp().locker_button.set_sensitive(false);
    }

    fn set_vault_row_state_backend_available(&self) {
        self.imp().vaults_page_row.set_subtitle("");
        self.imp().locker_button.set_sensitive(true);
    }

    fn mount_added_triggered(&self, mount: &Mount) {
        let config_mount_directory = self.imp().config.borrow().clone().unwrap().mount_directory;

        let config_mount_directory_path = std::path::Path::new(&config_mount_directory);

        let config_mount_directory_file_name = config_mount_directory_path.file_name();

        match config_mount_directory_file_name {
            Some(config_mount_directory_file_name) => {
                match config_mount_directory_file_name.to_str() {
                    Some(file_name) => {
                        if mount.name() == file_name {
                            self.set_vault_row_state_opened();
                        }
                    }
                    None => {
                        log::debug!("Could not get mount directory path");
                    }
                }
            }
            None => {
                log::debug!("Could not get config mount directory file name");
            }
        }
    }

    fn mount_removed_triggered(&self, mount: &Mount) {
        let config_mount_directory = self.imp().config.borrow().clone().unwrap().mount_directory;

        let config_mount_directory_path = std::path::Path::new(&config_mount_directory);

        let config_mount_directory_file_name = config_mount_directory_path.file_name();

        match config_mount_directory_file_name {
            Some(config_mount_directory_file_name) => {
                match config_mount_directory_file_name.to_str() {
                    Some(file_name) => {
                        if mount.name() == file_name {
                            self.set_vault_row_state_closed();
                        }
                    }
                    None => {
                        log::debug!("Could not get mount directory path");
                    }
                }
            }
            None => {
                log::debug!("Could not get config mount directory file name");
            }
        }
    }
}
