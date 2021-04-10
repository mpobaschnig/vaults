use crate::config;
use crate::window::ApplicationWindow;
use gio::ApplicationFlags;
use glib::clone;
use glib::WeakRef;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib};
use gtk_macros::action;
use log::{debug, info};
use once_cell::sync::OnceCell;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct VApplication {
        pub window: OnceCell<WeakRef<ApplicationWindow>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VApplication {
        const NAME: &'static str = "VApplication";
        type Type = super::VApplication;
        type ParentType = gtk::Application;
    }

    impl ObjectImpl for VApplication {}

    impl gio::subclass::prelude::ApplicationImpl for VApplication {
        fn activate(&self, app: &Self::Type) {
            debug!("GtkApplication<VApplication>::activate");

            let priv_ = VApplication::from_instance(app);
            if let Some(window) = priv_.window.get() {
                let window = window.upgrade().unwrap();
                window.show();
                window.present();
                return;
            }

            app.set_resource_base_path(Some("/com/gitlab/mpobaschnig/Vaults/"));
            app.setup_css();

            let window = ApplicationWindow::new(app);
            self.window
                .set(window.downgrade())
                .expect("Window already set.");

            app.setup_gactions();
            app.setup_accels();

            app.get_main_window().present();
        }

        fn startup(&self, app: &Self::Type) {
            debug!("GtkApplication<VApplication>::startup");
            self.parent_startup(app);
        }
    }

    impl GtkApplicationImpl for VApplication {}
}

glib::wrapper! {
    pub struct VApplication(ObjectSubclass<imp::VApplication>)
        @extends gio::Application, gtk::Application, @implements gio::ActionMap, gio::ActionGroup;
}

impl VApplication {
    pub fn new() -> Self {
        glib::Object::new(&[
            ("application-id", &Some(config::APP_ID)),
            ("flags", &ApplicationFlags::empty()),
        ])
        .expect("Application initialization failed...")
    }

    fn get_main_window(&self) -> ApplicationWindow {
        let priv_ = imp::VApplication::from_instance(self);
        priv_.window.get().unwrap().upgrade().unwrap()
    }

    fn setup_gactions(&self) {
        action!(
            self,
            "add_new_vault",
            clone!(@weak self as app => move |_, _| {
                app.add_new_vault();
            })
        );

        action!(
            self,
            "import_vault",
            clone!(@weak self as app => move |_, _| {
                app.import_vault();
            })
        );

        // About
        action!(
            self,
            "about",
            clone!(@weak self as app => move |_, _| {
                app.show_about_dialog();
            })
        );
    }

    // Sets up keyboard shortcuts
    fn setup_accels(&self) {
        self.set_accels_for_action("app.quit", &["<primary>q"]);
        self.set_accels_for_action("win.show-help-overlay", &["<primary>question"]);
    }

    fn setup_css(&self) {
        let provider = gtk::CssProvider::new();
        provider.load_from_resource("/com/gitlab/mpobaschnig/Vaults/style.css");
        if let Some(display) = gdk::Display::get_default() {
            gtk::StyleContext::add_provider_for_display(
                &display,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    }

    fn add_new_vault(&self) {
        println!("Add new vault submenu button clicked!");
    }

    fn import_vault(&self) {
        println!("Import vault submenu button clicked");
    }

    #[allow(dead_code)]
    fn refresh_button_clicked(&self) {
        println!("Refresh button clicked!");
    }

    fn show_about_dialog(&self) {
        let dialog = gtk::AboutDialogBuilder::new()
            .program_name("Vaults")
            .logo_icon_name(config::APP_ID)
            .license_type(gtk::License::Gpl30)
            .website("https://gitlab.com/mpobaschnig/Vaults")
            .version(config::VERSION)
            .transient_for(&self.get_main_window())
            .modal(true)
            .authors(vec!["Martin Pobaschnig".into()])
            .artists(vec!["Martin Pobaschnig".into()])
            .build();

        dialog.show();
    }

    pub fn run(&self) {
        info!("Vaults ({})", config::APP_ID);
        info!("Version: {} ({})", config::VERSION, config::PROFILE);
        info!("Datadir: {}", config::PKGDATADIR);

        ApplicationExtManual::run(self);
    }
}
