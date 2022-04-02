use gtk::{prelude::*, Picture, Label, Box, Overlay, ApplicationWindow, Application};

pub struct MainView {
    pub window: Option<ApplicationWindow>,
    pub picture: Option<Picture>,
    pub time_label: Option<Label>,
    pub date_label: Option<Label>,
}

impl MainView {
    pub fn new() -> Self {
        MainView { window: None, picture: None, time_label: None, date_label: None }
    }
    
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
        
        let overlay = Overlay::builder()
            .child(self.picture.as_ref().unwrap())
            .build();
    
        overlay.add_overlay(&time_box);
        
        self.window = Some(ApplicationWindow::builder()
            .application(app)
            .fullscreened(true)
            .child(&overlay)
            .build());
    
        self.window.as_ref().unwrap().present();
    }
}

