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

use crate::backend::AVAILABLE_BACKENDS;
use crate::config::{APP_ID, PROFILE};
use crate::ui::pages::*;
use crate::ui::window::glib::GString;
use crate::ui::{AddNewVaultDialog, ImportVaultDialog};
use crate::{
    application::VApplication, backend, backend::Backend, user_config_manager::UserConfigManager,
    vault::*,
};

use adw::subclass::prelude::*;
use gettextrs::gettext;
use glib::clone;
use gtk::gio::ListStore;
use gtk::subclass::prelude::*;
use gtk::{self, prelude::*};
use gtk::{gio, glib, CompositeTemplate};
use gtk_macros::action;

use std::cell::RefCell;

#[derive(PartialEq, Debug)]
pub enum View {
    Search,
    Start,
    Vaults,
}

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/io/github/mpobaschnig/Vaults/window.ui")]
    pub struct ApplicationWindow {
        #[template_child]
        pub window_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub start_page_status_page: TemplateChild<adw::StatusPage>,
        #[template_child]
        pub vaults_list_box: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub search_vaults_list_box: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub headerbar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub title_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub search_entry: TemplateChild<gtk::SearchEntry>,
        #[template_child]
        pub search_toggle_button: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub search_stack: TemplateChild<gtk::Stack>,

        pub list_store: ListStore,
        pub search_list_store: ListStore,

        pub search_results: RefCell<u32>,

        pub settings: gio::Settings,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ApplicationWindow {
        const NAME: &'static str = "ApplicationWindow";
        type Type = super::ApplicationWindow;
        type ParentType = adw::ApplicationWindow;

        fn new() -> Self {
            Self {
                window_stack: TemplateChild::default(),
                start_page_status_page: TemplateChild::default(),
                vaults_list_box: TemplateChild::default(),
                search_vaults_list_box: TemplateChild::default(),
                headerbar: TemplateChild::default(),
                title_stack: TemplateChild::default(),
                search_entry: TemplateChild::default(),
                search_toggle_button: TemplateChild::default(),
                search_stack: TemplateChild::default(),
                list_store: ListStore::new(gtk::Widget::static_type()),
                search_list_store: ListStore::new(gtk::Widget::static_type()),
                search_results: RefCell::new(0),
                settings: gio::Settings::new(APP_ID),
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

            obj.setup_gactions();
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
            object.set_view(View::Vaults);
        } else {
            object.set_view(View::Start);
        }

        object.setup_window();
        object.setup_search_page();
        object.setup_start_page();
        object.setup_vaults_page();

        let builder = gtk::Builder::from_resource("/io/github/mpobaschnig/Vaults/shortcuts.ui");
        gtk_macros::get_widget!(builder, gtk::ShortcutsWindow, shortcuts);
        object.set_help_overlay(Some(&shortcuts));

        object
    }

    fn setup_window(&self) {
        let self_ = imp::ApplicationWindow::from_instance(self);

        self_
            .search_toggle_button
            .connect_toggled(clone!(@weak self as obj => move |button| {
                let self_ = imp::ApplicationWindow::from_instance(&obj);

                if button.is_active() {
                    self_.title_stack.set_visible_child_name("search");
                    self_.search_entry.grab_focus();
                } else {
                    self_.search_entry.set_text("");
                    self_.title_stack.set_visible_child_name("title");
                    obj.refresh_clicked();
                }
            }));
    }

    fn setup_search_page(&self) {
        let self_ = imp::ApplicationWindow::from_instance(self);

        self_
            .search_vaults_list_box
            .bind_model(Some(&self_.search_list_store), |obj| {
                obj.clone().downcast::<gtk::Widget>().unwrap()
            });

        self_.search_stack.set_visible_child_name("start");

        self_
            .search_entry
            .connect_search_changed(clone!(@weak self as obj => move |_| {
                if obj.get_view().unwrap() != "search" {
                    obj.set_view(View::Search);
                }

                obj.search();
            }));
    }

    fn search(&self) {
        let self_ = imp::ApplicationWindow::from_instance(&self);

        let text = self_.search_entry.text();

        *self_.search_results.borrow_mut() = 0;
        let mut found = false;
        let map = UserConfigManager::instance().get_map();
        for (k, v) in &map {
            if k.contains(&text.to_string()) {
                if !found {
                    self_.search_list_store.remove_all();
                    found = true;
                }

                let vault = Vault::new(
                    k.to_owned(),
                    v.backend,
                    v.encrypted_data_directory.to_owned(),
                    v.mount_directory.to_owned(),
                );

                let row = VaultsPageRow::new(vault);
                self.search_row_connect_remove(&row);
                self.row_connect_save(&row);

                self_.search_list_store.insert_sorted(&row, |v1, v2| {
                    let row1 = v1.downcast_ref::<VaultsPageRow>().unwrap();
                    let name1 = row1.get_name();
                    let row2 = v2.downcast_ref::<VaultsPageRow>().unwrap();
                    let name2 = row2.get_name();
                    name1.cmp(&name2)
                });

                *self_.search_results.borrow_mut() += 1;
            }
        }

        if map.is_empty() {
            self_.search_stack.set_visible_child_name("start");
            return;
        }

        if found {
            self_.search_stack.set_visible_child_name("results");
        } else {
            self_.search_stack.set_visible_child_name("no-results");
        }
    }

    fn setup_start_page(&self) {
        let self_ = imp::ApplicationWindow::from_instance(self);

        self_.start_page_status_page.set_icon_name(Some(APP_ID));

        if let Ok(available_backends) = AVAILABLE_BACKENDS.lock() {
            if available_backends.is_empty() {
                self_.start_page_status_page.set_description(Some(&gettext(
                    "No backends available. Please install gocryptfs or CryFS on your system.",
                )));
            }
        }
    }

    fn setup_vaults_page(&self) {
        let self_ = imp::ApplicationWindow::from_instance(self);

        UserConfigManager::instance().connect_add_vault(clone!(@weak self as obj => move || {
            obj.add_vault();
        }));

        UserConfigManager::instance().connect_refresh(
            clone!(@weak self as obj => move |map_is_empty| {
                obj.refresh(map_is_empty);
            }),
        );

        self_
            .vaults_list_box
            .bind_model(Some(&self_.list_store), |obj| {
                obj.clone().downcast::<gtk::Widget>().unwrap()
            });

        self.fill_list_store();
    }

    fn setup_gactions(&self) {
        action!(
            self,
            "search",
            clone!(@weak self as obj => move |_, _| {
                let self_ = imp::ApplicationWindow::from_instance(&obj);
                if self_.search_toggle_button.is_sensitive() {
                    self_.search_toggle_button.set_active(true);
                }
            })
        );

        action!(
            self,
            "escape",
            clone!(@weak self as obj => move |_, _| {
                let self_ = imp::ApplicationWindow::from_instance(&obj);
                self_.search_toggle_button.set_active(false);
            })
        );

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

    fn fill_list_store(&self) {
        let self_ = imp::ApplicationWindow::from_instance(self);

        let map = UserConfigManager::instance().get_map();
        for (k, v) in map.iter() {
            let vault = Vault::new(
                k.to_owned(),
                v.backend,
                v.encrypted_data_directory.to_owned(),
                v.mount_directory.to_owned(),
            );

            let row = VaultsPageRow::new(vault);
            self.row_connect_remove(&row);
            self.row_connect_save(&row);

            self_.list_store.insert_sorted(&row, |v1, v2| {
                let row1 = v1.downcast_ref::<VaultsPageRow>().unwrap();
                let name1 = row1.get_name();
                let row2 = v2.downcast_ref::<VaultsPageRow>().unwrap();
                let name2 = row2.get_name();
                name1.cmp(&name2)
            });
        }
    }

    pub fn search_row_connect_remove(&self, row: &VaultsPageRow) {
        row.connect_remove(clone!(@weak self as obj, @weak row => move || {
            let obj_ = imp::ApplicationWindow::from_instance(&obj);
            let index = obj_.search_list_store.find(&row);
            if let Some(index) = index {
                obj_.search_list_store.remove(index);

                *obj_.search_results.borrow_mut() -= 1;

                if *obj_.search_results.borrow_mut() == 0 {
                    if UserConfigManager::instance().get_map().is_empty() {
                        obj_.search_stack.set_visible_child_name("start");
                        obj_.search_entry.set_text("");
                        obj_.title_stack.set_visible_child_name("title");
                        obj_.search_toggle_button.set_active(false);
                        obj_.search_toggle_button.set_sensitive(false);
                        return;
                    }
                    if obj.get_view().unwrap() != "search" {
                        obj.set_view(View::Search);
                    }
                    obj_.search_stack.set_visible_child_name("no-results");
                }
            } else {
                log::error!("Vault not initialised!");
            }
        }));
    }

    pub fn row_connect_remove(&self, row: &VaultsPageRow) {
        row.connect_remove(clone!(@weak self as obj, @weak row => move || {
            let obj_ = imp::ApplicationWindow::from_instance(&obj);
            let index = obj_.list_store.find(&row);
            if let Some(index) = index {
                obj_.list_store.remove(index);
                if UserConfigManager::instance().get_map().is_empty() {
                    obj_.search_entry.set_text("");
                    obj_.title_stack.set_visible_child_name("title");
                    obj_.search_toggle_button.set_active(false);
                    obj_.search_toggle_button.set_sensitive(false);
                }
            } else {
                log::error!("Vault not initialised!");
            }
        }));
    }

    pub fn row_connect_save(&self, row: &VaultsPageRow) {
        row.connect_save(clone!(@weak self as obj, @weak row as r => move || {
            let vault = UserConfigManager::instance().get_current_vault();
            if let Some(vault) = vault {
                r.set_vault(vault);
            } else {
                log::error!("Vault not initialised!");
            }
        }));
    }

    pub fn add_vault(&self) {
        let self_ = imp::ApplicationWindow::from_instance(self);

        let vault = UserConfigManager::instance().get_current_vault();

        if let Some(vault) = vault {
            let row = VaultsPageRow::new(vault.clone());
            self.row_connect_remove(&row);
            self.row_connect_save(&row);

            self_.list_store.insert_sorted(&row, |v1, v2| {
                let row1 = v1.downcast_ref::<VaultsPageRow>().unwrap();
                let name1 = row1.get_name();
                let row2 = v2.downcast_ref::<VaultsPageRow>().unwrap();
                let name2 = row2.get_name();
                name1.cmp(&name2)
            });

            self_.search_toggle_button.set_sensitive(true);
        } else {
            log::error!("Vault not initialised!");
        }
    }

    pub fn refresh(&self, map_is_empty: bool) {
        let self_ = imp::ApplicationWindow::from_instance(self);

        if self_.search_toggle_button.is_active() {
            return;
        }

        if map_is_empty {
            self.set_view(View::Start);
        } else {
            self.set_view(View::Vaults);
        }
    }

    pub fn refresh_new(&self) {
        if UserConfigManager::instance().get_map().is_empty() {
            self.set_view(View::Start);
        } else {
            self.set_view(View::Vaults);
        }
    }

    pub fn clear(&self) {
        let self_ = imp::ApplicationWindow::from_instance(self);
        self_.list_store.remove_all();
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
                        obj.set_view(View::Vaults);
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
                            let info_dialog = gtk::MessageDialog::builder()
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

                obj.set_view(View::Vaults);

                dialog.destroy();
            }
            _ => {
                dialog.destroy();
            }
        }));

        dialog.show();
    }

    fn refresh_clicked(&self) {
        self.clear();

        backend::probe_backends();

        UserConfigManager::instance().read_config();

        self.fill_list_store();

        if UserConfigManager::instance().get_map().is_empty() {
            self.set_view(View::Start);
        } else {
            self.set_view(View::Vaults);
        }
    }

    pub fn set_view(&self, view: View) {
        let self_ = imp::ApplicationWindow::from_instance(self);

        match view {
            View::Search => self_.window_stack.set_visible_child_name("search"),
            View::Start => {
                self_.search_toggle_button.set_sensitive(false);
                self_.window_stack.set_visible_child_name("start");
            }
            View::Vaults => {
                self_.search_toggle_button.set_sensitive(true);
                self_.window_stack.set_visible_child_name("vaults");
            }
        }
    }

    pub fn get_view(&self) -> Option<GString> {
        let self_ = imp::ApplicationWindow::from_instance(self);

        return self_.window_stack.visible_child_name();
    }
}
