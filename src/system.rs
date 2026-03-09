use sysinfo::{Disks, Networks, System};

pub struct SystemMetrics {
    system: System,
    disks: Disks,
    networks: Networks,
}

impl SystemMetrics {
    pub fn new() -> Self {
        let mut system = System::new();
        system.refresh_cpu_all();
        system.refresh_memory();

        Self {
            system,
            disks: Disks::new_with_refreshed_list(),
            networks: Networks::new_with_refreshed_list(),
        }
    }

    pub fn refresh(&mut self) {
        self.system.refresh_cpu_all();
        self.system.refresh_memory();
        self.disks.refresh();
        self.networks.refresh();
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
}
