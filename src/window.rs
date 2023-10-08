use crate::gui::play_pause_button::PpfPlayPauseButton;
use crate::photo::provider::{Config, MediaMessage};
use crate::photo::{Media, MediaProvider};
use crate::{spawn, spawn_tokio};
use gtk::glib::{MainContext, PRIORITY_DEFAULT};
use gtk::MediaFile;
use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};
use gtk::{
    glib::{clone, timeout_future_seconds},
    Label,
};
use rumqttc::{Event::Incoming, MqttOptions, Packet::Publish, QoS};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{debug, span, Level};

mod imp {
    use gtk::Picture;

    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(file = "../data/gtk/window.ui")]
    pub struct PpfWindow {
        #[template_child]
        pub(super) play_pause_button: TemplateChild<PpfPlayPauseButton>,
        #[template_child]
        pub(super) time_label: TemplateChild<Label>,
        #[template_child]
        pub(super) date_label: TemplateChild<Label>,
        #[template_child]
        pub(super) picture: TemplateChild<Picture>,
        #[template_child]
        pub(super) location_label: TemplateChild<Label>,
        #[template_child]
        pub(super) photo_date_label: TemplateChild<Label>,
        #[template_child]
        pub(super) photo_location_label: TemplateChild<Label>,
        #[template_child]
        pub(super) location_box: TemplateChild<gtk::Box>,

        pub(super) config: RefCell<Config>,
        pub(super) media_provider: RefCell<Arc<Mutex<MediaProvider>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PpfWindow {
        const NAME: &'static str = "PpfWindow";
        type Type = super::PpfWindow;
        type ParentType = gtk::Window;

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.bind_template_instance_callbacks();
        }
    }

    impl ObjectImpl for PpfWindow {
        fn constructed(&self) {
            let obj = self.obj();
            self.parent_constructed();

            spawn!(clone!(@weak obj => async move {
              obj.start_timer().await;
            }));

            self.play_pause_button.connect_notify_local(
                Some("is-paused"),
                clone!(@weak obj => move |button, _| {
                    let is_paused = button.is_paused();
                    obj.handle_play_pause_toggled(is_paused);
                }),
            );
        }
    }
    impl WindowImpl for PpfWindow {}
    impl WidgetImpl for PpfWindow {}
}

glib::wrapper! {
  pub struct PpfWindow(ObjectSubclass<imp::PpfWindow>)
    @extends gtk::Widget, gtk::Window,
    @implements gio::ActionMap, gio::ActionGroup, gtk::Native;
}

#[gtk::template_callbacks]
impl PpfWindow {
    pub fn new<A: IsA<gtk::Application>>(app: &A, config: Config) -> Self {
        let obj: PpfWindow = glib::Object::builder()
            .property("application", app)
            .property("fullscreened", true)
            .build();

        let imp = imp::PpfWindow::from_obj(&obj);
        imp.config.replace(config.clone());

        let media_provider = Arc::new(Mutex::new(MediaProvider::new(config.clone())));
        imp.media_provider.replace(media_provider);

        obj.start_worker_thread();

        obj
    }

    #[template_callback]
    fn handle_play_pause_toggled(&self, is_paused: bool) {
        println!("Play/Pause button toggled: {}", is_paused);
        match is_paused {
            true => {
                self.imp().photo_location_label.show();
            }
            false => {
                self.imp().photo_location_label.hide();
            }
        }

        self.imp()
            .media_provider
            .clone()
            .borrow()
            .lock()
            .unwrap()
            .paused = is_paused;
    }

    pub async fn start_timer(&self) {
        loop {
            timeout_future_seconds(1).await;

            let now = gtk::glib::DateTime::now_local().unwrap();
            let time_str = now.format("%H:%M").unwrap();
            let date_str = now.format("%A, %d %B %Y").unwrap();

            self.imp()
                .time_label
                .set_text(time_str.to_string().as_str());
            self.imp()
                .date_label
                .set_text(date_str.to_string().as_str());
        }
    }

