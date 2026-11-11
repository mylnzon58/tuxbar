//! Pure-Rust non-blocking telemetry provider for CPU, Memory, and Temperature

use std::fs;

#[derive(Debug, Clone, Copy)]
pub struct SystemStats {
    pub cpu_usage_pct: f32,
    pub mem_used_mb: u32,
    pub mem_total_mb: u32,
    pub temp_c: f32,
    pub core_count: usize,
    pub core_usage_pct: [f32; 16],
}

impl Default for SystemStats {
    fn default() -> Self {
        Self {
            cpu_usage_pct: 0.0,
            mem_used_mb: 0,
            mem_total_mb: 0,
            temp_c: 0.0,
            core_count: 0,
            core_usage_pct: [0.0; 16],
        }
    }
}

pub struct TelemetryProvider {
    last_idle: u64,
    last_total: u64,
    last_core_ticks: [(u64, u64); 16],
    cached_stats: SystemStats,
}

impl TelemetryProvider {
    #[must_use]
    pub fn new() -> Self {
        let (idle, total) = Self::read_cpu_raw();
        let mut core_ticks = [(0, 0); 16];
        if let Ok(stat) = fs::read_to_string("/proc/stat") {
            for line in stat.lines() {
                if line.starts_with("cpu") && !line.starts_with("cpu ") {
                    let first_word = line.split_whitespace().next().unwrap_or("");
                    if let Ok(core_id) = first_word[3..].parse::<usize>() {
                        if core_id < 16 {
                            core_ticks[core_id] = parse_cpu_line(line);
                        }
                    }
                }
            }
        }

        Self {
            last_idle: idle,
            last_total: total,
            last_core_ticks: core_ticks,
            cached_stats: SystemStats::default(),
        }
    }

    pub fn refresh(&mut self) -> &SystemStats {
        // 1. Calculate CPU aggregate and per-core deltas from /proc/stat
        if let Ok(stat) = fs::read_to_string("/proc/stat") {
            let mut detected_cores = 0;
            for line in stat.lines() {
                if line.starts_with("cpu ") {
                    let (idle, total) = parse_cpu_line(line);
                    let total_delta = total.saturating_sub(self.last_total);
                    let idle_delta = idle.saturating_sub(self.last_idle);
                    if total_delta > 0 {
                        let busy_delta = total_delta.saturating_sub(idle_delta);
                        self.cached_stats.cpu_usage_pct =
                            (busy_delta as f32 / total_delta as f32) * 100.0;
                    }
                    self.last_idle = idle;
                    self.last_total = total;
                } else if line.starts_with("cpu") {
                    let first_word = line.split_whitespace().next().unwrap_or("");
                    if let Ok(core_id) = first_word[3..].parse::<usize>() {
                        if core_id < 16 {
                            let (idle, total) = parse_cpu_line(line);
                            let (last_idle, last_total) = self.last_core_ticks[core_id];
                            let total_delta = total.saturating_sub(last_total);
                            let idle_delta = idle.saturating_sub(last_idle);
                            if total_delta > 0 {
                                let busy = total_delta.saturating_sub(idle_delta);
                                self.cached_stats.core_usage_pct[core_id] =
                                    (busy as f32 / total_delta as f32) * 100.0;
                            }
                            self.last_core_ticks[core_id] = (idle, total);
                            detected_cores = detected_cores.max(core_id + 1);
                        }
                    }
                }
            }
            self.cached_stats.core_count = detected_cores;
        }

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
                return parse_cpu_line(cpu_line);
            }
        }
        (0, 0)
    }

    fn read_cpu_temp() -> f32 {
        // Priority 1: Intel coretemp hwmon Package id 0 (true CPU package temperature)
        if let Ok(entries) = fs::read_dir("/sys/class/hwmon") {
            for entry in entries.flatten() {
                let p = entry.path();
                if let Ok(name) = fs::read_to_string(p.join("name")) {
                    if name.trim() == "coretemp" {
                        if let Ok(val_str) = fs::read_to_string(p.join("temp1_input")) {
                            if let Ok(val) = val_str.trim().parse::<f32>() {
                                let temp = if val > 1000.0 { val / 1000.0 } else { val };
                                if (20.0..=115.0).contains(&temp) {
                                    return temp;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Priority 2: x86_pkg_temp in thermal_zone
        if let Ok(entries) = fs::read_dir("/sys/class/thermal") {
            for entry in entries.flatten() {
                let p = entry.path();
                let fname = entry.file_name();
                if fname.to_string_lossy().starts_with("thermal_zone") {
                    if let Ok(ztype) = fs::read_to_string(p.join("type")) {
                        let t = ztype.trim().to_lowercase();
                        if t.contains("pkg") || t.contains("x86_pkg") || t.contains("cpu") {
                            if let Ok(val_str) = fs::read_to_string(p.join("temp")) {
                                if let Ok(val) = val_str.trim().parse::<f32>() {
                                    let temp = if val > 1000.0 { val / 1000.0 } else { val };
                                    if (20.0..=115.0).contains(&temp) {
                                        return temp;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Priority 3: Fallback thermal zones (skipping battery)
        for zone in 0..10 {
            let path = format!("/sys/class/thermal/thermal_zone{zone}");
            let type_path = format!("{path}/type");
            if let Ok(ztype) = fs::read_to_string(&type_path) {
                if ztype.contains("battery") {
                    continue;
                }
            }
            let temp_path = format!("{path}/temp");
            if let Ok(content) = fs::read_to_string(&temp_path) {
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

fn parse_cpu_line(line: &str) -> (u64, u64) {
    let parts: Vec<&str> = line.split_whitespace().skip(1).collect();
    if parts.len() >= 4 {
        let user: u64 = parts[0].parse().unwrap_or(0);
        let nice: u64 = parts[1].parse().unwrap_or(0);
        let system: u64 = parts[2].parse().unwrap_or(0);
        let idle: u64 = parts[3].parse().unwrap_or(0);
        let iowait: u64 = parts.get(4).and_then(|s| s.parse().ok()).unwrap_or(0);
        let irq: u64 = parts.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);
        let softirq: u64 = parts.get(6).and_then(|s| s.parse().ok()).unwrap_or(0);

        let total = user + nice + system + idle + iowait + irq + softirq;
        (idle + iowait, total)
    } else {
        (0, 0)
    }
}

fn parse_kb(line: &str) -> u64 {
    line.split_whitespace()
        .nth(1)
        .and_then(|val| val.parse::<u64>().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_kb() {
        assert_eq!(parse_kb("MemTotal:       16049216 kB"), 16049216);
        assert_eq!(parse_kb("MemAvailable:    3754944 kB"), 3754944);
        assert_eq!(parse_kb("Invalid line"), 0);
        assert_eq!(parse_kb(""), 0);
    }

    #[test]
    fn test_telemetry_provider_live() {
        let mut provider = TelemetryProvider::new();
        let stats = provider.refresh();
        // Total memory on real system should be > 0
        assert!(stats.mem_total_mb > 0);
        // CPU percentage clamped
        assert!(stats.cpu_usage_pct >= 0.0 && stats.cpu_usage_pct <= 100.0);
    }
}

