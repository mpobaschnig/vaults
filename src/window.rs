use crate::application::VApplication;
use crate::config::{APP_ID, PROFILE};
use adw::subclass::prelude::*;
use glib::clone;
use glib::signal::Inhibit;
use gtk::subclass::prelude::*;
use gtk::{self, prelude::*};
use gtk::{gio, glib, CompositeTemplate};
use log::warn;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/org/gnome/gitlab/mpobaschnig/Vaults/window.ui")]
    pub struct VApplicationWindow {
        #[template_child]
        pub headerbar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub refresh_button: TemplateChild<gtk::Button>,
        pub settings: gio::Settings,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VApplicationWindow {
        const NAME: &'static str = "VApplicationWindow";
        type Type = super::VApplicationWindow;
        type ParentType = adw::ApplicationWindow;

        fn new() -> Self {
            Self {
                headerbar: TemplateChild::default(),
                refresh_button: TemplateChild::default(),
                settings: gio::Settings::new(APP_ID),
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

    impl ObjectImpl for VApplicationWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            // Devel Profile
            if PROFILE == "Devel" {
                obj.get_style_context().add_class("devel");
            }

            obj.setup_connect_handlers();
        }
    }

    impl WidgetImpl for VApplicationWindow {}
    impl WindowImpl for VApplicationWindow {}

    impl ApplicationWindowImpl for VApplicationWindow {}
    impl AdwApplicationWindowImpl for VApplicationWindow {}
}

glib::wrapper! {
    pub struct VApplicationWindow(ObjectSubclass<imp::VApplicationWindow>)
        @extends gtk::Widget, gtk::Window, adw::ApplicationWindow, @implements gio::ActionMap, gio::ActionGroup;
}

impl VApplicationWindow {
    pub fn new(app: &VApplication) -> Self {
        let window: Self = glib::Object::new(&[]).expect("Failed to create VApplicationWindow");
        window.set_application(Some(app));

        // Set icons for shell
        gtk::Window::set_default_icon_name(APP_ID);

        window
    }

    fn setup_connect_handlers(&self) {
        let self_ = imp::VApplicationWindow::from_instance(self);

        self_
            .refresh_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.refresh_button_clicked();
            }));
    }

    fn refresh_button_clicked(&self) {
        println!("Refresh button clicked!");
    }
}
