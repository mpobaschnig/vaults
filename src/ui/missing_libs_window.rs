// missing_libs_window.rs
//
// Copyright 2026 Martin Pobaschnig
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

use crate::config::APP_ID;

use adw::subclass::prelude::*;
use gtk::gio::Settings;
use gtk::{self, CompositeTemplate, gio, glib, glib::clone, prelude::*};

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/io/github/mpobaschnig/Vaults/missing_libs_window.ui")]
    pub struct MissingLibsWindow {
        #[template_child]
        pub dont_show_again_switch_row: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub more_info_button_row: TemplateChild<adw::ButtonRow>,

        pub settings: Settings,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MissingLibsWindow {
        const NAME: &'static str = "MissingLibsWindow";
        type ParentType = adw::Dialog;
        type Type = super::MissingLibsWindow;

        fn new() -> Self {
            Self {
                dont_show_again_switch_row: TemplateChild::default(),
                more_info_button_row: TemplateChild::default(),
                settings: Settings::new(APP_ID),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MissingLibsWindow {
        fn constructed(&self) {
            let obj = self.obj();
            self.parent_constructed();

            obj.setup_signals();
        }
    }

    impl WidgetImpl for MissingLibsWindow {}
    impl WindowImpl for MissingLibsWindow {}
    impl AdwDialogImpl for MissingLibsWindow {}
}

glib::wrapper! {
    pub struct MissingLibsWindow(ObjectSubclass<imp::MissingLibsWindow>)
        @extends gtk::Widget, adw::Dialog, adw::Window, gtk::Window,
        @implements gio::ActionMap, gio::ActionGroup, gtk::Accessible, gtk::Native, gtk::Root, gtk::ShortcutManager, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for MissingLibsWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl MissingLibsWindow {
    pub fn new() -> Self {
        let dialog: Self = glib::Object::builder().build();

        dialog.add_css_class("flat");

        dialog
    }

    fn setup_signals(&self) {
        self.imp()
            .dont_show_again_switch_row
            .connect_active_notify(clone!(
                #[weak(rename_to = obj)]
                self,
                move |_| {
                    obj.imp()
                        .settings
                        .set_boolean(
                            "show-missing-libs-window",
                            !obj.imp().dont_show_again_switch_row.is_active(),
                        )
                        .expect("Failed to set show-missing-libs-window setting.");
                }
            ));

        self.imp().more_info_button_row.connect_activated(clone!(
            #[weak(rename_to = _obj)]
            self,
            move |_| {
                let launcher = gtk::UriLauncher::builder()
                    .uri("https://github.com/mpobaschnig/vaults/wiki/CryFS")
                    .build();

                let window = gio::Application::default()
                    .unwrap()
                    .downcast_ref::<crate::application::VApplication>()
                    .unwrap()
                    .active_window()
                    .unwrap()
                    .clone();

                launcher.launch(Some(&window), gio::Cancellable::NONE, move |_| {});
            }
        ));
    }
}
