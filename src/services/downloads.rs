use serde::Serialize;
use reqwest::Client;

#[derive(Debug, Clone, Serialize)]
pub struct TorrentInfo {
    pub name: String,
    pub progress: f64,       // 0.0 - 100.0
    pub dlspeed: u64,        // bytes/sec
    pub upspeed: u64,        // bytes/sec
    pub size: u64,           // total bytes
    pub state: String,
    pub eta: i64,            // seconds, -1 = unknown
}

#[derive(Debug, Clone, Serialize)]
pub struct DownloadStats {
    pub torrents: Vec<TorrentInfo>,
    pub active_count: usize,
    pub total_dl_speed: String,
    pub total_up_speed: String,
    pub total_dl_bytes: u64,
    pub total_up_bytes: u64,
}

fn format_speed(bytes_per_sec: u64) -> String {
    if bytes_per_sec >= 1_048_576 {
        format!("{:.1} MB/s", bytes_per_sec as f64 / 1_048_576.0)
    } else if bytes_per_sec >= 1024 {
        format!("{:.0} KB/s", bytes_per_sec as f64 / 1024.0)
    } else {
        format!("{} B/s", bytes_per_sec)
    }
}

pub async fn get_downloads() -> DownloadStats {
    match fetch_torrents().await {
        Ok(stats) => stats,
        Err(e) => {
            eprintln!("qBittorrent API error: {}", e);
            DownloadStats {
                torrents: vec![],
                active_count: 0,
                total_dl_speed: "0 B/s".into(),
                total_up_speed: "0 B/s".into(),
                total_dl_bytes: 0,
                total_up_bytes: 0,
            }
        }
    }
}

async fn fetch_torrents() -> Result<DownloadStats, Box<dyn std::error::Error>> {
    let qbit_url = std::env::var("QBIT_URL").unwrap_or_else(|_| "http://localhost:8080".into());
    let qbit_user = std::env::var("QBIT_USERNAME").unwrap_or_else(|_| "admin".into());
    let qbit_pass = std::env::var("QBIT_PASSWORD").unwrap_or_else(|_| "".into());

    let client = Client::builder()
        .cookie_store(true)
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    // Login
    client.post(format!("{}/api/v2/auth/login", qbit_url))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(format!("username={}&password={}", qbit_user, qbit_pass))
        .send()
        .await?;

    // Get transfer info for global speeds
    let transfer_resp = client.get(format!("{}/api/v2/transfer/info", qbit_url))
        .send().await?;
    let transfer: serde_json::Value = transfer_resp.json().await?;

    let global_dl = transfer["dl_info_speed"].as_u64().unwrap_or(0);
    let global_up = transfer["up_info_speed"].as_u64().unwrap_or(0);

    // Get torrent list
    let torrents_resp = client
        .get(format!("{}/api/v2/torrents/info?filter=all&sort=added_on&reverse=true&limit=20", qbit_url))
        .send().await?;
    let torrents_json: Vec<serde_json::Value> = torrents_resp.json().await?;

    let mut torrents = Vec::new();
    let mut active_count = 0;

    for t in &torrents_json {
        let state = t["state"].as_str().unwrap_or("unknown").to_string();
        let progress = t["progress"].as_f64().unwrap_or(0.0) * 100.0;
        let dlspeed = t["dlspeed"].as_u64().unwrap_or(0);
        let upspeed = t["upspeed"].as_u64().unwrap_or(0);

        let is_active = matches!(state.as_str(), 
            "downloading" | "uploading" | "stalledDL" | "forcedDL" | "metaDL" | "queuedDL" | "allocating"
        ) && progress < 100.0;

        if is_active {
            active_count += 1;
        }

        // Shorten name: remove common release group tags for cleaner display
        let name = t["name"].as_str().unwrap_or("Unknown").to_string();

        torrents.push(TorrentInfo {
            name,
            progress,
            dlspeed,
            upspeed,
            size: t["size"].as_u64().unwrap_or(0),
            state,
            eta: t["eta"].as_i64().unwrap_or(-1),
        });
    }

    // Sort: active/downloading first, then by progress ascending
    torrents.sort_by(|a, b| {
        let a_active = a.progress < 100.0;
        let b_active = b.progress < 100.0;
        b_active.cmp(&a_active)
            .then(a.progress.partial_cmp(&b.progress).unwrap_or(std::cmp::Ordering::Equal))
    });

    // Limit to top 8 for display
    torrents.truncate(8);

    Ok(DownloadStats {
        torrents,
        active_count,
        total_dl_speed: format_speed(global_dl),
        total_up_speed: format_speed(global_up),
        total_dl_bytes: global_dl,
        total_up_bytes: global_up,
    })
}
