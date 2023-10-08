use std::{fs, io, path::Path};

use gtk::gio::{self, prelude::*};
use gtk::glib;
use once_cell::sync::Lazy;
use photo::provider::Config;
use tracing::Level;

mod config;
mod geocoder;
mod gui;
mod photo;
mod utils;
mod window;

mod application;

use application::PpfApplication;

fn load_config() -> Config {
    let mut path = Path::new(".config.json5");
    if !path.exists() {
        path = Path::new("/etc/pi-photo-frame.json5");
    }
    json5::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

pub static RUNTIME: Lazy<tokio::runtime::Runtime> =
    Lazy::new(|| tokio::runtime::Runtime::new().unwrap());

static GRESOURCE_BYTES: &[u8] =
    gvdb_macros::include_gresource_from_dir!("/com/mcostea/PiPhotoFrame", "data/resources");

fn main() -> glib::ExitCode {
    let config = load_config();

    gio::resources_register(
        &gio::Resource::from_data(&glib::Bytes::from_static(GRESOURCE_BYTES)).unwrap(),
    );

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_writer(io::stdout)
        .init();

    PpfApplication::new(config).run()
}
