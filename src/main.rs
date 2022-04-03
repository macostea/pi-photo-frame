use std::{fs, path::Path};

mod gui;
use gui::application::{App, Config};

mod photo;

fn load_config() -> Config {
    let config_path = Path::new("/etc/pi-photo-frame.json5");
    json5::from_str(&fs::read_to_string(config_path).unwrap()).unwrap()
}

fn main() {
    let config = load_config();
    let mut app = App::new();
    
    app.build_application(config);

    app.run();
}
