use std::collections::VecDeque;
use std::time::Instant;
use sysinfo::{Disks, Networks, System};

const HISTORY_CAP: usize = 60;

pub struct SystemMetrics {
    system: System,
    disks: Disks,
    networks: Networks,
    pub cpu_history: VecDeque<f64>,
    pub ram_history: VecDeque<f64>,
    pub net_rx_rate: f64,
    pub net_tx_rate: f64,
    pub net_rx_history: VecDeque<u64>,
    pub net_tx_history: VecDeque<u64>,
    prev_rx: u64,
    prev_tx: u64,
    last_refresh: Instant,
}

impl SystemMetrics {
    pub fn new() -> Self {
        let mut system = System::new();
        system.refresh_cpu_all();
        system.refresh_memory();

        let networks = Networks::new_with_refreshed_list();
        let prev_rx: u64 = networks.iter().map(|(_, n)| n.total_received()).sum();
        let prev_tx: u64 = networks.iter().map(|(_, n)| n.total_transmitted()).sum();

        Self {
            system,
            disks: Disks::new_with_refreshed_list(),
            networks,
            cpu_history: VecDeque::with_capacity(HISTORY_CAP),
            ram_history: VecDeque::with_capacity(HISTORY_CAP),
            net_rx_rate: 0.0,
            net_tx_rate: 0.0,
            net_rx_history: VecDeque::with_capacity(HISTORY_CAP),
            net_tx_history: VecDeque::with_capacity(HISTORY_CAP),
            prev_rx,
            prev_tx,
            last_refresh: Instant::now(),
        }
    }

    pub fn refresh(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refresh).as_secs_f64();

        self.system.refresh_cpu_all();
        self.system.refresh_memory();
        self.disks.refresh();
        self.networks.refresh();

        // Push CPU and RAM history
        let cpu = self.cpu_usage() as f64;
        let ram = self.memory_usage_pct() as f64;

        if self.cpu_history.len() >= HISTORY_CAP {
            self.cpu_history.pop_front();
        }
        self.cpu_history.push_back(cpu);

        if self.ram_history.len() >= HISTORY_CAP {
            self.ram_history.pop_front();
        }
        self.ram_history.push_back(ram);

        // Network throughput
        let cur_rx: u64 = self.networks.iter().map(|(_, n)| n.total_received()).sum();
        let cur_tx: u64 = self.networks.iter().map(|(_, n)| n.total_transmitted()).sum();

        if elapsed > 0.0 {
            let delta_rx = cur_rx.saturating_sub(self.prev_rx);
            let delta_tx = cur_tx.saturating_sub(self.prev_tx);
            self.net_rx_rate = delta_rx as f64 / elapsed;
            self.net_tx_rate = delta_tx as f64 / elapsed;
        }

        if self.net_rx_history.len() >= HISTORY_CAP {
            self.net_rx_history.pop_front();
        }
        self.net_rx_history.push_back(self.net_rx_rate as u64);

        if self.net_tx_history.len() >= HISTORY_CAP {
            self.net_tx_history.pop_front();
        }
        self.net_tx_history.push_back(self.net_tx_rate as u64);

        self.prev_rx = cur_rx;
        self.prev_tx = cur_tx;
        self.last_refresh = now;
    }

    pub fn cpu_usage(&self) -> f32 {
        self.system.global_cpu_usage()
    }

    pub fn total_memory(&self) -> u64 {
        self.system.total_memory()
    }

    pub fn available_memory(&self) -> u64 {
        self.system.available_memory()
    }

    pub fn memory_usage_pct(&self) -> f32 {
        let total = self.system.total_memory();
        if total == 0 {
            0.0
        } else {
            (total - self.system.available_memory()) as f32 / total as f32 * 100.0
        }
    }

    pub fn disk_info(&self) -> Vec<DiskInfo> {
        self.disks
            .iter()
            .filter(|d| d.total_space() > 0)
            .map(|d| DiskInfo {
                name: d.name().to_string_lossy().to_string(),
                mount_point: d.mount_point().to_string_lossy().to_string(),
                total: d.total_space(),
                available: d.available_space(),
            })
            .collect()
    }

    pub fn disk_usage_pct(&self) -> f32 {
        self.disks
            .iter()
            .find(|d| d.total_space() > 0)
            .map(|d| {
                (d.total_space() - d.available_space()) as f32 / d.total_space() as f32 * 100.0
            })
            .unwrap_or(0.0)
    }

    pub fn network_received(&self) -> u64 {
        self.networks.iter().map(|(_, n)| n.total_received()).sum()
    }

    pub fn network_transmitted(&self) -> u64 {
        self.networks.iter().map(|(_, n)| n.total_transmitted()).sum()
    }

    pub fn cpu_trend(&self) -> &'static str {
        Self::trend(&self.cpu_history)
    }

    pub fn ram_trend(&self) -> &'static str {
        Self::trend(&self.ram_history)
    }

    fn trend(history: &VecDeque<f64>) -> &'static str {
        if history.len() < 5 {
            return "\u{2192}"; // →
        }
        let len = history.len();
        let recent_avg = (history[len - 1] + history[len - 2]) / 2.0;
        let old_avg = (history[len - 5] + history[len - 4]) / 2.0;
        let diff = recent_avg - old_avg;
        if diff > 2.0 {
            "\u{2191}" // ↑
        } else if diff < -2.0 {
            "\u{2193}" // ↓
        } else {
            "\u{2192}" // →
        }
    }
}

pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total: u64,
    pub available: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_metrics_basic() {
        let mut metrics = SystemMetrics::new();
        metrics.refresh();

        assert!(metrics.cpu_usage() >= 0.0);
        assert!(metrics.memory_usage_pct() >= 0.0);
        assert!(metrics.disk_usage_pct() >= 0.0);
    }

    #[test]
    fn test_memory_calculation() {
        let metrics = SystemMetrics::new();
        let total = metrics.total_memory();
        let available = metrics.available_memory();

        assert!(total >= available);
    }

    #[test]
    fn test_history_accumulates() {
        let mut metrics = SystemMetrics::new();
        for _ in 0..5 {
            metrics.refresh();
        }
        assert_eq!(metrics.cpu_history.len(), 5);
        assert_eq!(metrics.ram_history.len(), 5);
    }

    #[test]
    fn test_history_caps_at_60() {
        let mut metrics = SystemMetrics::new();
        for _ in 0..70 {
            metrics.refresh();
        }
        assert_eq!(metrics.cpu_history.len(), HISTORY_CAP);
        assert_eq!(metrics.ram_history.len(), HISTORY_CAP);
    }

    #[test]
    fn test_trend_stable() {
        let mut history = VecDeque::new();
        for _ in 0..5 {
            history.push_back(50.0);
        }
        assert_eq!(SystemMetrics::trend(&history), "\u{2192}");
    }

    #[test]
    fn test_trend_insufficient_data() {
        let mut history = VecDeque::new();
        history.push_back(10.0);
        history.push_back(20.0);
        assert_eq!(SystemMetrics::trend(&history), "\u{2192}");
    }
}
