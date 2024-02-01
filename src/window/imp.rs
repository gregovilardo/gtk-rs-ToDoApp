use crate::task_object::{TaskData, TaskObject};
use crate::utils::data_path;
use gio::Settings;
use glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, Button, CompositeTemplate, Entry, ListView};
use std::cell::OnceCell;
use std::cell::RefCell;
use std::fs::File;

// Object holding the state
#[derive(CompositeTemplate, Default)]
#[template(resource = "/org/gtk_rs/todo/window.ui")]
pub struct Window {
    #[template_child(id = "button")]
    pub button: TemplateChild<Button>,
    #[template_child(id = "entry")]
    pub entry: TemplateChild<Entry>,
    #[template_child(id = "tasks_list")]
    pub tasks_list: TemplateChild<ListView>,
    pub tasks: RefCell<Option<gio::ListStore>>,
    pub settings: OnceCell<Settings>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for Window {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "MyGtkAppWindow";
    type Type = super::Window;
    type ParentType = gtk::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        // klass.bind_template_callbacks();
        // Create action to remove done tasks and add to action group "win"
        klass.install_action("win.remove-done-tasks", None, |window, _, _| {
            window.remove_done_tasks();
        });
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for Window {
    fn constructed(&self) {
        // Call "constructed" on parent
        self.parent_constructed();
        self.button.add_css_class("boton");

        // Setup
        let obj = self.obj();
        obj.setup_settings();
        obj.setup_tasks();
        obj.restore_data();
        obj.setup_callbacks();
        obj.setup_factory();
        obj.setup_actions();
    }
}

// Trait shared by all widgets
impl WidgetImpl for Window {}

// Trait shared by all windows
impl WindowImpl for Window {
    fn close_request(&self) -> glib::Propagation {
        // Store task data in vector
        let backup_data: Vec<TaskData> = self
            .obj()
            .get_tasks()
            .iter::<TaskObject>()
            .filter_map(Result::ok)
            .map(|task_object| task_object.task_data())
            .collect();

        // Save state to file
        let file = File::create(data_path()).expect("Could not create json file.");
        serde_json::to_writer(file, &backup_data).expect("Could not write data to json file");

        // Pass close request on to the parent
        self.parent_close_request()
    }
}

// Trait shared by all application windows
impl ApplicationWindowImpl for Window {}
