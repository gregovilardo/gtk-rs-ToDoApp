mod imp;
use crate::utils::data_path;
use gio::Settings;
use glib::{clone, Object};
use gtk::subclass::prelude::*;
use gtk::{
    gio, glib, Application, CustomFilter, FilterListModel, NoSelection, SignalListItemFactory,
};
use gtk::{prelude::*, ListItem};
use std::fs::File;

use crate::task_object::{TaskData, TaskObject};
use crate::task_row::TaskRow;
use crate::APP_ID;

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &Application) -> Self {
        // Create new window
        Object::builder().property("application", app).build()
    }

    fn get_tasks(&self) -> gio::ListStore {
        // Get state
        self.imp()
            .tasks
            .borrow()
            .clone()
            .expect("Could not get current tasks.")
    }

    fn setup_tasks(&self) {
        // Create new model
        let model = gio::ListStore::new::<TaskObject>();

        // Get state and set model
        self.imp().tasks.replace(Some(model));

        // Wrap model with filter and selection and pass it to the list view
        let filter_model = FilterListModel::new(Some(self.get_tasks()), self.filter());
        let selection_model = NoSelection::new(Some(filter_model.clone()));
        self.imp().tasks_list.set_model(Some(&selection_model));

        self.settings().connect_changed(
            Some("filter"),
            clone!(@weak self as window, @weak filter_model => move |_, _| {
                filter_model.set_filter(window.filter().as_ref());
            }),
        );
    }

    fn new_task(&self) {
        // Get content from entry and clear it
        let buffer = self.imp().entry.buffer();
        let content = buffer.text().to_string();
        if content.is_empty() {
            return;
        }
        buffer.set_text("");

        // Add new task to model
        let task = TaskObject::new(false, content);
        self.get_tasks().append(&task);
    }

    fn setup_callbacks(&self) {
        // Setup callback for activation of the entry
        // self.imp().entry.connect_activate(move || {
        //     self.new_task();
        // });
        self.imp()
            .button
            .connect_clicked(clone!(@weak self as window => move |_| {
                window.new_task();
            }));

        self.imp()
            .entry
            .connect_activate(clone!(@weak self as window => move |_| {
                window.new_task();
            }));

        // Setup callback for clicking (and the releasing) the icon of the entry
        // self.imp()
        //     .entry
        //     .connect_icon_release(clone!(@weak self as window => move |_,_| {
        //         window.new_task();
        //     }));
    }

    fn setup_settings(&self) {
        let settings = Settings::new(APP_ID);
        self.imp()
            .settings
            .set(settings)
            .expect("`settings` should not be set before calling `setup_settings`.");
    }

    fn restore_data(&self) {
        if let Ok(file) = File::open(data_path()) {
            // Deserialize data from file to vector
            let backup_data: Vec<TaskData> = serde_json::from_reader(file)
                .expect("It should be possible to read `backup_data` from the json file.");

            // Convert `Vec<TaskData>` to `Vec<TaskObject>`
            let task_objects: Vec<TaskObject> = backup_data
                .into_iter()
                .map(TaskObject::from_task_data)
                .collect();

            // Insert restored objects into model
            self.get_tasks().extend_from_slice(&task_objects);
        }
    }

    fn settings(&self) -> &Settings {
        self.imp()
            .settings
            .get()
            .expect("`settings` should be set in `setup_settings`.")
    }

    fn setup_actions(&self) {
        // Create action from key "filter" and add to action group "win"
        let action_filter = self.settings().create_action("filter");
        self.add_action(&action_filter);
    }

    fn remove_done_tasks(&self) {
        let tasks = self.get_tasks();
        let mut position = 0;
        while let Some(item) = tasks.item(position) {
            // Get `TaskObject` from `glib::Object`
            let task_object = item
                .downcast_ref::<TaskObject>()
                .expect("The object needs to be of type `TaskObject`.");

            if task_object.is_completed() {
                tasks.remove(position);
            } else {
                position += 1;
            }
        }
    }

    fn filter(&self) -> Option<CustomFilter> {
        // Get filter_state from settings
        let filter_state: String = self.settings().get("filter");

        // Create custom filters
        let filter_open = CustomFilter::new(|obj| {
            // Get `TaskObject` from `glib::Object`
            let task_object = obj
                .downcast_ref::<TaskObject>()
                .expect("The object needs to be of type `TaskObject`.");

            // Only allow completed tasks
            !task_object.is_completed()
        });
        let filter_done = CustomFilter::new(|obj| {
            // Get `TaskObject` from `glib::Object`
            let task_object = obj
                .downcast_ref::<TaskObject>()
                .expect("The object needs to be of type `TaskObject`.");

            // Only allow done tasks
            task_object.is_completed()
        });

        // Return the correct filter
        match filter_state.as_str() {
            "All" => None,
            "Open" => Some(filter_open),
            "Done" => Some(filter_done),
            _ => unreachable!(),
        }
    }

    fn setup_factory(&self) {
        // Create a new factory
        let factory = SignalListItemFactory::new();

        // Create an empty `TaskRow` during setup
        factory.connect_setup(move |_, list_item| {
            // Create `TaskRow`
            let task_row = TaskRow::new();
            list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem")
                .set_child(Some(&task_row));
        });

        // Tell factory how to bind `TaskRow` to a `TaskObject`
        factory.connect_bind(move |_, list_item| {
            // Get `TaskObject` from `ListItem`
            let task_object = list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem")
                .item()
                .and_downcast::<TaskObject>()
                .expect("The item has to be an `TaskObject`.");

            // Get `TaskRow` from `ListItem`
            let task_row = list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem")
                .child()
                .and_downcast::<TaskRow>()
                .expect("The child has to be a `TaskRow`.");

            task_row.bind(&task_object);
        });

        // Tell factory how to unbind `TaskRow` from `TaskObject`
        factory.connect_unbind(move |_, list_item| {
            // Get `TaskRow` from `ListItem`
            let task_row = list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem")
                .child()
                .and_downcast::<TaskRow>()
                .expect("The child has to be a `TaskRow`.");

            task_row.unbind();
        });

        // Set the factory of the list view
        self.imp().tasks_list.set_factory(Some(&factory));
    }
}
