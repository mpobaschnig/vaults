// window.rs
//
// Copyright 2021 Martin Pobaschnig <martin.pobaschnig@protonmail.com>
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

use crate::config::{APP_ID, PROFILE};
use crate::password_manager::PasswordManager;
use crate::ui::pages::*;
use crate::{application::VApplication, backend::*, user_config_manager::UserConfigManager};
use adw::subclass::prelude::*;
use gettextrs::gettext;
use glib::{clone, GEnum, ParamSpec, ToValue};
use gtk::glib::subclass::Signal;
use gtk::subclass::prelude::*;
use gtk::{self, prelude::*};
use gtk::{gio, glib, CompositeTemplate};
use log::*;
use once_cell::sync::Lazy;
use std::cell::RefCell;

#[derive(Copy, Debug, Clone, PartialEq, GEnum)]
#[repr(u32)]
#[genum(type_name = "VVView")]
pub enum VView {
    Add,
    SettingsPage,
    Start,
    Vaults,
    UnlockVault,
}

impl Default for VView {
    fn default() -> Self {
        VView::Start
    }
}

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/io/github/mpobaschnig/Vaults/window.ui")]
    pub struct ApplicationWindow {
        #[template_child]
        pub window_leaflet: TemplateChild<adw::Leaflet>,
        #[template_child]
        pub add_page: TemplateChild<AddPage>,
        #[template_child]
        pub settings_page: TemplateChild<SettingsPage>,
        #[template_child]
        pub start_page: TemplateChild<VStartPage>,
        #[template_child]
        pub vaults_page: TemplateChild<VVaultsPage>,
        #[template_child]
        pub unlock_vault_page: TemplateChild<UnlockVaultPage>,

        #[template_child]
        pub headerbar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub add_button: TemplateChild<gtk::Button>,

        pub spinner: RefCell<gtk::Spinner>,

        pub settings: gio::Settings,

        pub view: RefCell<VView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ApplicationWindow {
        const NAME: &'static str = "ApplicationWindow";
        type Type = super::ApplicationWindow;
        type ParentType = adw::ApplicationWindow;

        fn new() -> Self {
            Self {
                window_leaflet: TemplateChild::default(),
                add_page: TemplateChild::default(),
                settings_page: TemplateChild::default(),
                start_page: TemplateChild::default(),
                vaults_page: TemplateChild::default(),
                unlock_vault_page: TemplateChild::default(),
                headerbar: TemplateChild::default(),
                add_button: TemplateChild::default(),
                spinner: RefCell::new(gtk::Spinner::new()),
                settings: gio::Settings::new(APP_ID),
                view: RefCell::new(VView::Start),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ApplicationWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            if PROFILE == "Devel" {
                obj.style_context().add_class("devel");
            }

            self.add_page.init();
            self.vaults_page.init();

            self.add_page.connect_add(clone!(@weak obj => move || {
                let self_ = imp::ApplicationWindow::from_instance(&obj);

                let vault = UserConfigManager::instance().get_current_vault().unwrap();
                let password = PasswordManager::instance().get_current_password().unwrap();
                PasswordManager::instance().clear_current_pssword();
                let vault_config = vault.get_config().clone().unwrap();

                *self_.spinner.borrow_mut() = gtk::Spinner::new();
                let spinner = self_.spinner.borrow().clone();
                self_.add_button.set_child(Some(&spinner));

                spinner.start();

                enum Message {
                    Finished,
                    Error(BackendError),
                }

                let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
                std::thread::spawn(move || match Backend::init(&vault_config, password) {
                    Ok(_) => {
                        let _ = sender.send(Message::Finished);
                    }
                    Err(e) => {
                        let _ = sender.send(Message::Error(e));
                    }
                });

                let add_button = self_.add_button.clone();
                let add_page = self_.add_page.clone();
                receiver.attach(None, move |message| {
                    let vault = UserConfigManager::instance().get_current_vault().unwrap();
                    match message {
                        Message::Finished => {
                            add_button.set_icon_name(&"list-add-symbolic");
                            UserConfigManager::instance().add_vault(vault);
                            obj.set_view(VView::Vaults);
                        }
                        Message::Error(e) => {
                            add_button.set_icon_name(&"list-add-symbolic");
                            gtk::glib::MainContext::default().spawn_local(async move {
                                let window = gtk::gio::Application::default()
                                    .unwrap()
                                    .downcast_ref::<VApplication>()
                                    .unwrap()
                                    .active_window()
                                    .unwrap()
                                    .clone();
                                let info_dialog = gtk::MessageDialogBuilder::new()
                                    .message_type(gtk::MessageType::Error)
                                    .transient_for(&window)
                                    .modal(true)
                                    .buttons(gtk::ButtonsType::Close)
                                    .text(&vault.get_name().unwrap())
                                    .secondary_text(&format!("{}", e))
                                    .build();

                                info_dialog.run_future().await;
                                info_dialog.close();
                            });
                        }
                    }

                    add_button.set_icon_name(&"list-add-symbolic");
                    add_button.set_tooltip_text(Some(&gettext("Add or Import New Vault")));

                    add_page.set_last_page_sensitive(true);

                    spinner.stop();

                    glib::Continue(true)
                });
            }));

            self.add_page.connect_import(clone!(@weak obj => move || {
                let self_ = imp::ApplicationWindow::from_instance(&obj);

                let vault = UserConfigManager::instance().get_current_vault().unwrap();

                UserConfigManager::instance().add_vault(vault);

                obj.set_view(VView::Vaults);

                self_.add_button.set_icon_name(&"list-add-symbolic");
                self_.add_button.set_tooltip_text(Some(&gettext("Add or Import New Vault")));
            }));

            obj.setup_connect_handlers();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpec::new_enum(
                    "view",
                    "View",
                    "View",
                    VView::static_type(),
                    VView::default() as i32,
                    glib::ParamFlags::READWRITE,
                )]
            });

            PROPERTIES.as_ref()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "view" => self.view.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &ParamSpec,
        ) {
            match pspec.name() {
                "view" => {
                    //let view = value.get().unwrap().to_value();
                    *self.view.borrow_mut() = value.get().unwrap();
                    obj.update_view();
                }
                _ => unimplemented!(),
            }
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("unlock", &[], glib::Type::UNIT.into()).build()]);

            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for ApplicationWindow {}
    impl WindowImpl for ApplicationWindow {}

    impl ApplicationWindowImpl for ApplicationWindow {}
    impl AdwApplicationWindowImpl for ApplicationWindow {}
}

