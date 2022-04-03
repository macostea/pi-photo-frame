use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use gtk::glib::{self, timeout_future_seconds, timeout_future};
use gtk::glib::{MainContext, clone};
use gtk::{prelude::*, CssProvider, StyleContext, Application};
use gtk::gdk::Display;
use gtk::gdk_pixbuf::{Pixbuf, PixbufRotation};
use serde::Deserialize;
use rumqttc::{MqttOptions, QoS, Client, Connection, Event::Incoming, Packet::Publish};

use crate::photo::PhotoProvider;

use super::main_view::MainView;

#[derive(Deserialize, Default)]
pub struct Config {
    paths: Vec<String>,
    transition_time: u32,
    mqtt: bool,
    mqtt_host: String,
    mqtt_topic: String,
}

pub struct App {
    gtk_application: Option<Application>,
    config: Config,
    main_view: Rc<RefCell<MainView>>,
    photo_provider: Rc<RefCell<PhotoProvider>>,
}

impl App {
    pub fn new() -> Self {
        App {
            gtk_application: None,
            config: Default::default(),
            main_view: Rc::new(RefCell::new(MainView::new())),
            photo_provider: Rc::new(RefCell::new(PhotoProvider::default())),
        }
    }

    pub fn build_application(&mut self, config: Config) {
        self.config = config;

        let photo_provider = PhotoProvider::new(self.config.paths.clone());
        self.photo_provider = Rc::new(RefCell::new(photo_provider));

        let app = Application::builder()
            .application_id("com.mcostea.photo-frame")
            .build();

        let main_view = self.main_view.clone();

        let photo_provider = self.photo_provider.clone();
        let timeout = self.config.transition_time;
        let mqtt = self.config.mqtt.clone();
        let mqtt_host = self.config.mqtt_host.clone();
        let mqtt_topic = self.config.mqtt_topic.clone();
    
        app.connect_startup(|_| App::load_css());
        app.connect_activate(move |app| {
            main_view.borrow_mut().build_main_view(&app);

            let main_context = MainContext::default();
            let time_label = main_view.borrow().time_label.clone().unwrap();
            let date_label = main_view.borrow().date_label.clone().unwrap();
            let picture = main_view.borrow().picture.clone().unwrap();

            main_context.spawn_local(clone!(@weak time_label, @weak date_label => async move {
                loop {
                    timeout_future_seconds(1).await;

                    let now = Rc::new(gtk::glib::DateTime::now_local().unwrap());
                    let time_str = now.clone().format("%H:%M").unwrap();
                    let date_str = now.clone().format("%A, %B %d, %Y").unwrap();
                    time_label.set_text(time_str.to_string().as_str());
                    date_label.set_text(date_str.to_string().as_str());
                }
            }));

            main_context.spawn_local(clone!(@weak picture, @weak photo_provider => async move {
                loop {
                    timeout_future_seconds(timeout).await;

                    let photo = photo_provider.borrow().get_photo();
                    if let Ok(photo) = photo {
                        if photo.orientation == 1 {
                            let file = gtk::gio::File::for_path(photo.path.to_str().unwrap());
                            picture.set_file(Some(&file));
                        } else {
                            // We need to rotate the image
                            let pixbuf = Pixbuf::from_file(photo.path.to_str().unwrap()).unwrap();
                            let new_pixbuf = match photo.orientation {
                                2 => {
                                    pixbuf.flip(true).unwrap()
                                },
                                3 => {
                                    pixbuf.rotate_simple(PixbufRotation::Upsidedown).unwrap()
                                },
                                4 => {
                                    pixbuf.flip(true).unwrap().rotate_simple(PixbufRotation::Upsidedown).unwrap()
                                },
                                5 => {
                                    pixbuf.flip(true).unwrap().rotate_simple(PixbufRotation::Clockwise).unwrap()
                                },
                                6 => {
                                    pixbuf.rotate_simple(PixbufRotation::Clockwise).unwrap()
                                },
                                7 => {
                                    pixbuf.flip(true).unwrap().rotate_simple(PixbufRotation::Counterclockwise).unwrap()
                                },
                                8 => {
                                    pixbuf.rotate_simple(PixbufRotation::Counterclockwise).unwrap()
                                },
                                _ => pixbuf
                            };
                            picture.set_pixbuf(Some(&new_pixbuf));
                        }

                    } else {
                        println!("Error getting photo, {}", photo.unwrap_err());
                    }
                }
            }));

            if mqtt {
                main_context.spawn_local(clone!(@strong mqtt_host, @strong mqtt_topic, @weak photo_provider => async move {
                    let mqtt_topic_clone = mqtt_topic.clone();
                    let mut connection = App::connect_mqtt(mqtt_host, mqtt_topic);
    
                    for (_, notification) in connection.iter().enumerate() {
                        timeout_future(Duration::from_millis(500)).await;
                        if let Ok(Incoming(Publish(notification))) = notification {
                            if notification.topic == mqtt_topic_clone {
                                let payload = String::from_utf8(notification.payload[..].to_vec()).unwrap();
                                let power = if payload == "1" {"0"} else {"1"};
                                println!("Received MQTT notification {}", payload);
                                if payload == "1" {
                                    photo_provider.borrow_mut().paused = false;
                                } else {
                                    photo_provider.borrow_mut().paused = true;
                                }
                                let err = run_script::run_script!(format!("echo {} | sudo tee /sys/class/backlight/rpi_backlight/bl_power", power));
                                if err.is_err() {
                                    println!("Failed to switch lcd display");
                                }
                            }
                        }
                    }
                }));
            }
        });
    
        self.gtk_application = Some(app);
    }

    pub fn run(&self) {
        let gtk_application = self.gtk_application.as_ref().unwrap();
        gtk_application.run();
    }

    fn load_css() {
        let provider = CssProvider::new();
        provider.load_from_data(include_bytes!("../style/style.css"));
    
        StyleContext::add_provider_for_display(
            &Display::default().expect("Could not connect to a display"),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    fn connect_mqtt(mqtt_host: String, mqtt_topic: String) -> Connection {
        let mut mqtt_options = MqttOptions::new("pi-photo-frame", mqtt_host, 1883);
        mqtt_options.set_keep_alive(Duration::from_secs(5));
        
        let (mut client, connection) = Client::new(mqtt_options, 10);
        client.subscribe(mqtt_topic, QoS::AtMostOnce).unwrap();

        return connection;
    }
}




