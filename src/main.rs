mod application;
#[rustfmt::skip]
mod config;
mod window;

use application::VApplication;
use config::{GETTEXT_PACKAGE, LOCALEDIR, RESOURCES_FILE};
use gettextrs::*;
use gtk::gio;

fn main() {
    // Initialize logger, debug is carried out via debug!, info!, and warn!.
    pretty_env_logger::init();

    // Prepare i18n
    setlocale(LocaleCategory::LcAll, "");

    bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR);
    textdomain(GETTEXT_PACKAGE);

    gtk::glib::set_application_name("Vaults");
    gtk::glib::set_prgname(Some("vaults"));

    gtk::init().expect("Unable to start GTK4");

    let res = gio::Resource::load(RESOURCES_FILE).expect("Could not load gresource file");
    gio::resources_register(&res);

    let app = VApplication::new();
    app.run();
}
