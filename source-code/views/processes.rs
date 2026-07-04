use gtk::glib;
use gtk::prelude::*;
use gtk::{
    ColumnView, ColumnViewColumn, Label, ListItem, NoSelection, ScrolledWindow,
    SignalListItemFactory, SortListModel, Box as GtkBox, Orientation, SearchEntry,
    FilterListModel, CustomFilter,
};
use gio::ListStore;

use crate::process_object::ProcessObject;

pub struct ProcessesView {
    pub root: GtkBox,
    pub store: ListStore,
    pub filter: CustomFilter,
}

fn text_column(title: &str, expand: bool, bind: impl Fn(&ProcessObject) -> String + 'static) -> ColumnViewColumn {
    let factory = SignalListItemFactory::new();
    factory.connect_setup(move |_, list_item| {
        let label = Label::new(None);
        label.set_xalign(0.0);
        label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        list_item
            .downcast_ref::<ListItem>()
            .unwrap()
            .set_child(Some(&label));
    });
    let bind = std::rc::Rc::new(bind);
    factory.connect_bind(move |_, list_item| {
        let list_item = list_item.downcast_ref::<ListItem>().unwrap();
        let obj = list_item.item().and_downcast::<ProcessObject>().unwrap();
        let label = list_item.child().and_downcast::<Label>().unwrap();
        label.set_text(&bind(&obj));
    });
    let col = ColumnViewColumn::new(Some(title), Some(factory));
    col.set_expand(expand);
    col.set_resizable(true);
    col
}

pub fn build() -> ProcessesView {
    let store = ListStore::new::<ProcessObject>();

    let filter = CustomFilter::new(|_obj| true);
    let filter_model = FilterListModel::new(Some(store.clone()), Some(filter.clone()));

    let sorter_model = SortListModel::new(Some(filter_model), None::<gtk::Sorter>);
    let selection = NoSelection::new(Some(sorter_model.clone()));

    let column_view = ColumnView::new(Some(selection));
    column_view.add_css_class("data-table");
    column_view.set_show_row_separators(true);

    let name_col = text_column("Nazwa procesu", true, |p| p.name());
    let pid_col = text_column("PID", false, |p| p.pid().to_string());
    let user_col = text_column("Użytkownik", false, |p| p.user());
    let cpu_col = text_column("CPU %", false, |p| format!("{:.1}", p.cpu_usage()));
    let mem_col = text_column("Pamięć", false, |p| human_bytes(p.memory_bytes()));
    let status_col = text_column("Status", false, |p| p.status());

    column_view.append_column(&name_col);
    column_view.append_column(&pid_col);
    column_view.append_column(&user_col);
    column_view.append_column(&cpu_col);
    column_view.append_column(&mem_col);
    column_view.append_column(&status_col);

    // Sort by CPU usage descending by default.
    column_view.sort_by_column(Some(&cpu_col), gtk::SortType::Descending);

    let sorter = column_view.sorter();
    sorter_model.set_sorter(sorter.as_ref());

    let scroller = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .child(&column_view)
        .build();

    let search = SearchEntry::builder()
        .placeholder_text("Filtruj procesy…")
        .margin_start(12)
        .margin_end(12)
        .margin_top(8)
        .margin_bottom(4)
        .build();

    {
        let filter = filter.clone();
        search.connect_search_changed(move |entry| {
            let text = entry.text().to_string().to_lowercase();
            filter.set_filter_func(move |obj| {
                if text.is_empty() {
                    return true;
                }
                let p = obj.downcast_ref::<ProcessObject>().unwrap();
                p.name().to_lowercase().contains(&text) || p.pid().to_string().contains(&text)
            });
        });
    }

    let root = GtkBox::new(Orientation::Vertical, 0);
    root.append(&search);
    root.append(&scroller);

    ProcessesView { root, store, filter }
}

/// Rebuild the store from a fresh process snapshot. Cheap enough at 1 Hz
/// for typical process counts (a few hundred).
pub fn refresh(view: &ProcessesView, sys: &sysinfo::System) {
    let mut items: Vec<ProcessObject> = Vec::with_capacity(sys.processes().len());
    for (pid, process) in sys.processes() {
        let user = process
            .user_id()
            .map(|_| String::new())
            .unwrap_or_default();
        items.push(ProcessObject::new(
            pid.as_u32(),
            process.name(),
            process.cpu_usage(),
            process.memory(),
            &format!("{:?}", process.status()),
            &user,
        ));
    }
    view.store.remove_all();
    for item in items {
        view.store.append(&item);
    }
}

fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{:.1} {}", value, UNITS[unit])
}

// re-export for main.rs glue
use gtk::gio;
use glib::object::Cast;
