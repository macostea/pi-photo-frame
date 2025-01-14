use crate::{
    config,
    photo::provider::{Config, FailedFiles},
    window::PpfWindow,
};
use gtk::gdk::Display;
use gtk::CssProvider;
use gtk::STYLE_PROVIDER_PRIORITY_APPLICATION;
use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Default, Debug)]
    pub struct PpfApplication {
        pub(super) config: RefCell<Config>,
        pub(super) failed_files: RefCell<FailedFiles>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PpfApplication {
        const NAME: &'static str = "PpfApplication";
        type Type = super::PpfApplication;
        type ParentType = gtk::Application;
    }

    impl ObjectImpl for PpfApplication {}
    impl ApplicationImpl for PpfApplication {
        fn activate(&self) {
            let application = self.obj();
            let window = PpfWindow::new(
                &*application,
                self.config.borrow().clone(),
                self.failed_files.borrow().clone(),
            );
            window.present();
        }

        fn startup(&self) {
            self.parent_startup();

            let provider = CssProvider::new();
            provider.load_from_resource("/com/mcostea/PiPhotoFrame/style.css");

            gtk::style_context_add_provider_for_display(
                &Display::default().expect("Could not connect to a display"),
                &provider,
                STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    }

    impl GtkApplicationImpl for PpfApplication {}
}

glib::wrapper! {
  pub struct PpfApplication(ObjectSubclass<imp::PpfApplication>)
    @extends gtk::Application, gio::Application,
    @implements gio::ActionGroup, gio::ActionMap;
}

impl PpfApplication {
    pub fn new(config: Config, failed_files: FailedFiles) -> Self {
        let obj: PpfApplication = glib::Object::builder()
            .property("application-id", config::APP_ID)
            .property("resource-base-path", "/com/mcostea/PiPhotoFrame")
            .build();
        let imp = imp::PpfApplication::from_obj(&obj);
        imp.config.replace(config);
        imp.failed_files.replace(failed_files);

        obj
    }
}
