use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};

use gtk::glib::{self, timeout_future_seconds, PRIORITY_DEFAULT};
use gtk::glib::{MainContext, clone};
use gtk::{prelude::*, CssProvider, StyleContext, Application, MediaFile};
use gtk::gdk::Display;
use gtk::gdk_pixbuf::{Pixbuf, PixbufRotation, Colorspace};
use serde::Deserialize;
use rumqttc::{MqttOptions, QoS, Client, Connection, Event::Incoming, Packet::Publish};

use crate::photo::MediaProvider;
use crate::geocoder::Geocoder;
use crate::photo::Media;

use super::main_view::MainView;

#[derive(Deserialize, Default)]
pub struct Config {
    paths: Vec<String>,
    transition_time: u32,
    mqtt: bool,
    mqtt_host: String,
    mqtt_topic: String,
    reverse_geocode: bool,
    mapbox_api_key: String,
}

pub struct App {
    gtk_application: Option<Application>,
    config: Config,
    main_view: Rc<RefCell<MainView>>,
}

enum MediaMessage {
    Photo {
        photo: Media,
        photo_data: PhotoData,
        address: Result<String, String>,
    },
    Video {
        video: Media
    }
}

struct PhotoData {
    bytes: Box<glib::Bytes>,
    colorspace: Colorspace,
    has_alpha: bool,
    bits_per_sample: i32,
    width: i32,
    height: i32,
    rowstride: i32,
}

impl App {
    pub fn new() -> Self {
        App {
            gtk_application: None,
            config: Default::default(),
            main_view: Rc::new(RefCell::new(MainView::default())),
        }
    }

