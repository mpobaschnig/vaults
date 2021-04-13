// window.rs
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

use crate::config::{APP_ID, PROFILE};
use crate::ui::pages::*;
use crate::ui::{AddNewVaultDialog, ImportVaultDialog};
use crate::user_config;
use crate::{application::VApplication, user_config::VAULTS};

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
    #[template(resource = "/com/gitlab/mpobaschnig/Vaults/window.ui")]
    pub struct ApplicationWindow {
        #[template_child]
        pub window_leaflet: TemplateChild<adw::Leaflet>,
        #[template_child]
        pub start_page: TemplateChild<VStartPage>,
        #[template_child]
        pub vaults_page: TemplateChild<VVaultsPage>,

        #[template_child]
        pub headerbar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub refresh_button: TemplateChild<gtk::Button>,

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
                refresh_button: TemplateChild::default(),
                settings: gio::Settings::new(APP_ID),
                view: RefCell::new(VView::Start),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        // You must call `Widget`'s `init_template()` within `instance_init()`.
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ApplicationWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            if PROFILE == "Devel" {
                obj.get_style_context().add_class("devel");
            }

            self.vaults_page.init();

            obj.setup_connect_handlers();
            obj.setup_gactions();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpec::enum_(
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

        fn get_property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.get_name() {
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
            match pspec.get_name() {
                "view" => {
                    let view = value.get().unwrap();
                    *self.view.borrow_mut() = view.unwrap();
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
        @extends gtk::Widget, gtk::Window, adw::ApplicationWindow, @implements gio::ActionMap, gio::ActionGroup;
}

impl ApplicationWindow {
    pub fn new(app: &VApplication) -> Self {
        let window: Self = glib::Object::new(&[]).expect("Failed to create ApplicationWindow");
        window.set_application(Some(app));

        // Set icons for shell
        gtk::Window::set_default_icon_name(APP_ID);

        match VAULTS.lock() {
            Ok(v) => {
                if !v.vault.is_empty() {
                    window.set_view(VView::Vaults);
                }
            }
            Err(e) => {
                log::error!("Failed to aquire mutex lock of VAULTS: {}", e);
            }
        }

        window
    }

    fn setup_connect_handlers(&self) {
        let self_ = imp::ApplicationWindow::from_instance(self);

        self_
            .refresh_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.refresh_button_clicked();
            }));
    }

    fn setup_gactions(&self) {
        action!(
            self,
            "add_new_vault",
            clone!(@weak self as win => move |_, _| {
                win.add_new_vault_clicked();
            })
        );

        action!(
            self,
            "import_vault",
            clone!(@weak self as win => move |_, _| {
                win.import_vault_clicked();
            })
        );
    }

    fn add_new_vault_clicked(&self) {
        let dialog = AddNewVaultDialog::new();
        dialog.connect_response(|dialog, id| match id {
            gtk::ResponseType::Ok => {
                let vault = dialog.get_vault();

                match user_config::VAULTS.lock() {
                    Ok(mut v) => {
                        v.vault.push(vault);
                    }
                    Err(e) => {
                        log::error!("Failed to aquire mutex lock of USER_DATA_DIRECTORY: {}", e);
                    }
                }

                user_config::write();

                dialog.destroy();
            }
            _ => {
                dialog.destroy();
            }
        });

        dialog.show();
    }

    fn import_vault_clicked(&self) {
        let dialog = ImportVaultDialog::new();
        dialog.connect_response(|dialog, id| match id {
            gtk::ResponseType::Ok => {
                let vault = dialog.get_vault();

                match user_config::VAULTS.lock() {
                    Ok(mut v) => {
                        v.vault.push(vault);
                    }
                    Err(e) => {
                        log::error!("Failed to aquire mutex lock of USER_DATA_DIRECTORY: {}", e);
                    }
                }

                user_config::write();

                dialog.destroy();
            }
            _ => {
                dialog.destroy();
            }
        });

        dialog.show();
    }

    fn refresh_button_clicked(&self) {
        self.update_view();
    }

    fn update_view(&self) {
        let self_ = imp::ApplicationWindow::from_instance(self);
        let view = *self_.view.borrow();
        debug!("Set view to {:?}", view);

        // Show requested view / page
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
