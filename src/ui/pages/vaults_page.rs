// vaults_page.rs
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

use super::VaultsPageRow;
use adw::subclass::prelude::*;
use glib::{clone, subclass};
use gtk::gio::ListStore;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use crate::{user_config_manager::UserConnfigManager, vault::*};

mod imp {
    use glib::once_cell::sync::Lazy;
    use gtk::glib::subclass::Signal;

    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/io/github/mpobaschnig/Vaults/vaults_page.ui")]
    pub struct VVaultsPage {
        #[template_child]
        pub vaults_list_box: TemplateChild<gtk::ListBox>,

        pub list_store: ListStore,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VVaultsPage {
        const NAME: &'static str = "VVaultsPage";
        type ParentType = adw::Bin;
        type Type = super::VVaultsPage;

        fn new() -> Self {
            Self {
                vaults_list_box: TemplateChild::default(),
                list_store: ListStore::new(gtk::Widget::static_type()),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for VVaultsPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            UserConnfigManager::instance().connect_add_vault(clone!(@weak obj => move || {
                obj.add_vault();
            }));

            let obj_ = imp::VVaultsPage::from_instance(&obj);
            obj_.vaults_list_box
                .bind_model(Some(&obj_.list_store), |obj| {
                    obj.clone().downcast::<gtk::Widget>().unwrap()
                });
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(
                    || vec![Signal::builder("refresh", &[], glib::Type::UNIT.into()).build()],
                );

            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for VVaultsPage {}

    impl BinImpl for VVaultsPage {}
}

glib::wrapper! {
    pub struct VVaultsPage(ObjectSubclass<imp::VVaultsPage>)
        @extends gtk::Widget, adw::Bin;
}

impl VVaultsPage {
    pub fn connect_refresh<F: Fn() + 'static>(&self, callback: F) -> glib::SignalHandlerId {
        self.connect_local("refresh", false, move |_| {
            callback();
            None
        })
        .unwrap()
    }

    pub fn row_connect_remove(&self, row: &VaultsPageRow) {
        row.connect_remove(clone!(@weak self as obj, @weak row => move || {
            let obj_ = imp::VVaultsPage::from_instance(&obj);
            let index = obj_.list_store.find(&row);
            if let Some(index) = index {
                obj_.list_store.remove(index);
                obj.emit_by_name("refresh", &[]).unwrap();
            } else {
                log::error!("Vault not initialised!");
            }

        }));
    }

    pub fn row_connect_save(&self, row: &VaultsPageRow) {
        row.connect_save(clone!(@weak self as obj, @weak row as r => move || {
            let vault = UserConnfigManager::instance().get_current_vault();
            if let Some(vault) = vault {
                r.set_vault(vault);
                obj.emit_by_name("refresh", &[]).unwrap();
            } else {
                log::error!("Vault not initialised!");
            }
        }));
    }

    pub fn new() -> Self {
        let window: Self = glib::Object::new(&[]).expect("Failed to create VVaultsPage");

        window
    }

    pub fn init(&self) {
        let self_ = imp::VVaultsPage::from_instance(self);

        let map = UserConnfigManager::instance().get_map();
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

    pub fn add_vault(&self) {
        let self_ = imp::VVaultsPage::from_instance(self);

        let vault = UserConnfigManager::instance().get_current_vault();

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
        } else {
            log::error!("Vault not initialised!");
        }
    }

    pub fn clear(&self) {
        let self_ = imp::VVaultsPage::from_instance(self);
        self_.list_store.remove_all();
    }
}
