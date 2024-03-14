use std::io;

use gtk::gio::{self, prelude::*};
use gtk::glib;
use once_cell::sync::Lazy;
use photo::provider::{load_config, load_failed_files};
use tracing::Level;

mod config;
mod geocoder;
mod gui;
mod photo;
mod utils;
mod window;

mod application;

use application::PpfApplication;

pub static RUNTIME: Lazy<tokio::runtime::Runtime> =
    Lazy::new(|| tokio::runtime::Runtime::new().unwrap());

static GRESOURCE_BYTES: &[u8] =
    gvdb_macros::include_gresource_from_dir!("/com/mcostea/PiPhotoFrame", "data/resources");

fn main() -> glib::ExitCode {
    let config = load_config();
    let failed_files = load_failed_files();

    let _guard = sentry::init(("SENTRY_DSN", sentry::ClientOptions {
        release: sentry::release_name!(),
        ..Default::default()
    }));

    gio::resources_register(
        &gio::Resource::from_data(&glib::Bytes::from_static(GRESOURCE_BYTES)).unwrap(),
    );

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_writer(io::stdout)
        .init();

    PpfApplication::new(config, failed_files).run()
}
