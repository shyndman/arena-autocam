use glib::ObjectExt;
use rand::Rng;

glib::wrapper! {
    pub struct Rect(ObjectSubclass<imp::Rect>) @extends gst::Object;
}

impl Rect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        glib::Object::new(&[("x", &x), ("y", &y), ("width", &width), ("height", &height)])
    }

    pub fn x(&self) -> f64 {
        self.property("x")
    }

    pub fn y(&self) -> f64 {
        self.property("y")
    }

    pub fn width(&self) -> f64 {
        self.property("width")
    }

    pub fn height(&self) -> f64 {
        self.property("height")
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
}

mod imp {
    use std::sync::Mutex;

    use gst::glib::prelude::*;
    use gst::subclass::prelude::*;
    use once_cell::sync::Lazy;

    #[derive(Debug, Default)]
    pub struct Rect {
        props_storage: Mutex<PropsStorage>,
    }

    // This trait registers our type with the GObject object system and
    // provides the entry points for creating a new instance and setting
    // up the class data
    #[glib::object_subclass]
    impl ObjectSubclass for Rect {
        const NAME: &'static str = "Rect";
        type Type = super::Rect;
        type ParentType = gst::Object;
    }

    impl ObjectImpl for Rect {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecDouble::builder("x")
                        .default_value(0.0)
                        .build(),
                    glib::ParamSpecDouble::builder("y")
                        .default_value(0.0)
                        .build(),
                    glib::ParamSpecDouble::builder("width")
                        .default_value(0.0)
                        .build(),
                    glib::ParamSpecDouble::builder("height")
                        .default_value(0.0)
                        .build(),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let props_guard = self.props_storage.lock().unwrap();
            match pspec.name() {
                "x" => props_guard.x.to_value(),
                "y" => props_guard.y.to_value(),
                "width" => props_guard.width.to_value(),
                "height" => props_guard.height.to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let mut props_guard = self.props_storage.lock().unwrap();
            match pspec.name() {
                "x" => props_guard.x = value.get().expect("type checked upstream"),
                "y" => props_guard.y = value.get().expect("type checked upstream"),
                "width" => props_guard.width = value.get().expect("type checked upstream"),
                "height" => props_guard.height = value.get().expect("type checked upstream"),
                _ => unimplemented!(),
            }
        }
    }

    impl GstObjectImpl for Rect {}

    #[derive(Debug, Default)]
    struct PropsStorage {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    }
}

pub fn random_fraction_rect() -> Rect {
    let mut r = rand::thread_rng();
    let w = r.gen_range(0.0_f64..=1.0).clamp(0.03, 0.3);
    let x = r.gen_range(0.0..(1.0 - w));
    let h = r.gen_range(0.0_f64..=1.0).clamp(0.03, 0.3);
    let y = r.gen_range(0.0..(1.0 - h));
    Rect::new(x, y, w, h)
}
