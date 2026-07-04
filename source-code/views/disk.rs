use gtk::prelude::*;
use gtk::{Box as GtkBox, Label, LevelBar, Orientation, ScrolledWindow};

use crate::stats::DiskInfo;

pub struct DiskView {
    pub root: GtkBox,
    pub list_box: GtkBox,
}

pub fn build() -> DiskView {
    let root = GtkBox::new(Orientation::Vertical, 12);
    root.set_margin_start(16);
    root.set_margin_end(16);
    root.set_margin_top(16);
    root.set_margin_bottom(16);

    let title = Label::new(Some("Dyski"));
    title.add_css_class("view-title");
    title.set_xalign(0.0);
    root.append(&title);

    let list_box = GtkBox::new(Orientation::Vertical, 8);
    let scroller = ScrolledWindow::builder()
        .vexpand(true)
        .child(&list_box)
        .build();
    root.append(&scroller);

    DiskView { root, list_box }
}

pub fn refresh(view: &DiskView, disks: &[DiskInfo]) {
    while let Some(child) = view.list_box.first_child() {
        view.list_box.remove(&child);
    }

    for disk in disks {
        let card = GtkBox::new(Orientation::Vertical, 4);
        card.add_css_class("stat-card");
        card.set_margin_top(4);
        card.set_margin_bottom(4);

        let header = GtkBox::new(Orientation::Horizontal, 8);
        let name = Label::new(Some(&format!(
            "{}  ({})",
            disk.mount_point, disk.name
        )));
        name.set_xalign(0.0);
        name.set_hexpand(true);
        name.add_css_class("card-title");

        let used = disk.total_bytes.saturating_sub(disk.available_bytes);
        let pct = if disk.total_bytes > 0 {
            used as f64 / disk.total_bytes as f64 * 100.0
        } else {
            0.0
        };
        let size_label = Label::new(Some(&format!(
            "{} / {} ({:.0}%)",
            human_bytes(used),
            human_bytes(disk.total_bytes),
            pct
        )));
        size_label.add_css_class("dim-label");

        header.append(&name);
        header.append(&size_label);

        let bar = LevelBar::new();
        bar.set_min_value(0.0);
        bar.set_max_value(100.0);
        bar.set_value(pct);
        bar.set_height_request(8);

        card.append(&header);
        card.append(&bar);

        if disk.is_removable {
            let removable = Label::new(Some("Nośnik wymienny"));
            removable.add_css_class("dim-label");
            removable.set_xalign(0.0);
            card.append(&removable);
        }

        view.list_box.append(&card);
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
