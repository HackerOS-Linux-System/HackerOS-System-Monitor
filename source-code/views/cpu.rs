use gtk::prelude::*;
use gtk::{Box as GtkBox, FlowBox, Label, Orientation, ScrolledWindow, SelectionMode};

use crate::graph::{Graph, ACCENT_BLUE};
use crate::stats::HISTORY_LEN;

pub struct CpuView {
    pub root: GtkBox,
    pub overall_graph: Graph,
    pub overall_label: Label,
    pub info_label: Label,
    pub core_graphs: Vec<Graph>,
}

pub fn build(logical_cores: usize) -> CpuView {
    let root = GtkBox::new(Orientation::Vertical, 12);
    root.set_margin_start(16);
    root.set_margin_end(16);
    root.set_margin_top(16);
    root.set_margin_bottom(16);

    let header = GtkBox::new(Orientation::Horizontal, 8);
    let title = Label::new(Some("Procesor"));
    title.add_css_class("view-title");
    title.set_xalign(0.0);
    title.set_hexpand(true);
    let overall_label = Label::new(Some("0%"));
    overall_label.add_css_class("card-value");
    header.append(&title);
    header.append(&overall_label);
    root.append(&header);

    let overall_graph = Graph::new(ACCENT_BLUE, Some(100.0), HISTORY_LEN);
    overall_graph.widget.set_content_height(140);
    root.append(&overall_graph.widget);

    let info_label = Label::new(None);
    info_label.set_xalign(0.0);
    info_label.add_css_class("dim-label");
    root.append(&info_label);

    let cores_title = Label::new(Some("Rdzenie"));
    cores_title.add_css_class("view-title");
    cores_title.set_xalign(0.0);
    cores_title.set_margin_top(8);
    root.append(&cores_title);

    let flow = FlowBox::new();
    flow.set_selection_mode(SelectionMode::None);
    flow.set_max_children_per_line(4);
    flow.set_min_children_per_line(2);
    flow.set_row_spacing(8);
    flow.set_column_spacing(8);
    flow.set_homogeneous(true);

    let mut core_graphs = Vec::with_capacity(logical_cores);
    for i in 0..logical_cores {
        let g = Graph::new(ACCENT_BLUE, Some(100.0), HISTORY_LEN);
        g.widget.set_content_height(48);
        g.widget.set_size_request(140, 48);

        let cell = GtkBox::new(Orientation::Vertical, 2);
        cell.add_css_class("core-cell");
        let label = Label::new(Some(&format!("CPU {i}")));
        label.set_xalign(0.0);
        label.add_css_class("dim-label");
        cell.append(&label);
        cell.append(&g.widget);
        flow.insert(&cell, -1);
        core_graphs.push(g);
    }

    let scroller = ScrolledWindow::builder()
        .vexpand(true)
        .child(&flow)
        .build();
    root.append(&scroller);

    CpuView {
        root,
        overall_graph,
        overall_label,
        info_label,
        core_graphs,
    }
}
