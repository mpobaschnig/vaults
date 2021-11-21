// start_page.rs
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

use adw::subclass::prelude::*;
use glib::subclass;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/mpobaschnig/Vaults/start_page.ui")]
    pub struct VStartPage {}

    #[glib::object_subclass]
    impl ObjectSubclass for VStartPage {
        const NAME: &'static str = "VStartPage";
        type ParentType = adw::Bin;
        type Type = super::VStartPage;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for VStartPage {}

    impl WidgetImpl for VStartPage {}

    impl BinImpl for VStartPage {}
}

glib::wrapper! {
    pub struct VStartPage(ObjectSubclass<imp::VStartPage>)
        @extends gtk::Widget, adw::Bin;
}

impl VStartPage {
    pub fn init(&self) {}
}
