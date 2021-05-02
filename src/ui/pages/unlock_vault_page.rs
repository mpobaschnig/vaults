// unlock_vault_page.rs
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

use crate::password_manager::PasswordManager;
use crate::ui::pages::VaultsPageRow;
use crate::ui::ApplicationWindow;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use glib::once_cell::sync::Lazy;
use glib::subclass;
use gtk::glib;
use gtk::glib::clone;
use gtk::glib::subclass::Signal;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/mpobaschnig/Vaults/unlock_vault_page.ui")]
    pub struct UnlockVaultPage {
        #[template_child]
        pub password_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub unlock_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub password_entry: TemplateChild<gtk::Entry>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for UnlockVaultPage {
        const NAME: &'static str = "UnlockVaultPage";
        type ParentType = adw::Bin;
        type Type = super::UnlockVaultPage;

        fn new() -> Self {
            Self {
                password_label: TemplateChild::default(),
                password_entry: TemplateChild::default(),
                unlock_button: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for UnlockVaultPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.setup_signals();
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("unlock", &[], glib::Type::UNIT.into()).build()]);

            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for UnlockVaultPage {}

    impl BinImpl for UnlockVaultPage {}
}

glib::wrapper! {
    pub struct UnlockVaultPage(ObjectSubclass<imp::UnlockVaultPage>)
        @extends gtk::Widget, adw::Bin;
}

impl UnlockVaultPage {
    pub fn connect_unlock<F: Fn() + 'static>(&self, callback: F) -> glib::SignalHandlerId {
        self.connect_local("unlock", false, move |_| {
            callback();
            None
        })
        .unwrap()
    }

    pub fn init(&self) {
        let self_ = imp::UnlockVaultPage::from_instance(self);

        self_.password_entry.set_text("");
        self_.unlock_button.set_sensitive(false);

        self_.password_entry.grab_focus_without_selecting();
    }

    fn setup_signals(&self) {
        let self_ = imp::UnlockVaultPage::from_instance(self);

        self_
            .unlock_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.unlock_button_clicked();
            }));

        self_
            .password_entry
            .connect_property_text_notify(clone!(@weak self as obj => move |_| {
                obj.check_unlock_button_enable_conditions();
            }));

        self_
            .password_entry
            .connect_activate(clone!(@weak self as obj => move |_| {
                obj.connect_activate();
            }));
    }

    fn unlock_button_clicked(&self) {
        let self_ = imp::UnlockVaultPage::from_instance(self);

        let password = self_.password_entry.get_text().to_string();

        PasswordManager::instance().set_current_password(password);

        self.emit_by_name("unlock", &[]).unwrap();
    }

    fn check_unlock_button_enable_conditions(&self) {
        let self_ = imp::UnlockVaultPage::from_instance(self);

        let password = self_.password_entry.get_text();

        if !password.is_empty() {
            self_.unlock_button.set_sensitive(true);
        } else {
            self_.unlock_button.set_sensitive(false);
        }
    }

    fn connect_activate(&self) {
        let self_ = imp::UnlockVaultPage::from_instance(self);

        if !self_.password_entry.get_text().is_empty() {
            self.unlock_button_clicked();
        }
    }

    pub fn call_unlock(&self, row: &VaultsPageRow) {
        let self_ = imp::UnlockVaultPage::from_instance(&self);

        let name = row.get_name();
        let mut label_text = String::from(&gettext("Enter Password for"));
        label_text.push_str(" ");
        label_text.push_str(&name);

        self_.password_label.set_text(&label_text);

        let ancestor = self.get_ancestor(ApplicationWindow::static_type()).unwrap();
        let window = ancestor.downcast_ref::<ApplicationWindow>().unwrap();

        window.set_unlock_vault_view();

        row.set_settings_handler_id(self.connect_unlock(
            clone!(@weak self as obj, @weak row => move || {
                let self_ = imp::UnlockVaultPage::from_instance(&obj);

                let password = self_.password_entry.get_text().to_string();

                obj.disconnect(row.get_settings_handler_id());

                let ancestor = obj.get_ancestor(ApplicationWindow::static_type()).unwrap();
                let window = ancestor.downcast_ref::<ApplicationWindow>().unwrap();
                window.set_standard_window_view();

                row.unlock(password);
            }),
        ));
    }
}
