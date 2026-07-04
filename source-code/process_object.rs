use std::cell::{Cell, RefCell};

use glib::subclass::prelude::ObjectSubclassIsExt;
use glib::Object;

mod imp {
    use super::*;
    use glib::subclass::prelude::*;

    #[derive(Default)]
    pub struct ProcessObject {
        pub pid: Cell<u32>,
        pub name: RefCell<String>,
        pub cpu_usage: Cell<f32>,
        pub memory_bytes: Cell<u64>,
        pub status: RefCell<String>,
        pub user: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProcessObject {
        const NAME: &'static str = "MissionMonitorProcessObject";
        type Type = super::ProcessObject;
    }

    impl ObjectImpl for ProcessObject {}
}

glib::wrapper! {
    pub struct ProcessObject(ObjectSubclass<imp::ProcessObject>);
}

impl ProcessObject {
    pub fn new(pid: u32, name: &str, cpu_usage: f32, memory_bytes: u64, status: &str, user: &str) -> Self {
        let obj: Self = Object::builder().build();
        let imp = obj.imp();
        imp.pid.set(pid);
        *imp.name.borrow_mut() = name.to_string();
        imp.cpu_usage.set(cpu_usage);
        imp.memory_bytes.set(memory_bytes);
        *imp.status.borrow_mut() = status.to_string();
        *imp.user.borrow_mut() = user.to_string();
        obj
    }

    pub fn pid(&self) -> u32 {
        self.imp().pid.get()
    }

    pub fn name(&self) -> String {
        self.imp().name.borrow().clone()
    }

    pub fn cpu_usage(&self) -> f32 {
        self.imp().cpu_usage.get()
    }

    pub fn memory_bytes(&self) -> u64 {
        self.imp().memory_bytes.get()
    }

    pub fn status(&self) -> String {
        self.imp().status.borrow().clone()
    }

    pub fn user(&self) -> String {
        self.imp().user.borrow().clone()
    }
}

impl Default for ProcessObject {
    fn default() -> Self {
        Self::new(0, "", 0.0, 0, "", "")
    }
}
