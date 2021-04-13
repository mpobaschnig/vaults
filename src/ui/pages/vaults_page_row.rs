// vaults_page_row.rs
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

use adw::{subclass::prelude::*, PreferencesRowExt};
use glib::subclass;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::cell::RefCell;

use crate::vault::Vault;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/gitlab/mpobaschnig/Vaults/vaults_page_row.ui")]
    pub struct VaultsPageRow {
        #[template_child]
        pub vaults_page_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub open_folder_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub locker_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub settings_button: TemplateChild<gtk::Button>,

        pub vault: RefCell<Vault>,
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
                vault: RefCell::new(Vault::default()),
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
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
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
    pub fn new(vault: Vault) -> Self {
        let object: Self = glib::Object::new(&[]).expect("Failed to create VaultsPageRow");

        let self_ = &imp::VaultsPageRow::from_instance(&object);
        self_.vaults_page_row.set_title(Some(&vault.name));
        self_.vault.replace(vault);

        object
    }

    pub fn set_vault(&self, vault: Vault) {
        let self_ = imp::VaultsPageRow::from_instance(&self);
        self_.vault.replace(vault);
    }

    pub fn get_vault(&self) -> Vault {
        let self_ = imp::VaultsPageRow::from_instance(&self);
        return self_.vault.borrow().clone();
    }
}