    pub fn start_worker_thread(&self) {
        let media_provider = self.imp().media_provider.clone();
        let (media_sender, media_receiver) = MainContext::channel::<MediaMessage>(PRIORITY_DEFAULT);
        MediaProvider::start_worker(media_provider.borrow().clone(), media_sender);

        let this = self;

        media_receiver.attach(None, clone!(@weak this => @default-return Continue(false), move |photo_obj| {
          match photo_obj {
            MediaMessage::Photo { photo, photo_data, address } => {
              let span = span!(Level::TRACE, "show_picture_thread");
              let _enter = span.enter();

              debug!("Recreating photo");

              this.imp().picture.set_pixbuf(Some(photo_data.pixbuf.as_ref()));

              debug!("Done setting photo on screen");

              let mut location_found = false;
              let mut date_found = false;

              match address {
                  Ok(a) => {
                      location_found = true;
                      this.imp().location_label.set_text(format!("{}", a).as_str());
                  },
                  Err(e) => {
                      this.imp().location_label.set_text("");
                      println!("Failed to get reverse geocode response, {}", e);
                  }
              }

              if let Media::Photo { path, orientation: _, location: _, date } = photo {
                  if let Some(string_date) = date {
                      date_found = true;
                      this.imp().photo_date_label.set_text(string_date.as_str());
                  } else {
                      this.imp().photo_date_label.set_text("");
                  }

                  this.imp().photo_location_label.set_text(format!("{}", path.to_str().unwrap()).as_str());
              }

              if location_found || date_found {
                  this.imp().location_box.show();
              } else {
                  this.imp().location_box.hide();
              }
              debug!("Done setting everything on screen");
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

                  this.imp().picture.set_paintable(Some(&media_file));
                  media_file.play();
              }
            }
          }
          Continue(true)
      }));

        let config = self.imp().config.borrow();
        let mqtt_host = config.mqtt_host.clone();
        let mqtt_topic = config.mqtt_topic.clone();
        let mqtt_topic_clone = mqtt_topic.clone();
        if config.mqtt {
            let (sender, receiver) = MainContext::channel::<bool>(PRIORITY_DEFAULT);
            let (mut client, mut eventloop) = PpfWindow::connect_mqtt_async(mqtt_host.clone());

            spawn_tokio!(async move {
                PpfWindow::subscribe_mqtt_async(&mut client, &mqtt_topic).await;
            });

            spawn_tokio!(async move {
                while let Ok(notification) = eventloop.poll().await {
                    println!("Event = {notification:?}");
                    match notification {
                        Incoming(Publish(notification)) => {
                            if notification.topic == mqtt_topic_clone {
                                let payload =
                                    String::from_utf8(notification.payload[..].to_vec()).unwrap();
                                let power = if payload == "1" { "0" } else { "1" };
                                println!("Received MQTT notification {}", payload);
                                let err = run_script::run_script!(format!(
                                    "echo {} | sudo tee /sys/class/backlight/*/bl_power",
                                    power
                                ));
                                if err.is_err() {
                                    println!("Failed to switch lcd display");
                                }

                                if payload == "1" {
                                    sender.send(false).unwrap();
                                } else {
                                    sender.send(true).unwrap();
                                }
                            }
                        }
                        _ => {}
                    }
                }
            });

            receiver.attach(
                None,
                clone!(@weak this => @default-return Continue(false), move |is_paused| {
                  this.imp().play_pause_button.set_property("is-paused", is_paused);

                  if !is_paused {
                    this.imp().photo_location_label.hide();
                  }
                  Continue(true)
                }),
            );
        }
    }

    fn connect_mqtt_async(mqtt_host: String) -> (rumqttc::AsyncClient, rumqttc::EventLoop) {
        let mut mqtt_options = MqttOptions::new("pi-photo-frame-2", mqtt_host, 1883);
        mqtt_options.set_keep_alive(Duration::from_secs(5));
        // mqtt_options.set_clean_session(false);

        let (client, eventloop) = rumqttc::AsyncClient::new(mqtt_options, 10);

        return (client, eventloop)
    }

    async fn subscribe_mqtt_async(client: &mut rumqttc::AsyncClient, mqtt_topic: &String) {
        client.subscribe(mqtt_topic, QoS::AtMostOnce).await.unwrap();
    }
}
