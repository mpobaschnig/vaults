// window.rs
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

use crate::config::{APP_ID, PROFILE};
use crate::ui::pages::*;
use crate::ui::{AddNewVaultDialog, ImportVaultDialog};
use crate::{
    application::VApplication, backend, backend::Backend, user_config_manager::UserConfigManager,
};

use adw::subclass::prelude::*;
use glib::{clone, GEnum, ParamSpec, ToValue};
use gtk::subclass::prelude::*;
use gtk::{self, prelude::*};
use gtk::{gio, glib, CompositeTemplate};
use gtk_macros::action;
use log::*;
use once_cell::sync::Lazy;
use std::cell::RefCell;

#[derive(Copy, Debug, Clone, PartialEq, GEnum)]
#[repr(u32)]
#[genum(type_name = "VVView")]
pub enum VView {
    Start,
    Vaults,
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
        pub start_page: TemplateChild<VStartPage>,
        #[template_child]
        pub vaults_page: TemplateChild<VVaultsPage>,

        #[template_child]
        pub headerbar: TemplateChild<adw::HeaderBar>,

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
                start_page: TemplateChild::default(),
                vaults_page: TemplateChild::default(),
                headerbar: TemplateChild::default(),
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

            self.vaults_page.init();

            obj.setup_gactions();
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
                    *self.view.borrow_mut() = value.get().unwrap();
                    obj.update_view();
                }
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for ApplicationWindow {}
    impl WindowImpl for ApplicationWindow {}

    impl ApplicationWindowImpl for ApplicationWindow {}
    impl AdwApplicationWindowImpl for ApplicationWindow {}
}

glib::wrapper! {
    pub struct ApplicationWindow(ObjectSubclass<imp::ApplicationWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow, @implements gio::ActionMap, gio::ActionGroup;
}

impl ApplicationWindow {
    pub fn new(app: &VApplication) -> Self {
        let object: Self = glib::Object::new(&[]).expect("Failed to create ApplicationWindow");
        object.set_application(Some(app));

        gtk::Window::set_default_icon_name(APP_ID);

        if !UserConfigManager::instance().get_map().is_empty() {
            object.set_view(VView::Vaults);
        }

        let self_ = imp::ApplicationWindow::from_instance(&object);
        self_
            .vaults_page
            .connect_refresh(clone!(@weak object => move || {
                if UserConfigManager::instance().get_map().is_empty() {
                    object.set_view(VView::Start);
                }
            }));
        
        let builder = gtk::Builder::from_resource("/io/github/mpobaschnig/Vaults/shortcuts.ui");
        gtk_macros::get_widget!(builder, gtk::ShortcutsWindow, shortcuts);
        object.set_help_overlay(Some(&shortcuts));

        object
    }

    fn setup_gactions(&self) {
        action!(
            self,
            "refresh",
            clone!(@weak self as obj => move |_, _| {
                obj.refresh_clicked();
            })
        );

        action!(
            self,
            "add_new_vault",
            clone!(@weak self as obj => move |_, _| {
                obj.add_new_vault_clicked();
            })
        );

        action!(
            self,
            "import_vault",
            clone!(@weak self as obj => move |_, _| {
                obj.import_vault_clicked();
            })
        );
    }

    fn add_new_vault_clicked(&self) {
        backend::probe_backends();

        let dialog = AddNewVaultDialog::new();
        dialog.connect_response(clone!(@weak self as obj => move |dialog, id|
            if id == gtk::ResponseType::Ok {
                let vault = dialog.get_vault();
                let password = dialog.get_password();
                match Backend::init(&vault.get_config().unwrap(), password) {
                    Ok(_) => {
                        UserConfigManager::instance().add_vault(vault);
                        obj.set_view(VView::Vaults);
                    }
                    Err(e) => {
                        log::error!("Could not init vault: {}", e);
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
            }

            dialog.destroy();
        ));

        dialog.show();
    }

    fn import_vault_clicked(&self) {
        let dialog = ImportVaultDialog::new();
        dialog.connect_response(clone!(@weak self as obj => move |dialog, id| match id {
            gtk::ResponseType::Ok => {
                let vault = dialog.get_vault();

                UserConfigManager::instance().add_vault(vault);

                obj.set_view(VView::Vaults);

                dialog.destroy();
            }
            _ => {
                dialog.destroy();
            }
        }));

        dialog.show();
    }

    fn refresh_clicked(&self) {
        let self_ = imp::ApplicationWindow::from_instance(self);

        self_.vaults_page.clear();

        backend::probe_backends();

        UserConfigManager::instance().read_config();

        self_.vaults_page.init();

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
        }
    }

    pub fn set_view(&self, view: VView) {
        self.set_property("view", &view).unwrap()
    }
}
