use serde::Serialize;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct CpuTemp {
    pub temp_c: f64,
    pub label: String,
    pub color: String, // "green", "yellow", "red"
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemVitals {
    pub cpu_percent: f64,
    pub mem_used_gb: f64,
    pub mem_total_gb: f64,
    pub mem_percent: f64,
    pub disk_used_gb: f64,
    pub disk_total_gb: f64,
    pub disk_percent: f64,
    pub media_used_gb: f64,
    pub media_total_gb: f64,
    pub media_percent: f64,
    pub cpu_temp: Option<CpuTemp>,
}

pub async fn get_vitals() -> SystemVitals {
    let cpu = get_cpu().await;
    let (mem_used, mem_total) = get_memory().await;
    let (disk_used, disk_total) = get_disk("/").await;
    let (media_used, media_total) = get_disk("/mnt/media").await;
    let cpu_temp = get_cpu_temp().await;

    SystemVitals {
        cpu_percent: cpu,
        mem_used_gb: mem_used,
        mem_total_gb: mem_total,
        mem_percent: if mem_total > 0.0 { (mem_used / mem_total) * 100.0 } else { 0.0 },
        disk_used_gb: disk_used,
        disk_total_gb: disk_total,
        disk_percent: if disk_total > 0.0 { (disk_used / disk_total) * 100.0 } else { 0.0 },
        media_used_gb: media_used,
        media_total_gb: media_total,
        media_percent: if media_total > 0.0 { (media_used / media_total) * 100.0 } else { 0.0 },
        cpu_temp,
    }
}

async fn get_cpu() -> f64 {
    let output = Command::new("sh")
        .arg("-c")
        .arg("top -bn1 | grep 'Cpu(s)' | awk '{print $2}'")
        .output().await;
    match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim().parse().unwrap_or(0.0),
        Err(_) => 0.0,
    }
}

async fn get_memory() -> (f64, f64) {
    let output = Command::new("sh")
        .arg("-c")
        .arg("free -b | awk '/Mem:/{printf \"%.2f %.2f\", $3/1073741824, $2/1073741824}'")
        .output().await;
    match output {
        Ok(o) => {
            let s = String::from_utf8_lossy(&o.stdout);
            let parts: Vec<f64> = s.trim().split_whitespace()
                .filter_map(|p| p.parse().ok()).collect();
            if parts.len() == 2 { (parts[0], parts[1]) } else { (0.0, 0.0) }
        }
        Err(_) => (0.0, 0.0),
    }
}

async fn get_disk(path: &str) -> (f64, f64) {
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("df -B1 {} 2>/dev/null | awk 'NR==2{{printf \"%.2f %.2f\", $3/1073741824, $2/1073741824}}'", path))
        .output().await;
    match output {
        Ok(o) => {
            let s = String::from_utf8_lossy(&o.stdout);
            let parts: Vec<f64> = s.trim().split_whitespace()
                .filter_map(|p| p.parse().ok()).collect();
            if parts.len() == 2 { (parts[0], parts[1]) } else { (0.0, 0.0) }
        }
        Err(_) => (0.0, 0.0),
    }
}

async fn get_cpu_temp() -> Option<CpuTemp> {
    // Try to find the CPU package temp (x86_pkg_temp) first, then fall back
    let zones = ["thermal_zone6", "thermal_zone0", "thermal_zone2"];

    for zone in &zones {
        let temp_path = format!("/sys/class/thermal/{}/temp", zone);
        if let Ok(content) = tokio::fs::read_to_string(&temp_path).await {
            if let Ok(millideg) = content.trim().parse::<f64>() {
                let temp_c = millideg / 1000.0;
                let color = if temp_c < 60.0 {
                    "green"
                } else if temp_c < 80.0 {
                    "yellow"
                } else {
                    "red"
                };

                // Read the zone type for label
                let type_path = format!("/sys/class/thermal/{}/type", zone);
                let label = tokio::fs::read_to_string(&type_path)
                    .await
                    .unwrap_or_else(|_| "CPU".into())
                    .trim()
                    .to_string();

                return Some(CpuTemp {
                    temp_c,
                    label,
                    color: color.into(),
                });
            }
        }
    }

    // Fallback: try any zone
    let output = Command::new("sh")
        .arg("-c")
        .arg("cat /sys/class/thermal/thermal_zone*/temp 2>/dev/null | head -1")
        .output()
        .await;

    if let Ok(o) = output {
        let s = String::from_utf8_lossy(&o.stdout);
        if let Ok(millideg) = s.trim().parse::<f64>() {
            let temp_c = millideg / 1000.0;
            let color = if temp_c < 60.0 {
                "green"
            } else if temp_c < 80.0 {
                "yellow"
            } else {
                "red"
            };
            return Some(CpuTemp {
                temp_c,
                label: "CPU".into(),
                color: color.into(),
            });
        }
    }

    None
}
