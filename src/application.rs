// application.rs
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

use crate::backend::Backend;
use crate::config;
use crate::ui::pages::VaultsPageRowPasswordPromptWindow;
use crate::ui::ApplicationWindow;
use crate::ui::VaultsSettingsWindow;
use crate::user_config_manager::UserConfigManager;

use adw::prelude::AdwDialogExt;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use gio::ApplicationFlags;
use glib::clone;
use gtk::glib::closure_local;
use gtk::glib::VariantTy;
use gtk::prelude::*;
use gtk::{gio, glib};
use gtk_macros::action;
use log::{debug, info};
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, Default, PartialEq)]
    enum OnlyPromptType {
        #[default]
        None = 0,
        Open = 1,
        Close = 2,
    }

    #[derive(Debug, Default)]
    pub struct VApplication {
        pub window: RefCell<Option<ApplicationWindow>>,

        only_prompt_type: RefCell<OnlyPromptType>,
        only_pompt_vault: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VApplication {
        const NAME: &'static str = "VApplication";
        type Type = super::VApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for VApplication {}

    impl gio::subclass::prelude::ApplicationImpl for VApplication {
        fn activate(&self) {
            debug!("GtkApplication<VApplication>::activate");

            if let Some(ref window) = *self.window.borrow() {
                window.present();
                return;
            }

            let app = self.obj();

            app.setup_accels();
            app.setup_gactions();
            app.set_resource_base_path(Some("/io/github/mpobaschnig/Vaults/"));

            match *self.only_prompt_type.borrow() {
                OnlyPromptType::None => {
                    let window = ApplicationWindow::new(&app);
                    window.present();
                    self.window.replace(Some(window));
                }
                OnlyPromptType::Open => {
                    let window = ApplicationWindow::new(&app);
                    self.window.replace(Some(window));

                    let map = UserConfigManager::instance().get_map();
                    let vault_config = map.get(&*self.only_pompt_vault.borrow());

                    match vault_config {
                        Some(vault_config) => {
                            let dialog = VaultsPageRowPasswordPromptWindow::new();
                            dialog.set_name(&self.only_pompt_vault.borrow());
                            dialog.connect_closure(
                                "unlock",
                                false,
                                closure_local!(
                                    #[strong(rename_to = vc)]
                                    vault_config,
                                    #[strong(rename_to = a)]
                                    app,
                                    move |dialog: VaultsPageRowPasswordPromptWindow| {
                                        let password = dialog.get_password();
                                        let result = Backend::open(&vc, password);
                                        match result {
                                            Ok(_) => log::info!("Opened vault successfully."),
                                            Err(e) => log::error!("{e}"),
                                        }
                                        a.quit();
                                    }
                                ),
                            );

                            AdwDialogExt::present(&dialog, Option::<&gtk::Widget>::None);
                        }
                        None => {
                            log::error!(
                                "Vault {} does not exist.",
                                *self.only_pompt_vault.borrow()
                            );
                            app.quit();
                        }
                    }
                }
                OnlyPromptType::Close => {
                    let window = ApplicationWindow::new(&app);
                    self.window.replace(Some(window));

                    let map = UserConfigManager::instance().get_map();
                    let vault_config = map.get(&*self.only_pompt_vault.borrow());

                    match vault_config {
                        Some(vault_config) => {
                            let result = Backend::close(vault_config);
                            match result {
                                Ok(_) => log::info!("Closed vault successfully."),
                                Err(e) => log::error!("{e}"),
                            }
                        }
                        None => {
                            log::error!(
                                "Vault {} does not exist.",
                                *self.only_pompt_vault.borrow()
                            );
                        }
                    }
                    app.quit();
                }
            }
        }

        fn startup(&self) {
            debug!("GtkApplication<VApplication>::startup");
            self.parent_startup();
        }

        fn handle_local_options(&self, options: &glib::VariantDict) -> glib::ExitCode {
            if let Some(vault_name) = options.lookup_value("open", Some(VariantTy::STRING)) {
                *self.only_prompt_type.borrow_mut() = OnlyPromptType::Open;
                *self.only_pompt_vault.borrow_mut() = vault_name.get::<String>().unwrap();
            }

            if let Some(vault_name) = options.lookup_value("close", Some(VariantTy::STRING)) {
                if *self.only_prompt_type.borrow() != OnlyPromptType::None {
                    log::error!("{}", gettext("Cannot open and close at the same time."));
                    return glib::ExitCode::from(2);
                }

                *self.only_prompt_type.borrow_mut() = OnlyPromptType::Close;
                *self.only_pompt_vault.borrow_mut() = vault_name.get::<String>().unwrap();
            }

            glib::ExitCode::from(-1)
        }
    }

    impl GtkApplicationImpl for VApplication {}
    impl AdwApplicationImpl for VApplication {}
}

glib::wrapper! {
    pub struct VApplication(ObjectSubclass<imp::VApplication>)
    @extends gio::Application, gtk::ApplicationWindow, gtk::Application, adw::Application, @implements gio::ActionMap, gio::ActionGroup;
}

impl Default for VApplication {
    fn default() -> Self {
        Self::new()
    }
}

impl VApplication {
    pub fn new() -> Self {
        let object: Self = glib::Object::new();
        object.set_property("application-id", config::APP_ID);
        object.set_property("flags", ApplicationFlags::FLAGS_NONE);
        object.set_property("register-session", true);

        object.add_main_option(
            "open",
            glib::Char::from(b'o'),
            glib::OptionFlags::IN_MAIN,
            glib::OptionArg::String,
            "Open given vault",
            None,
        );
        object.add_main_option(
            "close",
            glib::Char::from(b'c'),
            glib::OptionFlags::IN_MAIN,
            glib::OptionArg::String,
            "Close given vault",
            None,
        );

        object
    }

    fn setup_gactions(&self) {
        action!(
            self,
            "preferences",
            clone!(
                #[strong(rename_to = obj)]
                self,
                move |_, _| {
                    obj.show_preferences();
                }
            )
        );

        action!(
            self,
            "about",
            clone!(
                #[weak(rename_to = obj)]
                self,
                move |_, _| {
                    obj.show_about_dialog();
                }
            )
        );

        action!(
            self,
            "quit",
            clone!(
                #[weak(rename_to = obj)]
                self,
                move |_, _| {
                    obj.quit();
                }
            )
        );
    }

    fn setup_accels(&self) {
        self.set_accels_for_action("win.add_new_vault", &["<primary>a"]);
        self.set_accels_for_action("win.import_vault", &["<primary>i"]);
        self.set_accels_for_action("win.search", &["<primary>f"]);
        self.set_accels_for_action("win.escape", &["Escape"]);
        self.set_accels_for_action("win.refresh", &["<primary>r"]);

        self.set_accels_for_action("app.preferences", &["<primary>comma"]);
        self.set_accels_for_action("win.show-help-overlay", &["<primary>question"]);
        self.set_accels_for_action("app.quit", &["<primary>q"]);
    }

    fn show_preferences(&self) {
        let preferences = VaultsSettingsWindow::new();
        AdwDialogExt::present(&preferences, Some(&self.active_window().unwrap()));
    }

    fn show_about_dialog(&self) {
        let about_window = adw::AboutDialog::new();

        about_window.set_application_icon(config::APP_ID);
        about_window.set_application_name("Vaults");
        about_window.set_artists(&["Martin Pobaschnig", "Jacson Hilgert"]);
        about_window.set_copyright("© 2022 Martin Pobaschnig");
        // Translators: Replace "translator-credits" with your names, one name per line
        about_window.set_translator_credits(&gettext("translator-credits"));
        about_window.set_developer_name("Martin Pobschnig");
        about_window.set_issue_url("https://github.com/mpobaschnig/Vaults/issues");
        about_window.set_license_type(gtk::License::Gpl30);
        about_window.set_support_url("https://github.com/mpobaschnig/Vaults/discussions");
        about_window.set_version(config::VERSION);
        about_window.set_website("https://github.com/mpobaschnig/Vaults");

        about_window.present(Some(&self.active_window().unwrap()));
    }

    pub fn run(&self) {
        info!("Vaults ({})", config::APP_ID);
        info!("Version: {} ({})", config::VERSION, config::PROFILE);
        info!("Datadir: {}", config::PKGDATADIR);

        ApplicationExtManual::run(self);
    }
}
