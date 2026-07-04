use std::collections::VecDeque;

use sysinfo::{Disks, Networks, System};

/// Number of samples kept for every history graph (≈ 60s at 1 Hz).
pub const HISTORY_LEN: usize = 60;

/// A fixed-capacity ring buffer of `f64` samples used to back a [`crate::graph::Graph`].
#[derive(Debug, Clone)]
pub struct History {
    pub samples: VecDeque<f64>,
    capacity: usize,
}

impl History {
    pub fn new(capacity: usize) -> Self {
        let mut samples = VecDeque::with_capacity(capacity);
        samples.resize(capacity, 0.0);
        Self { samples, capacity }
    }

    pub fn push(&mut self, value: f64) {
        if self.samples.len() >= self.capacity {
            self.samples.pop_front();
        }
        self.samples.push_back(value);
    }
}

/// Snapshot of one network interface's throughput, in bytes/sec.
#[derive(Debug, Clone, Default)]
pub struct NetSample {
    pub rx_bytes_per_sec: u64,
    pub tx_bytes_per_sec: u64,
}

/// Snapshot of one disk's usage.
#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub is_removable: bool,
}

pub struct AppState {
    pub sys: System,
    pub disks: Disks,
    pub networks: Networks,

    pub cpu_overall_history: History,
    pub per_core_history: Vec<History>,
    pub memory_history: History,
    pub swap_history: History,
    pub net_rx_history: History,
    pub net_tx_history: History,

    pub net_last_totals: (u64, u64),
    pub net_current: NetSample,

    pub cpu_brand: String,
    pub cpu_physical_cores: usize,
    pub cpu_logical_cores: usize,
}

impl AppState {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let disks = Disks::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();

        let logical = sys.cpus().len();
        let physical = sys.physical_core_count().unwrap_or(logical);
        let brand = sys
            .cpus()
            .first()
            .map(|c| c.brand().trim().to_string())
            .unwrap_or_else(|| "Unknown CPU".to_string());

        let per_core_history = (0..logical).map(|_| History::new(HISTORY_LEN)).collect();

        Self {
            sys,
            disks,
            networks,
            cpu_overall_history: History::new(HISTORY_LEN),
            per_core_history,
            memory_history: History::new(HISTORY_LEN),
            swap_history: History::new(HISTORY_LEN),
            net_rx_history: History::new(HISTORY_LEN),
            net_tx_history: History::new(HISTORY_LEN),
            net_last_totals: (0, 0),
            net_current: NetSample::default(),
            cpu_brand: brand,
            cpu_physical_cores: physical,
            cpu_logical_cores: logical,
        }
    }

    /// Refresh every subsystem and push a new sample into each history buffer.
    /// Should be invoked roughly once per second.
    pub fn refresh(&mut self) {
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();
        self.sys.refresh_processes();
        self.disks.refresh();
        self.networks.refresh();

        let overall = self.sys.global_cpu_info().cpu_usage() as f64;
        self.cpu_overall_history.push(overall);

        for (i, cpu) in self.sys.cpus().iter().enumerate() {
            if let Some(hist) = self.per_core_history.get_mut(i) {
                hist.push(cpu.cpu_usage() as f64);
            }
        }

        let total_mem = self.sys.total_memory().max(1);
        let used_mem = self.sys.used_memory();
        self.memory_history
            .push(used_mem as f64 / total_mem as f64 * 100.0);

        let total_swap = self.sys.total_swap().max(1);
        let used_swap = self.sys.used_swap();
        self.swap_history
            .push(used_swap as f64 / total_swap as f64 * 100.0);

        let mut rx_total = 0u64;
        let mut tx_total = 0u64;
        for (_iface, data) in self.networks.list() {
            rx_total += data.total_received();
            tx_total += data.total_transmitted();
        }
        let (prev_rx, prev_tx) = self.net_last_totals;
        let rx_rate = rx_total.saturating_sub(prev_rx);
        let tx_rate = tx_total.saturating_sub(prev_tx);
        self.net_last_totals = (rx_total, tx_total);
        self.net_current = NetSample {
            rx_bytes_per_sec: rx_rate,
            tx_bytes_per_sec: tx_rate,
        };
        self.net_rx_history.push(rx_rate as f64);
        self.net_tx_history.push(tx_rate as f64);
    }

    pub fn used_mem_pct(&self) -> f64 {
        let total = self.sys.total_memory().max(1);
        self.sys.used_memory() as f64 / total as f64 * 100.0
    }

    pub fn disk_infos(&self) -> Vec<DiskInfo> {
        self.disks
            .list()
            .iter()
            .map(|d| DiskInfo {
                name: d.name().to_string_lossy().to_string(),
                mount_point: d.mount_point().to_string_lossy().to_string(),
                total_bytes: d.total_space(),
                available_bytes: d.available_space(),
                is_removable: d.is_removable(),
            })
            .collect()
    }
}