    pub fn build_application(&mut self, config: Config) {
        self.config = config;

        let media_provider = Arc::new(Mutex::new(MediaProvider::new(self.config.paths.clone())));

        let app = Application::builder()
            .application_id("com.mcostea.photo-frame")
            .build();

        let main_view = self.main_view.clone();

        let timeout = self.config.transition_time;
        let mqtt = self.config.mqtt.clone();
        let mqtt_host = self.config.mqtt_host.clone();
        let mqtt_topic = self.config.mqtt_topic.clone();
        let reverse_geocode = self.config.reverse_geocode.clone();
        let mapbox_api_key = self.config.mapbox_api_key.clone();
    
        app.connect_startup(|_| App::load_css());
        app.connect_activate(move |app| {
            main_view.borrow_mut().build_main_view(&app);

            let main_context = MainContext::default();
            let overlay = main_view.borrow().overlay.clone().unwrap();
            let time_label = main_view.borrow().time_label.clone().unwrap();
            let date_label = main_view.borrow().date_label.clone().unwrap();
            let picture = main_view.borrow().picture.clone().unwrap();
            let location_label = main_view.borrow().location_label.clone().unwrap();
            let photo_date_label = main_view.borrow().photo_date_label.clone().unwrap();
            let location_box = main_view.borrow().location_box.clone().unwrap();
            let mapbox_api_key = mapbox_api_key.clone();
            let play_pause_button = main_view.borrow().play_pause_button.clone().unwrap();
            let play_image = main_view.borrow().play_image.clone().unwrap();
            let pause_image = main_view.borrow().pause_image.clone().unwrap();
            let photo_location_label = main_view.borrow().photo_location_label.clone().unwrap();

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

            let (media_sender, media_receiver) = MainContext::channel::<MediaMessage>(PRIORITY_DEFAULT);
            let media_provider_clone = Arc::clone(&media_provider);

            thread::spawn(move || {
                let geocoder = Geocoder::new(mapbox_api_key);
                let photo_provider = Arc::clone(&media_provider_clone);
                loop {
                    thread::sleep(Duration::from_secs(timeout.into()));

                    let media = photo_provider.lock().unwrap().get_media();
                    match media {
                        Ok(Some(Media::Photo { ref path, orientation, location, date: _})) => {
                            // We might need to rotate the image
                            let pixbuf = Rc::new(Pixbuf::from_file(path.to_str().unwrap()).unwrap());
                            let new_pixbuf = match orientation {
                                1 => {
                                    Rc::clone(&pixbuf)
                                }
                                2 => {
                                    Rc::new(pixbuf.flip(true).unwrap())
                                },
                                3 => {
                                    Rc::new(pixbuf.rotate_simple(PixbufRotation::Upsidedown).unwrap())
                                },
                                4 => {
                                    Rc::new(pixbuf.flip(true).unwrap().rotate_simple(PixbufRotation::Upsidedown).unwrap())
                                },
                                5 => {
                                    Rc::new(pixbuf.flip(true).unwrap().rotate_simple(PixbufRotation::Clockwise).unwrap())
                                },
                                6 => {
                                    Rc::new(pixbuf.rotate_simple(PixbufRotation::Clockwise).unwrap())
                                },
                                7 => {
                                    Rc::new(pixbuf.flip(true).unwrap().rotate_simple(PixbufRotation::Counterclockwise).unwrap())
                                },
                                8 => {
                                    Rc::new(pixbuf.rotate_simple(PixbufRotation::Counterclockwise).unwrap())
                                },
                                _ => Rc::clone(&pixbuf)
                            };

                            drop(pixbuf);

                            let mut address_message = Err("Not set".into());
                            if reverse_geocode {
                                if let Some(location) = location {
                                    let address = geocoder.reverse_geocode(location.0, location.1);
                                    address_message = address;
                                }
                            }

                            let photo_obj = MediaMessage::Photo {
                                photo: media.unwrap().unwrap().clone(),
                                photo_data: PhotoData {
                                    bytes: Box::new(new_pixbuf.read_pixel_bytes().unwrap()),
                                    colorspace: new_pixbuf.colorspace(),
                                    has_alpha: new_pixbuf.has_alpha(),
                                    bits_per_sample: new_pixbuf.bits_per_sample(),
                                    width: new_pixbuf.width(),
                                    height: new_pixbuf.height(),
                                    rowstride: new_pixbuf.rowstride()
                                },
                                address: address_message
                            };

                            drop(new_pixbuf);



                            let res = media_sender.send(photo_obj);
                            if let Err(e) = res {
                                println!("Failed to send photo_obj between threads {}", e);
                            }

                        },
                        Ok(Some(Media::Video { path: _ })) => {
                            let video_obj = MediaMessage::Video {
                                video: media.unwrap().unwrap().clone()
                            };

                            let res = media_sender.send(video_obj);
                            if let Err(e) = res {
                                println!("Failed to send video_obj between threads {}", e);
                            }
                        },
                        Ok(None) => {
                            // Everything went ok but there was no media
                            // Most likely paused, don't do anything.
                        }
                        _ => {
                            println!("Error getting photo, {}", media.unwrap_err());
                        }
                    }
                }
            });

            media_receiver.attach(
                None,
                clone!(@weak overlay, @weak picture, @weak location_box, @weak location_label, @weak photo_location_label, @weak photo_date_label => @default-return Continue(false),
                    move |photo_obj| {
                        match photo_obj {
                            MediaMessage::Photo { photo, photo_data, address } => {
                                let pixbuf = Box::new(Pixbuf::from_bytes(
                                    &photo_data.bytes,
                                    photo_data.colorspace,
                                    photo_data.has_alpha,
                                    photo_data.bits_per_sample,
                                    photo_data.width,
                                    photo_data.height,
                                    photo_data.rowstride
                                ));

                                picture.set_pixbuf(Some(&pixbuf));

                                let mut location_found = false;
                                let mut date_found = false;

                                match address {
                                    Ok(a) => {
                                        location_found = true;
                                        location_label.set_text(format!("{}", a).as_str());
                                    },
                                    Err(e) => {
                                        location_label.set_text("");
                                        println!("Failed to get reverse geocode response, {}", e);
                                    }
                                }

                                if let Media::Photo { path, orientation: _, location: _, date } = photo {
                                    if let Some(string_date) = date {
                                        date_found = true;
                                        photo_date_label.set_text(string_date.as_str());
                                    } else {
                                        photo_date_label.set_text("");
                                    }

                                    photo_location_label.set_text(format!("{}", path.to_str().unwrap()).as_str());

                                }

                                if location_found || date_found {
                                    location_box.show();
                                } else {
                                    location_box.hide();
                                }
                            },

                            MediaMessage::Video { video: video_file } => {
                                if let Media::Video { path } = video_file {
                                    println!("Got a video, trying to play it {}", path.to_str().unwrap());
                                    let media_file = MediaFile::new();
                                    let file = gtk::gio::File::for_path(path);
                                    media_file.set_file(Some(&file));

                                    media_file.connect_playing_notify(
                                        move |media_file| {
                                            println!("Media is playing: {}", media_file.is_playing());
                                        }
                                    );
                                    media_file.connect_error_notify(
                                        move |media_file| {
                                            let error = media_file.error().unwrap();
                                            println!("Error in MediaFile: {}", error);
                                        }
                                    );
                                    media_file.connect_prepared_notify(
                                        move |media_file| {
                                            if media_file.error().is_some() {
                                                return;
                                            }
                                            if !media_file.has_video() {
                                                println!("Media is not a valid video file");
                                                return;
                                            }
                                        }
                                    );

                                    picture.set_paintable(Some(&media_file));
                                    media_file.play();
                                }
                            }
                        }

                        Continue(true)
                    })
            );

            let mqtt_host = mqtt_host.clone();
            let mqtt_topic = mqtt_topic.clone();

            if mqtt {
                let (sender, receiver) = MainContext::channel::<bool>(PRIORITY_DEFAULT);
                thread::spawn(move || {
                    let mqtt_topic_clone = mqtt_topic.clone();
                    let (mut client, mut connection) = App::connect_mqtt(mqtt_host);
                    App::subscribe_mqtt(&mut client, &mqtt_topic);

                    for (_, notification) in connection.iter().enumerate() {
                        match notification {
                            Ok(Incoming(Publish(notification))) => {
                                if notification.topic == mqtt_topic_clone {
                                    let payload = String::from_utf8(notification.payload[..].to_vec()).unwrap();
                                    let power = if payload == "1" {"0"} else {"1"};
                                    println!("Received MQTT notification {}", payload);
                                    let err = run_script::run_script!(format!("echo {} | sudo tee /sys/class/backlight/rpi_backlight/bl_power", power));
                                    if err.is_err() {
                                        println!("Failed to switch lcd display");
                                    }

                                    if payload == "1" {
                                        sender.send(false).unwrap();
                                    } else {
                                        sender.send(true).unwrap();
                                    }
                                }
                            },
                            Ok(Incoming(rumqttc::Packet::ConnAck(_))) => {
                                App::subscribe_mqtt(&mut client, &mqtt_topic);
                            },
                            _ => {

                            }
                        }
                    }
                });


                let media_provider_clone = Arc::clone(&media_provider);

                receiver.attach(
                    None,
                    clone!(@weak play_pause_button, @weak pause_image, @weak photo_location_label => @default-return Continue(false),
                        move |pause| {
                            media_provider_clone.lock().unwrap().paused = pause;
                            if !pause {
                                play_pause_button.set_child(Some(&pause_image));
                                photo_location_label.hide();
                            }
                            Continue(true)
                        })
                );
            }

            play_pause_button.connect_clicked(clone!(@weak media_provider, @weak photo_location_label => move |play_pause_button| {
                let mut media_provider = media_provider.lock().unwrap();
                match media_provider.paused {
                    true => {
                        media_provider.paused = false;
                        play_pause_button.set_child(Some(&pause_image));
                        photo_location_label.hide();
                    }
                    false => {
                        media_provider.paused = true;
                        play_pause_button.set_child(Some(&play_image));
                        photo_location_label.show();
                    }
                }
            }));
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

    fn connect_mqtt(mqtt_host: String) -> (Client, Connection) {
        let mut mqtt_options = MqttOptions::new("pi-photo-frame", mqtt_host, 1883);
        mqtt_options.set_keep_alive(Duration::from_secs(5));
        mqtt_options.set_clean_session(false);
        
        let (client, connection) = Client::new(mqtt_options, 10);

        return (client, connection);
    }

    fn subscribe_mqtt(client: &mut Client, mqtt_topic: &String) {
        client.subscribe(mqtt_topic, QoS::AtMostOnce).unwrap();
    }
}
