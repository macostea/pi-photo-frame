use std::{fs, io, path::Path};

mod controllers;

use controllers::application::{App, Config};
use tracing::Level;

mod geocoder;
mod gui;
mod photo;
mod utils;

#[cfg(feature = "sentry-native")]
#[link(name = "sentrysample", kind = "dylib")]
#[link(name = "sentry", kind = "dylib")]

extern "C" {
    fn init_native();
}

fn load_config() -> Config {
    let mut path = Path::new("config.json5");
    if !path.exists() {
        path = Path::new("/etc/pi-photo-frame.json5");
    }
    json5::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn main() {
    #[cfg(feature = "sentry-native")]
    unsafe {
        init_native();
    }

    let config = load_config();
    let _guard = sentry::init((
        config.sentry_uri.clone(),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));
    let mut app = App::new();

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_writer(io::stdout)
        .init();

    app.build_application(config);

    app.run();
}