glib::wrapper! {
    pub struct ApplicationWindow(ObjectSubclass<imp::ApplicationWindow>)
        @extends gtk::Widget, gtk::Window, adw::ApplicationWindow, @implements gio::ActionMap, gio::ActionGroup;
}

impl ApplicationWindow {
    pub fn connect_unlock<F: Fn() + 'static>(&self, callback: F) -> glib::SignalHandlerId {
        self.connect_local("unlock", false, move |_| {
            callback();
            None
        })
        .unwrap()
    }

    pub fn window_connect_add_import(&self) {}

    pub fn new(app: &VApplication) -> Self {
        let object: Self = glib::Object::new(&[]).expect("Failed to create ApplicationWindow");
        object.set_application(Some(app));

        gtk::Window::set_default_icon_name(APP_ID);

        if !UserConfigManager::instance().get_map().is_empty() {
            object.set_view(VView::Vaults);
        } else {
            object.set_view(VView::Start);
        }

        let self_ = imp::ApplicationWindow::from_instance(&object);
        self_
            .vaults_page
            .connect_refresh(clone!(@weak object => move || {
                if UserConfigManager::instance().get_map().is_empty() {
                    object.set_view(VView::Start);
                }
            }));

        object
    }

    fn setup_connect_handlers(&self) {
        let self_ = imp::ApplicationWindow::from_instance(self);

        self_
            .add_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.add_button_clicked();
            }));
    }

    fn add_button_clicked(&self) {
        let self_ = imp::ApplicationWindow::from_instance(self);
        let view = *self_.view.borrow();

        if self_.spinner.borrow().is_spinning() {
            return;
        }

        match view {
            VView::Add => {
                self.set_standard_window_view();
            }
            VView::UnlockVault => {
                self_.unlock_vault_page.disconnect_all_signals();
                self.set_standard_window_view();
            }
            VView::SettingsPage => {
                self_.settings_page.disconnect_all_signals();
                self.set_standard_window_view();
            }
            _ => {
                self_.add_button.set_icon_name(&"go-previous-symbolic");
                self_.add_button.set_tooltip_text(Some(&gettext("Go Back")));
                self_.add_page.init();
                self.set_view(VView::Add);
            }
        }
    }

    pub fn set_standard_window_view(&self) {
        let self_ = imp::ApplicationWindow::from_instance(self);

        self_.add_button.set_icon_name(&"list-add-symbolic");
        self_
            .add_button
            .set_tooltip_text(Some(&gettext("Add or Import New Vault")));
        if UserConfigManager::instance().get_map().is_empty() {
            self.set_view(VView::Start);
        } else {
            self.set_view(VView::Vaults);
        }
    }

    fn update_view(&self) {
        let self_ = imp::ApplicationWindow::from_instance(self);
        let view = *self_.view.borrow();
        debug!("Set view to {:?}", view);

        match view {
            VView::Add => {
                self_
                    .window_leaflet
                    .set_visible_child(&self_.add_page.get());
            }
            VView::SettingsPage => {
                self_
                    .window_leaflet
                    .set_visible_child(&self_.settings_page.get());
            }
            VView::Start => {
                self_
                    .window_leaflet
                    .set_visible_child(&self_.start_page.get());
            }
            VView::Vaults => {
                self_
                    .window_leaflet
                    .set_visible_child(&self_.vaults_page.get());
            }
            VView::UnlockVault => {
                self_
                    .window_leaflet
                    .set_visible_child(&self_.unlock_vault_page.get());
            }
        }
    }

    pub fn set_view(&self, view: VView) {
        self.set_property("view", &view).unwrap()
    }

    pub fn set_unlock_vault_view(&self) {
        let self_ = imp::ApplicationWindow::from_instance(self);

        self_.add_button.set_icon_name(&"go-previous-symbolic");
        self_.add_button.set_tooltip_text(Some(&gettext("Go Back")));
        self_.unlock_vault_page.init();

        self.set_view(VView::UnlockVault);
    }

    pub fn call_unlock(&self, row: &VaultsPageRow) {
        let self_ = imp::ApplicationWindow::from_instance(self);

        self_.unlock_vault_page.init();
        self_.unlock_vault_page.call_unlock(row);
    }

    pub fn set_settings_page(&self) {
        let self_ = imp::ApplicationWindow::from_instance(self);

        self_.add_button.set_icon_name(&"go-previous-symbolic");
        self_.add_button.set_tooltip_text(Some(&gettext("Go Back")));
        self_.unlock_vault_page.init();

        self.set_view(VView::SettingsPage);
    }

    pub fn call_settings(&self, row: &VaultsPageRow) {
        let self_ = imp::ApplicationWindow::from_instance(self);

        self_.settings_page.init();
        self_.settings_page.call_settings(row);
    }
}
