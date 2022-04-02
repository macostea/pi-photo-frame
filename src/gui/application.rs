use std::cell::RefCell;
use std::rc::Rc;

use gtk::glib::{self, timeout_future_seconds};
use gtk::glib::{MainContext, clone};
use gtk::{prelude::*, CssProvider, StyleContext, Application};
use gtk::gdk::Display;
use serde::Deserialize;

use crate::photo::PhotoProvider;

use super::main_view::MainView;

#[derive(Deserialize, Default)]
pub struct Config {
    paths: Vec<String>,
    transition_time: u32,
}

pub struct App {
    gtk_application: Option<Application>,
    config: Config,
    main_view: Rc<RefCell<MainView>>,
    photo_provider: Rc<PhotoProvider>,
}

impl App {
    pub fn new() -> Self {
        App {
            gtk_application: None,
            config: Default::default(),
            main_view: Rc::new(RefCell::new(MainView::new())),
            photo_provider: Rc::new(PhotoProvider::default()),
        }
    }

    pub fn build_application(&mut self, config: Config) {
        self.config = config;

        let photo_provider = PhotoProvider::new(self.config.paths.clone());
        self.photo_provider = Rc::new(photo_provider);

        let app = Application::builder()
        .application_id("com.mcostea.photo-frame")
        .build();

        let main_view = self.main_view.clone();

        let photo_provider = self.photo_provider.clone();
        let timeout = self.config.transition_time;
    
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

                    let photo = photo_provider.get_photo();
                    if let Ok(photo) = photo {
                        let file = gtk::gio::File::for_path(photo.to_str().unwrap());
                        picture.set_file(Some(&file));
                    } else {
                        println!("Error getting photo, {}", photo.unwrap_err());
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
}




