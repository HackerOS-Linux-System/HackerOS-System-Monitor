use gtk::prelude::*;
use gtk::{Box as GtkBox, Label, Orientation, ScrolledWindow};

use crate::graph::{Graph, ACCENT_ORANGE, ACCENT_PURPLE};
use crate::stats::HISTORY_LEN;
use sysinfo::Networks;

pub struct NetworkView {
    pub root: GtkBox,
    pub rx_graph: Graph,
    pub tx_graph: Graph,
    pub rx_label: Label,
    pub tx_label: Label,
    pub iface_box: GtkBox,
}

pub fn build() -> NetworkView {
    let root = GtkBox::new(Orientation::Vertical, 12);
    root.set_margin_start(16);
    root.set_margin_end(16);
    root.set_margin_top(16);
    root.set_margin_bottom(16);

    let title = Label::new(Some("Sieć"));
    title.add_css_class("view-title");
    title.set_xalign(0.0);
    root.append(&title);

    let rx_title = Label::new(Some("Pobieranie"));
    rx_title.set_xalign(0.0);
    rx_title.add_css_class("dim-label");
    let rx_label = Label::new(Some("0 B/s"));
    rx_label.set_xalign(0.0);
    rx_label.add_css_class("card-value");
    let rx_graph = Graph::new(ACCENT_ORANGE, None, HISTORY_LEN);
    rx_graph.widget.set_content_height(100);

    let tx_title = Label::new(Some("Wysyłanie"));
    tx_title.set_xalign(0.0);
    tx_title.add_css_class("dim-label");
    let tx_label = Label::new(Some("0 B/s"));
    tx_label.set_xalign(0.0);
    tx_label.add_css_class("card-value");
    let tx_graph = Graph::new(ACCENT_PURPLE, None, HISTORY_LEN);
    tx_graph.widget.set_content_height(100);

    root.append(&rx_title);
    root.append(&rx_label);
    root.append(&rx_graph.widget);
    root.append(&tx_title);
    root.append(&tx_label);
    root.append(&tx_graph.widget);

    let iface_title = Label::new(Some("Interfejsy"));
    iface_title.add_css_class("view-title");
    iface_title.set_xalign(0.0);
    iface_title.set_margin_top(8);
    root.append(&iface_title);

    let iface_box = GtkBox::new(Orientation::Vertical, 6);
    let scroller = ScrolledWindow::builder()
        .vexpand(true)
        .child(&iface_box)
        .build();
    root.append(&scroller);

    NetworkView {
        root,
        rx_graph,
        tx_graph,
        rx_label,
        tx_label,
        iface_box,
    }
}

pub fn refresh_interfaces(view: &NetworkView, networks: &Networks) {
    while let Some(child) = view.iface_box.first_child() {
        view.iface_box.remove(&child);
    }
    for (name, data) in networks.list() {
        let row = GtkBox::new(Orientation::Horizontal, 8);
        row.add_css_class("stat-card");
        let name_label = Label::new(Some(name));
        name_label.set_xalign(0.0);
        name_label.set_hexpand(true);
        let totals = Label::new(Some(&format!(
            "↓ {}   ↑ {}",
            human_bytes(data.total_received()),
            human_bytes(data.total_transmitted())
        )));
        totals.add_css_class("dim-label");
        row.append(&name_label);
        row.append(&totals);
        view.iface_box.append(&row);
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
