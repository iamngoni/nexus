use serde::Serialize;
use tokio::process::Command;

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
}

pub async fn get_vitals() -> SystemVitals {
    let cpu = get_cpu().await;
    let (mem_used, mem_total) = get_memory().await;
    let (disk_used, disk_total) = get_disk("/").await;
    let (media_used, media_total) = get_disk("/mnt/media").await;

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
