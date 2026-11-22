// vaults_page_row_change_password_window.rs
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

use adw::{
    prelude::{AdwDialogExt, EntryRowExt},
    subclass::prelude::*,
};
use gtk::{self, CompositeTemplate, gio, glib, glib::clone, prelude::*};

mod imp {
    use gtk::glib::subclass::Signal;
    use once_cell::sync::Lazy;

    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(
        resource = "/io/github/mpobaschnig/Vaults/vaults_page_row_change_password_window.ui"
    )]
    pub struct VaultsPageRowChangePasswordWindow {
        #[template_child]
        pub change_password_row: TemplateChild<adw::ButtonRow>,
        #[template_child]
        pub old_password_entry_row: TemplateChild<adw::PasswordEntryRow>,
        #[template_child]
        pub new_password_entry_row: TemplateChild<adw::PasswordEntryRow>,
        #[template_child]
        pub confirm_password_entry_row: TemplateChild<adw::PasswordEntryRow>,
        #[template_child]
        pub password_mismatch_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub status_page: TemplateChild<adw::StatusPage>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VaultsPageRowChangePasswordWindow {
        const NAME: &'static str = "VaultsPageRowChangePasswordWindow";
        type ParentType = adw::Dialog;
        type Type = super::VaultsPageRowChangePasswordWindow;

        fn new() -> Self {
            Self {
                change_password_row: TemplateChild::default(),
                old_password_entry_row: TemplateChild::default(),
                new_password_entry_row: TemplateChild::default(),
                confirm_password_entry_row: TemplateChild::default(),
                password_mismatch_label: TemplateChild::default(),
                status_page: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for VaultsPageRowChangePasswordWindow {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("change").build()]);
            SIGNALS.as_ref()
        }

        fn constructed(&self) {
            let obj = self.obj();
            self.parent_constructed();

            obj.setup_signals();
        }
    }

    impl WidgetImpl for VaultsPageRowChangePasswordWindow {}
    impl WindowImpl for VaultsPageRowChangePasswordWindow {}
    impl AdwDialogImpl for VaultsPageRowChangePasswordWindow {}
}

glib::wrapper! {
    pub struct VaultsPageRowChangePasswordWindow(ObjectSubclass<imp::VaultsPageRowChangePasswordWindow>)
        @extends gtk::Widget, adw::Dialog, adw::Window, gtk::Window,
        @implements gio::ActionMap, gio::ActionGroup, gtk::Accessible, gtk::Native, gtk::Root, gtk::ShortcutManager, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for VaultsPageRowChangePasswordWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl VaultsPageRowChangePasswordWindow {
    pub fn new() -> Self {
        let dialog: Self = glib::Object::builder().build();

        dialog.add_css_class("flat");

        dialog
    }

    fn setup_signals(&self) {
        self.imp().change_password_row.connect_activated(clone!(
            #[weak(rename_to = obj)]
            self,
            move |_| {
                obj.change_button_clicked();
            }
        ));

        self.imp()
            .old_password_entry_row
            .connect_text_notify(clone!(
                #[weak(rename_to = obj)]
                self,
                move |_| {
                    obj.check_change_button_enable_conditions();
                }
            ));

        self.imp()
            .new_password_entry_row
            .connect_text_notify(clone!(
                #[weak(rename_to = obj)]
                self,
                move |_| {
                    obj.check_change_button_enable_conditions();
                }
            ));

        self.imp()
            .confirm_password_entry_row
            .connect_text_notify(clone!(
                #[weak(rename_to = obj)]
                self,
                move |_| {
                    obj.check_change_button_enable_conditions();
                }
            ));

        self.imp()
            .new_password_entry_row
            .connect_entry_activated(clone!(
                #[weak(rename_to = obj)]
                self,
                move |_| {
                    obj.connect_activate();
                }
            ));

        self.imp()
            .confirm_password_entry_row
            .connect_entry_activated(clone!(
                #[weak(rename_to = obj)]
                self,
                move |_| {
                    obj.connect_activate();
                }
            ));
    }

    pub fn set_name(&self, name: &str) {
        self.imp().status_page.set_title(name);
    }

    fn change_button_clicked(&self) {
        self.emit_by_name::<()>("change", &[]);
        AdwDialogExt::close(self);
    }

    fn check_change_button_enable_conditions(&self) {
        let old_password = self.imp().old_password_entry_row.text();
        let new_password = self.imp().new_password_entry_row.text();
        let confirm_password = self.imp().confirm_password_entry_row.text();

        let passwords_match = new_password == confirm_password && !new_password.is_empty();

        // Show/hide mismatch error label
        if !new_password.is_empty() || !confirm_password.is_empty() {
            self.imp()
                .password_mismatch_label
                .set_visible(!passwords_match);
        } else {
            self.imp().password_mismatch_label.set_visible(false);
        }

        if !old_password.is_empty() && passwords_match {
            self.imp().change_password_row.set_sensitive(true);
        } else {
            self.imp().change_password_row.set_sensitive(false);
        }
    }

    fn connect_activate(&self) {
        let old_password = self.imp().old_password_entry_row.text();
        let new_password = self.imp().new_password_entry_row.text();
        let confirm_password = self.imp().confirm_password_entry_row.text();
        if !old_password.is_empty()
            && !new_password.is_empty()
            && !confirm_password.is_empty()
            && new_password == confirm_password
        {
            self.change_button_clicked();
        }
    }

    pub fn get_old_password(&self) -> String {
        self.imp().old_password_entry_row.text().to_string()
    }

    pub fn get_new_password(&self) -> String {
        self.imp().new_password_entry_row.text().to_string()
    }
}
