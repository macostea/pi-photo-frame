use gtk::{gio, glib, glib::subclass::Signal, prelude::*, subclass::prelude::*, CompositeTemplate};
use once_cell::sync::Lazy;
use std::cell::Cell;

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(file = "../../data/gtk/play_pause_button.ui")]
    pub struct PpfPlayPauseButton {
        pub(super) is_paused: Cell<bool>,

        #[template_child]
        pub(super) play_pause_image: TemplateChild<gtk::Image>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PpfPlayPauseButton {
        const NAME: &'static str = "PpfPlayPauseButton";
        type Type = super::PpfPlayPauseButton;
        type ParentType = gtk::Button;

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }
    }

    impl ObjectImpl for PpfPlayPauseButton {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("toggled")
                    .param_types([bool::static_type()])
                    .build()]
            });
            SIGNALS.as_ref()
        }
    }
    impl ButtonImpl for PpfPlayPauseButton {
        fn clicked(&self) {
            if self.is_paused.get() {
                self.is_paused.set(false);
                self.play_pause_image
                    .set_from_resource(Some("/com/mcostea/PiPhotoFrame/pause-icon.svg"));
            } else {
                self.is_paused.set(true);
                self.play_pause_image
                    .set_from_resource(Some("/com/mcostea/PiPhotoFrame/play-icon.svg"));
            }

            self.obj()
                .emit_by_name::<()>("toggled", &[&self.is_paused.get()]);
        }
    }

    impl WidgetImpl for PpfPlayPauseButton {}
}

glib::wrapper! {
    pub struct PpfPlayPauseButton(ObjectSubclass<imp::PpfPlayPauseButton>)
        @extends gtk::Button, gtk::Widget,
        @implements gio::ActionMap, gio::ActionGroup;
}

#[gtk::template_callbacks]
impl PpfPlayPauseButton {}
