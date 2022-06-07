use std::path::Path;

use gtk::{prelude::*, Picture, Label, Box, Overlay, ApplicationWindow, Application, Button, Image, gdk_pixbuf::Pixbuf};

#[derive(Default)]
pub struct MainView {
    pub window: Option<ApplicationWindow>,
    pub overlay: Option<Overlay>,
    pub picture: Option<Picture>,
    pub time_label: Option<Label>,
    pub date_label: Option<Label>,
    pub location_box: Option<Box>,
    pub location_label: Option<Label>,
    pub photo_date_label: Option<Label>,
    pub play_pause_box: Option<Box>,
    pub play_pause_button: Option<Button>,
    pub play_image: Option<Image>,
    pub pause_image: Option<Image>,
    pub photo_location_label: Option<Label>,
}

impl MainView {
    pub fn build_main_view(&mut self, app: &Application) {
        self.picture = Some(Picture::builder()
            .halign(gtk::Align::Fill)
            .valign(gtk::Align::Fill)
            .build());
    
        self.time_label = Some(Label::builder()
            .valign(gtk::Align::Center)
            .halign(gtk::Align::End)
            .build());
        
        self.time_label.as_ref().unwrap().set_text("20:37");
        self.time_label.as_ref().unwrap().add_css_class("time-label");
    
        self.date_label = Some(Label::builder()
            .valign(gtk::Align::Center)
            .halign(gtk::Align::End)
            .build());
    
        self.date_label.as_ref().unwrap().set_text("Friday, April 1, 2022");
        self.date_label.as_ref().unwrap().add_css_class("date-label");
    
        let time_box = Box::builder()
            .halign(gtk::Align::End)
            .valign(gtk::Align::Start)
            .orientation(gtk::Orientation::Vertical)
            .build();
    
        time_box.add_css_class("date-time-container");
    
        time_box.append(self.date_label.as_ref().unwrap());
        time_box.append(self.time_label.as_ref().unwrap());

        self.location_label = Some(Label::builder()
            .halign(gtk::Align::End)
            .build());
        
        self.location_label.as_ref().unwrap().add_css_class("location-label");

        self.photo_date_label = Some(Label::builder()
            .halign(gtk::Align::End)
            .build());

        self.photo_date_label.as_ref().unwrap().add_css_class("photo-date-label");

        self.location_box = Some(Box::builder()
            .halign(gtk::Align::End)
            .valign(gtk::Align::End)
            .orientation(gtk::Orientation::Vertical)
            .build());

        self.location_box.as_ref().unwrap().add_css_class("location-container");

        self.location_box.as_ref().unwrap().append(self.location_label.as_ref().unwrap());
        self.location_box.as_ref().unwrap().append(self.photo_date_label.as_ref().unwrap());

        let mut play_path = Path::new("resources/play-icon.svg");
        if !play_path.exists() {
            play_path = Path::new("/usr/local/lib/pi-photo-frame/resources/play-icon.svg");
        }
        let play_pixbuf = Pixbuf::from_file(play_path);
        self.play_image = Some(Image::from_pixbuf(play_pixbuf.ok().as_ref()));

        let mut pause_path = Path::new("resources/pause-icon.svg");
        if !pause_path.exists() {
            pause_path = Path::new("/usr/local/lib/pi-photo-frame/resources/pause-icon.svg");
        }
        let pause_pixbuf = Pixbuf::from_file(pause_path);
        self.pause_image = Some(Image::from_pixbuf(pause_pixbuf.ok().as_ref()));

        self.play_pause_button = Some(Button::builder()
            .child(self.pause_image.as_ref().unwrap())
            .valign(gtk::Align::Center)
            .halign(gtk::Align::Start)
            .build());

        self.photo_location_label = Some(Label::builder()
            .halign(gtk::Align::Start)
            .build());

        self.photo_location_label.as_ref().unwrap().add_css_class("photo-location-label");
        self.photo_location_label.as_ref().unwrap().hide();

        self.play_pause_box = Some(Box::builder()
            .valign(gtk::Align::Start)
            .halign(gtk::Align::Start)
            .orientation(gtk::Orientation::Vertical)
            .build());

        self.play_pause_box.as_ref().unwrap().add_css_class("paused-container");
        self.play_pause_box.as_ref().unwrap().append(self.play_pause_button.as_ref().unwrap());
        self.play_pause_box.as_ref().unwrap().append(self.photo_location_label.as_ref().unwrap());

        self.overlay = Some(Overlay::builder()
            .child(self.picture.as_ref().unwrap())
            .build());

        self.overlay.as_ref().unwrap().add_overlay(&time_box);
        self.overlay.as_ref().unwrap().add_overlay(self.location_box.as_ref().unwrap());
        self.overlay.as_ref().unwrap().add_overlay(self.play_pause_box.as_ref().unwrap());
        
        self.window = Some(ApplicationWindow::builder()
            .application(app)
            .fullscreened(true)
            .child(self.overlay.as_ref().unwrap())
            .build());
    
        self.window.as_ref().unwrap().present();
    }
}

