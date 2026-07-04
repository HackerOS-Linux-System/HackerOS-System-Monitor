use std::cell::RefCell;
use std::rc::Rc;

use gtk::glib;
use gtk::prelude::*;
use gtk::{
    Align, Application, ApplicationWindow, Box as GtkBox, HeaderBar, Image, Label, ListBox,
    ListBoxRow, Orientation, Stack, StackTransitionType,
};

use crate::stats::AppState;
use crate::views;

const APP_ID: &str = "org.debian.MissionMonitor";

pub fn run() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_startup(|_| load_css());
    app.connect_activate(build_ui);
    app.run()
}

fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(include_str!("style.css"));
    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("no display available"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

struct NavEntry {
    id: &'static str,
    icon: &'static str,
    label: &'static str,
}

const NAV_ENTRIES: &[NavEntry] = &[
    NavEntry { id: "overview", icon: "go-home-symbolic", label: "Przegląd" },
    NavEntry { id: "cpu", icon: "applications-system-symbolic", label: "Procesor" },
    NavEntry { id: "memory", icon: "drive-harddisk-solidstate-symbolic", label: "Pamięć" },
    NavEntry { id: "disk", icon: "drive-multidisk-symbolic", label: "Dyski" },
    NavEntry { id: "network", icon: "network-wired-symbolic", label: "Sieć" },
    NavEntry { id: "processes", icon: "view-list-symbolic", label: "Procesy" },
];

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Monitor Systemu")
        .default_width(1000)
        .default_height(680)
        .build();

    let header = HeaderBar::new();
    window.set_titlebar(Some(&header));

    let root = GtkBox::new(Orientation::Horizontal, 0);

    // --- Sidebar -----------------------------------------------------
    let sidebar = ListBox::new();
    sidebar.add_css_class("sidebar-nav");
    sidebar.set_width_request(190);
    sidebar.set_selection_mode(gtk::SelectionMode::Single);

    let stack = Stack::new();
    stack.set_transition_type(StackTransitionType::Crossfade);
    stack.set_hexpand(true);
    stack.set_vexpand(true);

    for entry in NAV_ENTRIES {
        let row = build_nav_row(entry.icon, entry.label);
        row.set_widget_name(entry.id);
        sidebar.append(&row);
    }

    // --- Build each view and stash handles for the refresh loop ------
    let state = Rc::new(RefCell::new(AppState::new()));

    let overview = views::overview::build();
    let cpu_view = {
        let logical = state.borrow().cpu_logical_cores;
        views::cpu::build(logical)
    };
    let memory_view = views::memory::build();
    let disk_view = views::disk::build();
    let network_view = views::network::build();
    let processes_view = views::processes::build();

    stack.add_named(&overview.root, Some("overview"));
    stack.add_named(&cpu_view.root, Some("cpu"));
    stack.add_named(&memory_view.root, Some("memory"));
    stack.add_named(&disk_view.root, Some("disk"));
    stack.add_named(&network_view.root, Some("network"));
    stack.add_named(&processes_view.root, Some("processes"));

    {
        let stack = stack.clone();
        sidebar.connect_row_selected(move |_, row| {
            if let Some(row) = row {
                let name = row.widget_name();
                stack.set_visible_child_name(&name);
            }
        });
    }
    if let Some(first) = sidebar.row_at_index(0) {
        sidebar.select_row(Some(&first));
    }

    root.append(&sidebar);
    root.append(&gtk::Separator::new(Orientation::Vertical));
    root.append(&stack);
    window.set_child(Some(&root));
    window.present();

    // Static CPU info label, set once.
    {
        let s = state.borrow();
        cpu_view.info_label.set_text(&format!(
            "{}  •  {} rdzeni fizycznych  •  {} wątków logicznych",
            s.cpu_brand, s.cpu_physical_cores, s.cpu_logical_cores
        ));
    }

    // --- Refresh loop (1 Hz) -----------------------------------------
    let tick_state = state.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(1000), move || {
        let mut s = tick_state.borrow_mut();
        s.refresh();

        // Overview
        overview.cpu_graph.set_data(&s.cpu_overall_history.samples);
        overview.mem_graph.set_data(&s.memory_history.samples);
        overview.net_rx_graph.set_data(&s.net_rx_history.samples);
        overview.net_tx_graph.set_data(&s.net_tx_history.samples);
        overview
            .cpu_label
            .set_text(&format!("{:.0}%", s.sys.global_cpu_info().cpu_usage()));
        let mem_pct = s.used_mem_pct();
        overview.mem_label.set_text(&format!("{mem_pct:.0}%"));
        overview
            .net_rx_label
            .set_text(&human_bytes_per_sec(s.net_current.rx_bytes_per_sec));
        overview
            .net_tx_label
            .set_text(&human_bytes_per_sec(s.net_current.tx_bytes_per_sec));

        // CPU view
        cpu_view
            .overall_graph
            .set_data(&s.cpu_overall_history.samples);
        cpu_view
            .overall_label
            .set_text(&format!("{:.0}%", s.sys.global_cpu_info().cpu_usage()));
        for (i, g) in cpu_view.core_graphs.iter().enumerate() {
            if let Some(hist) = s.per_core_history.get(i) {
                g.set_data(&hist.samples);
            }
        }

        // Memory view
        memory_view.ram_graph.set_data(&s.memory_history.samples);
        memory_view.swap_graph.set_data(&s.swap_history.samples);
        memory_view.ram_label.set_text(&format!(
            "{} / {}",
            human_bytes(s.sys.used_memory()),
            human_bytes(s.sys.total_memory())
        ));
        memory_view.swap_label.set_text(&format!(
            "{} / {}",
            human_bytes(s.sys.used_swap()),
            human_bytes(s.sys.total_swap())
        ));
        memory_view.breakdown_label.set_text(&format!(
            "Dostępne: {}",
            human_bytes(s.sys.available_memory())
        ));

        // Disk view
        views::disk::refresh(&disk_view, &s.disk_infos());

        // Network view
        network_view.rx_graph.set_data(&s.net_rx_history.samples);
        network_view.tx_graph.set_data(&s.net_tx_history.samples);
        network_view
            .rx_label
            .set_text(&human_bytes_per_sec(s.net_current.rx_bytes_per_sec));
        network_view
            .tx_label
            .set_text(&human_bytes_per_sec(s.net_current.tx_bytes_per_sec));
        views::network::refresh_interfaces(&network_view, &s.networks);

        // Processes view
        views::processes::refresh(&processes_view, &s.sys);

        glib::ControlFlow::Continue
    });
}

fn build_nav_row(icon_name: &str, label: &str) -> ListBoxRow {
    let hbox = GtkBox::new(Orientation::Horizontal, 10);
    hbox.set_margin_top(4);
    hbox.set_margin_bottom(4);
    let image = Image::from_icon_name(icon_name);
    image.set_pixel_size(18);
    let text = Label::new(Some(label));
    text.set_xalign(0.0);
    text.set_halign(Align::Start);
    hbox.append(&image);
    hbox.append(&text);

    let row = ListBoxRow::new();
    row.set_child(Some(&hbox));
    row
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

fn human_bytes_per_sec(bytes: u64) -> String {
    format!("{}/s", human_bytes(bytes))
}
