use anyhow::{Context, Result};

#[derive(Debug, Clone, Copy)]
pub struct CpuSnapshot {
    pub idle: u64,
    pub total: u64,
}

pub fn read_active_window() -> Result<String> {
    Ok(crate::hyprland::get_active_window_title()?.to_owned())
}

pub fn read_cpu_snapshot() -> Result<CpuSnapshot> {
    let stat = std::fs::read_to_string("/proc/stat").context("failed to read /proc/stat")?;

    let first_line = stat
        .lines()
        .next()
        .context("missing first line in /proc/stat")?;

    let mut parts = first_line.split_whitespace();

    let cpu = parts.next().context("missing cpu label")?;

    if cpu != "cpu" {
        anyhow::bail!("first /proc/stat line is not cpu");
    }

    let values = parts
        .map(str::parse::<u64>)
        .collect::<Result<Vec<_>, _>>()
        .context("failed to parse /proc/stat cpu values")?;

    if values.len() < 7 {
        anyhow::bail!("not enough cpu fields in /proc/stat");
    }

    let user = values[0];
    let nice = values[1];
    let system = values[2];
    let idle = values[3];
    let iowait = values[4];
    let irq = values[5];
    let softirq = values[6];
    let steal = values.get(7).copied().unwrap_or(0);

    let idle_all = idle + iowait;
    let total = user + nice + system + idle + iowait + irq + softirq + steal;

    Ok(CpuSnapshot {
        idle: idle_all,
        total,
    })
}

pub fn cpu_usage_percent(prev: CpuSnapshot, next: CpuSnapshot) -> f64 {
    let total_delta = next.total.saturating_sub(prev.total);
    let idle_delta = next.idle.saturating_sub(prev.idle);

    if total_delta == 0 {
        return 0.0;
    }

    let used = total_delta.saturating_sub(idle_delta);

    used as f64 * 100.0 / total_delta as f64
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryInfo {
    pub total_kib: u64,
    pub available_kib: u64,
    pub used_kib: u64,
    pub used_percent: f64,
}

pub fn read_memory_info() -> anyhow::Result<MemoryInfo> {
    let meminfo = std::fs::read_to_string("/proc/meminfo")?;

    let mut total_kib = None;
    let mut available_kib = None;

    for line in meminfo.lines() {
        if let Some(value) = parse_meminfo_value(line, "MemTotal:") {
            total_kib = Some(value);
        } else if let Some(value) = parse_meminfo_value(line, "MemAvailable:") {
            available_kib = Some(value);
        }
    }

    let total_kib = total_kib.ok_or_else(|| anyhow::anyhow!("missing MemTotal"))?;
    let available_kib = available_kib.ok_or_else(|| anyhow::anyhow!("missing MemAvailable"))?;

    let used_kib = total_kib.saturating_sub(available_kib);

    let used_percent = if total_kib == 0 {
        0.0
    } else {
        used_kib as f64 * 100.0 / total_kib as f64
    };

    Ok(MemoryInfo {
        total_kib,
        available_kib,
        used_kib,
        used_percent,
    })
}

fn parse_meminfo_value(line: &str, key: &str) -> Option<u64> {
    let rest = line.strip_prefix(key)?;

    rest.split_whitespace().next()?.parse::<u64>().ok()
}

pub fn format_kib(kib: u64) -> String {
    let mib = kib as f64 / 1024.0;
    let gib = mib / 1024.0;

    if gib >= 1.0 {
        format!("{gib:.1}G")
    } else {
        format!("{mib:.0}M")
    }
}
