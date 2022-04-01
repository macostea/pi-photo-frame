use gtk::gdk::Display;
use gtk::{prelude::*, CssProvider, StyleContext};
use gtk::{Application, ApplicationWindow, Overlay, Picture, Box, Label};

fn main() {
    let app = Application::builder()
        .application_id("com.mcostea.photo-frame")
        .build();

    app.connect_startup(|_| load_css());
    app.connect_activate(build_ui);

    app.run();
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_data(include_bytes!("./style/style.css"));

    StyleContext::add_provider_for_display(
        &Display::default().expect("Could not connect to a display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn build_ui(app: &Application) {
    let picture = Picture::builder()
        .halign(gtk::Align::Fill)
        .valign(gtk::Align::Fill)
        .build();

    let time_label = Label::builder()
        .valign(gtk::Align::Center)
        .halign(gtk::Align::End)
        .build();
    
    time_label.set_text("20:37");
    time_label.add_css_class("time-label");

    let date_label = Label::builder()
        .valign(gtk::Align::Center)
        .halign(gtk::Align::End)
        .build();

    date_label.set_text("Friday, April 1, 2022");
    date_label.add_css_class("date-label");

    let time_box = Box::builder()
        .halign(gtk::Align::End)
        .valign(gtk::Align::Start)
        .orientation(gtk::Orientation::Vertical)
        .build();

    time_box.add_css_class("date-time-container");

    time_box.append(&date_label);
    time_box.append(&time_label);
    
    let overlay = Overlay::builder()
        .child(&picture)
        .build();

    overlay.add_overlay(&time_box);

    let pic_file = gtk::gio::File::for_path("/Users/mihaicostea/Pictures/ex7gz9ilb8951.jpg");
    picture.set_file(Some(&pic_file));

    let window = ApplicationWindow::builder()
        .application(app)
        .fullscreened(true)
        .child(&overlay)
        .build();

    window.present();
}