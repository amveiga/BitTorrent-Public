use gtk::gdk;
use gtk::prelude::*;
use log::LevelFilter;
use sitos::csv_env as bitEnv;
use sitos::frontend::index as frontend;
use sitos::logger::Logger;

fn main() {
    bitEnv::set_env(String::from("congif"));
    Logger::activate(Some("sitos.log".to_string()), Some(LevelFilter::Info)).unwrap();
    let application = gtk::Application::new(Some("sitos.bit.torrent"), Default::default());

    application.connect_startup(|_app| {
        let provider = gtk::CssProvider::new();
        // Load the CSS file
        let style = include_bytes!("frontend/style.css");
        provider.load_from_data(style).expect("Failed to load CSS");
        // We give the CssProvided to the default screen so the CSS rules we added
        // can be applied to our window.
        gtk::StyleContext::add_provider_for_screen(
            &gdk::Screen::default().expect("Error initializing gtk css provider."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    });

    application.connect_activate(frontend::build_ui);

    application.run();
}
