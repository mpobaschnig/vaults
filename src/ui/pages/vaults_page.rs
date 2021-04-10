// vaults_page.rs
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

use adw::subclass::prelude::*;
use glib::subclass;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/gitlab/mpobaschnig/Vaults/vaults_page.ui")]
    pub struct VVaultsPage {}

    #[glib::object_subclass]
    impl ObjectSubclass for VVaultsPage {
        const NAME: &'static str = "VVaultsPage";
        type ParentType = adw::Bin;
        type Type = super::VVaultsPage;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for VVaultsPage {}

    impl WidgetImpl for VVaultsPage {}

    impl BinImpl for VVaultsPage {}
}

glib::wrapper! {
    pub struct VVaultsPage(ObjectSubclass<imp::VVaultsPage>)
        @extends gtk::Widget, adw::Bin;
}

impl VVaultsPage {
    pub fn init(&self) {}
}
