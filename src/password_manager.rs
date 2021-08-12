// password_manager.rs
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

use gtk::{
    gio::subclass::prelude::*,
    glib::{self},
};
use std::cell::RefCell;

static mut PASSWORD_MANAGER: Option<PasswordManager> = None;

mod imp {
    use super::*;

    #[derive(Debug)]
    pub struct PasswordManager {
        pub current_password: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PasswordManager {
        const NAME: &'static str = "PasswordManager";
        type ParentType = glib::Object;
        type Type = super::PasswordManager;

        fn new() -> Self {
            Self {
                current_password: RefCell::new(None),
            }
        }
    }

    impl ObjectImpl for PasswordManager {}
}

glib::wrapper! {
    pub struct PasswordManager(ObjectSubclass<imp::PasswordManager>);
}

impl PasswordManager {
    pub fn instance() -> Self {
        unsafe {
            match PASSWORD_MANAGER.as_ref() {
                Some(user_config) => user_config.clone(),
                None => {
                    let user_config = PasswordManager::new();
                    PASSWORD_MANAGER = Some(user_config.clone());
                    user_config
                }
            }
        }
    }

    fn new() -> Self {
        let object: Self = glib::Object::new(&[]).expect("Failed to create PasswordManager");

        object
    }

    pub fn get_current_password(&self) -> Option<String> {
        let self_ = &mut imp::PasswordManager::from_instance(&self);
        self_.current_password.borrow().clone()
    }

    pub fn set_current_password(&self, password: String) {
        let self_ = &mut imp::PasswordManager::from_instance(&self);
        self_.current_password.borrow_mut().replace(password);
    }

    pub fn clear_current_pssword(&self) {
        let self_ = &mut imp::PasswordManager::from_instance(&self);
        *self_.current_password.borrow_mut() = None;
    }
}
