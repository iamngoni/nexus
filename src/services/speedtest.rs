use serde::Serialize;
use reqwest::Client;

#[derive(Debug, Clone, Serialize)]
pub struct SpeedtestResult {
    pub download_mbps: String,
    pub upload_mbps: String,
    pub ping_ms: String,
    pub timestamp: String,
    pub failed: bool,
    pub available: bool,
}

pub async fn get_speedtest() -> SpeedtestResult {
    match fetch_speedtest().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Speedtest API error: {}", e);
            SpeedtestResult {
                download_mbps: "--".into(),
                upload_mbps: "--".into(),
                ping_ms: "--".into(),
                timestamp: String::new(),
                failed: true,
                available: false,
            }
        }
    }
}

async fn fetch_speedtest() -> Result<SpeedtestResult, Box<dyn std::error::Error>> {
    let url = std::env::var("SPEEDTEST_URL").unwrap_or_else(|_| "http://localhost:8765".into());
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    let resp: serde_json::Value = client
        .get(format!("{}/api/speedtest/latest", url))
        .send()
        .await?
        .json()
        .await?;

    let data = &resp["data"];
    let failed = data["failed"].as_bool().unwrap_or(true);

    if failed {
        let timestamp = data["created_at"].as_str().unwrap_or("");
        let ts = format_timestamp(timestamp);
        return Ok(SpeedtestResult {
            download_mbps: "--".into(),
            upload_mbps: "--".into(),
            ping_ms: "--".into(),
            timestamp: ts,
            failed: true,
            available: true,
        });
    }

    // Speedtest Tracker stores values in bits/sec, need to convert to Mbps
    let download = data["download"].as_f64().unwrap_or(0.0);
    let upload = data["upload"].as_f64().unwrap_or(0.0);
    let ping = data["ping"].as_f64().unwrap_or(0.0);
    let timestamp = data["created_at"].as_str().unwrap_or("");

    // The API may return in bits/sec or Mbps depending on version
    // If value > 1000, it's likely bits/sec; convert to Mbps
    let dl_mbps = if download > 1000.0 {
        download / 1_000_000.0
    } else {
        download
    };
    let ul_mbps = if upload > 1000.0 {
        upload / 1_000_000.0
    } else {
        upload
    };

    Ok(SpeedtestResult {
        download_mbps: format!("{:.1}", dl_mbps),
        upload_mbps: format!("{:.1}", ul_mbps),
        ping_ms: format!("{:.0}", ping),
        timestamp: format_timestamp(timestamp),
        failed: false,
        available: true,
    })
}

fn format_timestamp(iso: &str) -> String {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(iso) {
        dt.with_timezone(&chrono::Local)
            .format("%H:%M · %d %b")
            .to_string()
    } else if !iso.is_empty() {
        // Try without timezone
        iso.chars().take(16).collect()
    } else {
        String::new()
    }
}
