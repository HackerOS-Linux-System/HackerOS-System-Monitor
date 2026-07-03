mod app;
mod graph;
mod process_object;
mod stats;
mod views;

use gtk::glib;

fn main() -> glib::ExitCode {
    app::run()
}
