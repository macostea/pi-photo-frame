use std::{env, fs};

mod gui;
use gui::application::{App, Config};

mod photo;

fn load_config() -> Config {
    println!("Current dir {}", env::current_dir().unwrap().display());
    let config_path = env::current_dir().unwrap().join("config.json5");
    json5::from_str(&fs::read_to_string(config_path).unwrap()).unwrap()
}

fn main() {
    let config = load_config();
    let mut app = App::new();
    
    app.build_application(config);

    app.run();
}
