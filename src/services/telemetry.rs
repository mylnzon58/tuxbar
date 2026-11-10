//! Pure-Rust non-blocking telemetry provider for CPU, Memory, and Temperature

use std::fs;

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemStats {
    pub cpu_usage_pct: f32,
    pub mem_used_mb: u32,
    pub mem_total_mb: u32,
    pub temp_c: f32,
}

pub struct TelemetryProvider {
    last_idle: u64,
    last_total: u64,
    cached_stats: SystemStats,
}

impl TelemetryProvider {
    #[must_use]
    pub fn new() -> Self {
        let (idle, total) = Self::read_cpu_raw();
        Self {
            last_idle: idle,
            last_total: total,
            cached_stats: SystemStats::default(),
        }
    }

    pub fn refresh(&mut self) -> &SystemStats {
        // 1. Calculate CPU usage delta
        let (idle, total) = Self::read_cpu_raw();
        let total_delta = total.saturating_sub(self.last_total);
        let idle_delta = idle.saturating_sub(self.last_idle);

        if total_delta > 0 {
            let busy_delta = total_delta.saturating_sub(idle_delta);
            self.cached_stats.cpu_usage_pct = (busy_delta as f32 / total_delta as f32) * 100.0;
        }
        self.last_idle = idle;
        self.last_total = total;

        // 2. Read memory info
        if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
            let mut total_kb = 0u64;
            let mut avail_kb = 0u64;
            for line in meminfo.lines() {
                if line.starts_with("MemTotal:") {
                    total_kb = parse_kb(line);
                } else if line.starts_with("MemAvailable:") {
                    avail_kb = parse_kb(line);
                }
            }
            if total_kb > 0 {
                let used_kb = total_kb.saturating_sub(avail_kb);
                self.cached_stats.mem_total_mb = (total_kb / 1024) as u32;
                self.cached_stats.mem_used_mb = (used_kb / 1024) as u32;
            }
        }

        // 3. Read thermal zone (CPU package temperature)
        self.cached_stats.temp_c = Self::read_cpu_temp();

        &self.cached_stats
    }

    fn read_cpu_raw() -> (u64, u64) {
        if let Ok(stat) = fs::read_to_string("/proc/stat") {
            if let Some(cpu_line) = stat.lines().next() {
                let parts: Vec<&str> = cpu_line.split_whitespace().skip(1).collect();
                if parts.len() >= 4 {
                    let user: u64 = parts[0].parse().unwrap_or(0);
                    let nice: u64 = parts[1].parse().unwrap_or(0);
                    let system: u64 = parts[2].parse().unwrap_or(0);
                    let idle: u64 = parts[3].parse().unwrap_or(0);
                    let iowait: u64 = parts.get(4).and_then(|s| s.parse().ok()).unwrap_or(0);
                    let irq: u64 = parts.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);
                    let softirq: u64 = parts.get(6).and_then(|s| s.parse().ok()).unwrap_or(0);

                    let total = user + nice + system + idle + iowait + irq + softirq;
                    return (idle + iowait, total);
                }
            }
        }
        (0, 0)
    }

    fn read_cpu_temp() -> f32 {
        // Try thermal_zone0 first (standard ACPI)
        for zone in 0..6 {
            let path = format!("/sys/class/thermal/thermal_zone{zone}/temp");
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(val) = content.trim().parse::<f32>() {
                    let temp = if val > 1000.0 { val / 1000.0 } else { val };
                    if (30.0..=115.0).contains(&temp) {
                        return temp;
                    }
                }
            }
        }
        0.0
    }
}

fn parse_kb(line: &str) -> u64 {
    line.split_whitespace()
        .nth(1)
        .and_then(|val| val.parse::<u64>().ok())
        .unwrap_or(0)
}
