use gtk::prelude::*;
use gtk::{Box as GtkBox, Grid, Label, Orientation};

use crate::graph::{Graph, ACCENT_BLUE, ACCENT_GREEN, ACCENT_ORANGE, ACCENT_PURPLE};
use crate::stats::HISTORY_LEN;

pub struct OverviewView {
    pub root: GtkBox,
    pub cpu_graph: Graph,
    pub mem_graph: Graph,
    pub net_rx_graph: Graph,
    pub net_tx_graph: Graph,
    pub cpu_label: Label,
    pub mem_label: Label,
    pub net_rx_label: Label,
    pub net_tx_label: Label,
}

fn card(title: &str, graph: &Graph, value_label: &Label) -> GtkBox {
    let card = GtkBox::new(Orientation::Vertical, 4);
    card.add_css_class("stat-card");
    card.set_margin_start(8);
    card.set_margin_end(8);
    card.set_margin_top(8);
    card.set_margin_bottom(8);

    let header = GtkBox::new(Orientation::Horizontal, 8);
    let title_label = Label::new(Some(title));
    title_label.add_css_class("card-title");
    title_label.set_xalign(0.0);
    title_label.set_hexpand(true);
    value_label.add_css_class("card-value");
    header.append(&title_label);
    header.append(value_label);

    card.append(&header);
    card.append(&graph.widget);
    card
}

pub fn build() -> OverviewView {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.set_margin_start(16);
    root.set_margin_end(16);
    root.set_margin_top(16);
    root.set_margin_bottom(16);

    let grid = Grid::new();
    grid.set_row_spacing(8);
    grid.set_column_spacing(8);
    grid.set_row_homogeneous(true);
    grid.set_column_homogeneous(true);
    grid.set_vexpand(true);

    let cpu_graph = Graph::new(ACCENT_BLUE, Some(100.0), HISTORY_LEN);
    let mem_graph = Graph::new(ACCENT_GREEN, Some(100.0), HISTORY_LEN);
    let net_rx_graph = Graph::new(ACCENT_ORANGE, None, HISTORY_LEN);
    let net_tx_graph = Graph::new(ACCENT_PURPLE, None, HISTORY_LEN);

    let cpu_label = Label::new(Some("0%"));
    let mem_label = Label::new(Some("0%"));
    let net_rx_label = Label::new(Some("0 B/s"));
    let net_tx_label = Label::new(Some("0 B/s"));

    let cpu_card = card("Procesor", &cpu_graph, &cpu_label);
    let mem_card = card("Pamięć RAM", &mem_graph, &mem_label);
    let net_card = card("Sieć ↓ pobieranie", &net_rx_graph, &net_rx_label);
    let net_tx_card = card("Sieć ↑ wysyłanie", &net_tx_graph, &net_tx_label);

    grid.attach(&cpu_card, 0, 0, 1, 1);
    grid.attach(&mem_card, 1, 0, 1, 1);
    grid.attach(&net_card, 0, 1, 1, 1);
    grid.attach(&net_tx_card, 1, 1, 1, 1);

    root.append(&grid);

    OverviewView {
        root,
        cpu_graph,
        mem_graph,
        net_rx_graph,
        net_tx_graph,
        cpu_label,
        mem_label,
        net_rx_label,
        net_tx_label,
    }
}
