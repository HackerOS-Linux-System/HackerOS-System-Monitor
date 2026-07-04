use gtk::prelude::*;
use gtk::{Box as GtkBox, Grid, Label, Orientation};

use crate::graph::{Graph, ACCENT_GREEN, ACCENT_PURPLE};
use crate::stats::HISTORY_LEN;

pub struct MemoryView {
    pub root: GtkBox,
    pub ram_graph: Graph,
    pub swap_graph: Graph,
    pub ram_label: Label,
    pub swap_label: Label,
    pub breakdown_label: Label,
}

pub fn build() -> MemoryView {
    let root = GtkBox::new(Orientation::Vertical, 12);
    root.set_margin_start(16);
    root.set_margin_end(16);
    root.set_margin_top(16);
    root.set_margin_bottom(16);

    let title = Label::new(Some("Pamięć"));
    title.add_css_class("view-title");
    title.set_xalign(0.0);
    root.append(&title);

    let grid = Grid::new();
    grid.set_row_spacing(4);
    grid.set_column_spacing(4);

    let ram_title = Label::new(Some("RAM"));
    ram_title.set_xalign(0.0);
    ram_title.add_css_class("dim-label");
    let ram_label = Label::new(Some("0 / 0 GB"));
    ram_label.set_xalign(0.0);
    ram_label.add_css_class("card-value");

    let ram_graph = Graph::new(ACCENT_GREEN, Some(100.0), HISTORY_LEN);
    ram_graph.widget.set_content_height(140);

    let swap_title = Label::new(Some("Swap"));
    swap_title.set_xalign(0.0);
    swap_title.add_css_class("dim-label");
    let swap_label = Label::new(Some("0 / 0 GB"));
    swap_label.set_xalign(0.0);
    swap_label.add_css_class("card-value");

    let swap_graph = Graph::new(ACCENT_PURPLE, Some(100.0), HISTORY_LEN);
    swap_graph.widget.set_content_height(100);

    let breakdown_label = Label::new(None);
    breakdown_label.set_xalign(0.0);
    breakdown_label.add_css_class("dim-label");
    breakdown_label.set_wrap(true);

    root.append(&ram_title);
    root.append(&ram_label);
    root.append(&ram_graph.widget);
    root.append(&swap_title);
    root.append(&swap_label);
    root.append(&swap_graph.widget);
    root.append(&breakdown_label);

    let _ = grid; // reserved for future detailed breakdown layout

    MemoryView {
        root,
        ram_graph,
        swap_graph,
        ram_label,
        swap_label,
        breakdown_label,
    }
}
