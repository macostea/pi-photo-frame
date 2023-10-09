use gtk::glib::Properties;
use gtk::{gio, glib, glib::subclass::Signal, prelude::*, subclass::prelude::*, CompositeTemplate};
use once_cell::sync::Lazy;
use std::cell::Cell;

mod imp {
    use gtk::glib::{ParamSpec, Value};

    use super::*;

    #[derive(Default, Debug, CompositeTemplate, Properties)]
    #[template(file = "../../data/gtk/play_pause_button.ui")]
    #[properties(wrapper_type = super::PpfPlayPauseButton)]
    pub struct PpfPlayPauseButton {
        #[property(get, set)]
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
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            obj.connect_notify_local(Some("is-paused"), move |obj, _| {
                let is_paused = obj.imp().is_paused.get();
                if !is_paused {
                    obj.imp()
                        .play_pause_image
                        .set_from_resource(Some("/com/mcostea/PiPhotoFrame/pause-icon.svg"));
                } else {
                    obj.imp()
                        .play_pause_image
                        .set_from_resource(Some("/com/mcostea/PiPhotoFrame/play-icon.svg"));
                }
            });
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("toggled")
                    .param_types([bool::static_type()])
                    .build()]
            });
            SIGNALS.as_ref()
        }

        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }
        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "is-paused" => {
                    let is_paused = value.get().unwrap();
                    self.is_paused.set(is_paused);
                    self.obj()
                        .emit_by_name::<()>("toggled", &[&self.is_paused.get()]);
                }
                _ => unimplemented!(),
            }
        }
        fn property(&self, id: usize, pspec: &ParamSpec) -> Value {
            self.derived_property(id, pspec)
        }
    }

    impl ButtonImpl for PpfPlayPauseButton {
        fn clicked(&self) {
            self.obj().set_property("is-paused", !self.is_paused.get());

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
